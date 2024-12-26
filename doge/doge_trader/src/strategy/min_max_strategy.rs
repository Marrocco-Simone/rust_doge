use std::cell::{Ref, RefCell};
use std::rc::Rc;

use itertools::Itertools;
use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::good::good_kind::GoodKind::{EUR, USD, YEN, YUAN};
use unitn_market_2022::market::Market;

use doge_common::account::Account;

use crate::DOGE_TRADER_NAME;
use crate::event::TradingEvent;
use crate::event::TradingEventType::{Buy, LockBuy, LockSell, Sell};
use crate::event_handler::EventHandler;
use crate::markets::MarketWrapper;
use crate::strategy::Strategy;

pub struct MinMaxStrategy {
    handlers: Vec<Box<dyn EventHandler>>,
}

impl MinMaxStrategy {
    pub fn new() -> MinMaxStrategy {
        MinMaxStrategy {
            handlers: vec![],
        }
    }
}

struct Exchange {
    kind: GoodKind,
    seller: Rc<RefCell<dyn Market>>,
    bid: f32,
    buyer: Rc<RefCell<dyn Market>>,
    offer: f32,
    quantity: f32,
}

impl Strategy for MinMaxStrategy {
    fn apply(&mut self, account: &mut Account, markets: &mut MarketWrapper) {
        if let Some(Exchange { kind, seller, bid, buyer, mut offer, quantity }) = find_exchange(markets, account) {
            let result = seller.borrow_mut().lock_buy(kind, quantity, bid, DOGE_TRADER_NAME.to_owned());

            for handler in &mut self.handlers {
                let event = TradingEvent {
                    account: account.clone(),
                    market: seller.borrow().get_name().to_owned(),
                    event: LockBuy { kind, quantity, bid, result: result.clone() },
                };
                handler.handle_event(event);
            };

            // we locked the seller, thus the buyer might have updated the sell price:
            // check if the operation is still good for us
            if let Ok(new_offer) = buyer.borrow().get_sell_price(kind, quantity) {
                if new_offer > offer && new_offer <= bid {
                    println!("Aborting: the new offer ({new_offer}) would not produce a gain: it exceeds the bid ({bid})");
                    markets.wait_one_day();
                    return;
                } else {
                    // (new_offer > offer && new_offer > bid), or (new_offer <= offer)
                    // in either case, it produces a gain
                    offer = new_offer;
                }
            } else {
                markets.wait_one_day();
                return;
            }

            if let Ok(seller_token) = result {
                let result = buyer.borrow_mut().lock_sell(kind, quantity, offer, DOGE_TRADER_NAME.to_owned());

                for handler in &self.handlers {
                    let event = TradingEvent {
                        account: account.clone(),
                        market: buyer.borrow().get_name().to_owned(),
                        event: LockSell { kind, quantity, offer, result: result.clone() },
                    };
                    handler.handle_event(event);
                }

                if let Ok(buyer_token) = result {
                    let mut cash = account.withdraw(DEFAULT_GOOD_KIND, bid).unwrap();

                    let result = seller.borrow_mut().buy(seller_token.clone(), &mut cash);

                    for handler in &self.handlers {
                        let event = TradingEvent {
                            account: account.clone(),
                            market: seller.borrow().get_name().to_owned(),
                            event: Buy { token: seller_token.clone(), kind, quantity, bid, result: result.clone() },
                        };
                        handler.handle_event(event);
                    }

                    if let Ok(mut bought) = result {
                        let result = buyer.borrow_mut().sell(buyer_token.clone(), &mut bought);

                        if let Ok(bought) = result.clone() {
                            account.deposit(bought);
                        }

                        for handler in &self.handlers {
                            let event = TradingEvent {
                                account: account.clone(),
                                market: buyer.borrow().get_name().to_owned(),
                                event: Sell { token: buyer_token.clone(), kind, quantity, result: result.clone(), offer },
                            };
                            handler.handle_event(event);
                        }
                    } else {
                        account.deposit(cash);
                    }
                }
            }
        }
        markets.wait_one_day();
    }

    fn add_event_handler(&mut self, handler: Box<dyn EventHandler>) {
        self.handlers.push(handler);
    }

    fn get_name(&self) -> &'static str {
        "Min-Max Strategy"
    }
}

fn find_exchange(markets: &MarketWrapper, account: &Account) -> Option<Exchange> {
    [USD, YEN, YUAN].into_iter()
        .cartesian_product(make_combinations(markets))
        .max_by(|(kind1, (seller1, buyer1)), (kind2, (seller2, buyer2))| {
            let compute_difference = |kind: GoodKind, seller: &Ref<dyn Market>, buyer: &Ref<dyn Market>| {
                let buy_price = seller.get_buy_price(kind, 1.).unwrap_or(f32::INFINITY);
                let sell_price = buyer.get_sell_price(kind, 1.).unwrap();
                sell_price - buy_price // gain
            };

            let diff1 = compute_difference(*kind1, &seller1.borrow(), &buyer1.borrow());
            let diff2 = compute_difference(*kind2, &seller2.borrow(), &buyer2.borrow());

            diff1.total_cmp(&diff2) // sort from max to min
        })
        .map(|(kind, (seller, buyer))| {
            let quantity = compute_exchange_quantity(kind, seller.borrow(), buyer.borrow(), account);
            let bid = seller.borrow().get_buy_price(kind, quantity).unwrap();
            let offer = buyer.borrow().get_sell_price(kind, quantity).unwrap();
            Exchange { kind, seller, bid, buyer, offer, quantity }
        })
        .filter(|exchange| exchange.bid >= 0. && exchange.offer >= 0.)
        .filter(|exchange| exchange.offer > exchange.bid)
}

fn compute_exchange_quantity(kind: GoodKind, seller: Ref<dyn Market>, buyer: Ref<dyn Market>, account: &Account) -> f32 {
    //avoid more than 0.05
    const TRADE_PERCENTAGE: f32 = 0.005;

    const QUANTITY_DECREASE_FACTOR: f32 = 1.2;

    let from_seller_quantity = seller.get_goods().into_iter()
        .find(|label| label.good_kind == kind)
        .map(|label| label.quantity)
        .unwrap();
    let mut quantity_to_exchange = from_seller_quantity * TRADE_PERCENTAGE;

    //make sure trader has enough euro to buy goods from seller
    loop {
        let seller_price = seller.get_buy_price(kind, quantity_to_exchange).unwrap();
        if seller_price < account.get_quantity_by_kind(EUR) {
            break;
        } else {
            quantity_to_exchange /= QUANTITY_DECREASE_FACTOR;
        }
    }
    //make sure buyer has enough euro to buy from us
    loop {
        let buyer_price = buyer.get_sell_price(kind, quantity_to_exchange).unwrap();
        let buyer_budget = buyer.get_budget();
        if buyer_price < buyer_budget {
            break;
        } else {
            quantity_to_exchange /= QUANTITY_DECREASE_FACTOR;
        }
    }

    quantity_to_exchange
}

fn make_combinations(markets: &MarketWrapper) -> impl Iterator<Item=(Rc<RefCell<dyn Market>>, Rc<RefCell<dyn Market>>)> + Clone {
    let combinations = markets.iter().tuple_combinations::<(Rc<RefCell<dyn Market>>, Rc<RefCell<dyn Market>>)>();
    let opposite = combinations.clone().map(|(first, second)| (second, first));
    combinations.chain(opposite)
}


#[test]
fn test_combinations() {
    let markets = MarketWrapper::new();
    let pairs = make_combinations(&markets);
    pairs.for_each(|(buyer, seller)|
        println!("{} {}", buyer.borrow().get_name(), seller.borrow().get_name()))
    // BOSE RCNZ
    // BOSE SOL
    // RCNZ SOL
    // RCNZ BOSE
    // SOL BOSE
    // SOL RCNZ
}
