use std::cell::RefCell;
use std::rc::Rc;
use std::vec;

use rand::{Rng, thread_rng};
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
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

type SelectionStrategy = Box<dyn FnMut(&MarketWrapper) -> Rc<RefCell<dyn Market>>>;

pub struct RandomActionStrategy {
    handlers: Vec<Box<dyn EventHandler>>,
    rng: ThreadRng,
    pending: Vec<Tx>,
    selection_strategy: SelectionStrategy,
}

const MIN_LIMIT: f32 = 5.;

struct Tx {
    token: String,
    market: Rc<RefCell<dyn Market>>,
    tx: TxType,
}

enum TxType {
    Buy { kind: GoodKind, quantity: f32, bid: f32 },
    Sell { kind: GoodKind, quantity: f32, offer: f32 },
}

enum ActionToApply {
    WaitOneDay,
    LockBuy { market: Rc<RefCell<dyn Market>>, kind: GoodKind, percentage: f32 },
    LockSell { market: Rc<RefCell<dyn Market>>, kind: GoodKind, percentage: f32 },
    CompleteTx(Tx),
}

pub fn random_selection(markets: &MarketWrapper) -> Rc<RefCell<dyn Market>> {
    match thread_rng().gen_range(0..=2) {
        0 => Rc::clone(&markets.sol),
        1 => Rc::clone(&markets.rcnz),
        2 => Rc::clone(&markets.bose),
        _ => unreachable!()
    }
}

impl RandomActionStrategy {
    pub fn new() -> RandomActionStrategy {
        RandomActionStrategy {
            handlers: vec![],
            rng: thread_rng(),
            pending: vec![],
            selection_strategy: Box::new(random_selection),
        }
    }

    pub fn set_selection_strategy(&mut self, selection_strategy: SelectionStrategy) {
        println!("Change selection strategy");
        self.selection_strategy = selection_strategy;
    }

    /// If the trader has no ongoing transactions, then it chooses randomly to wait one day, to open a buy transaction or to open a sell transaction.
    /// Otherwise, it chooses randomly one to complete, to open a new one or wait.
    fn generate_random_action(&mut self, markets: &MarketWrapper) -> ActionToApply {
        if self.pending.is_empty() {
            if self.rng.gen_bool(0.5) {
                ActionToApply::WaitOneDay
            } else {
                self.create_random_lock_action(markets)
            }
        } else { // Trader has pending transactions...
            if self.rng.gen_bool(0.333) {
                ActionToApply::WaitOneDay
            } else if self.rng.gen_bool(0.5) {
                self.create_random_lock_action(markets)
            } else { // ...Than can be unwrapped
                let index = self.rng.gen_range(0..=self.pending.len() - 1);
                let tx = self.pending.remove(index);
                ActionToApply::CompleteTx(tx)
            }
        }
    }

    fn create_random_lock_action(&mut self, markets: &MarketWrapper) -> ActionToApply {
        return if self.rng.gen_bool(0.5) {
            ActionToApply::LockBuy {
                market: (self.selection_strategy)(markets),
                kind: *[USD, YEN, YUAN].choose(&mut self.rng).unwrap(),
                // percentage is one third wrt lock sell percentage
                // because EUR is 3 times more likely to be traded
                percentage: self.rng.gen_range((10f32 / 3.)..=30.0) / 100.,
            }
        } else {
            ActionToApply::LockSell {
                market: (self.selection_strategy)(markets),
                kind: *[USD, YEN, YUAN].choose(&mut self.rng).unwrap(),
                percentage: self.rng.gen_range(10.0..=90.0) / 100.,
            }
        };
    }
}

impl Strategy for RandomActionStrategy {
    fn apply(&mut self, account: &mut Account, markets: &mut MarketWrapper) {
        match self.generate_random_action(markets) {
            ActionToApply::WaitOneDay => {
                markets.wait_one_day();
            }
            ActionToApply::LockBuy { market, kind, percentage } => {
                // Since there is no way to know with certainty the amount of good
                // of a given kind that the market is willing to trade for a given
                // amount of money, we need to first do an estimation from the
                // exchange rate and then try to buy the estimated amount, checking
                // if it is in the range of prices we are willing to pay
                let minimum_bid = f32::max(account.get_quantity_by_kind(EUR) * (percentage - 0.01), MIN_LIMIT);
                let maximum_bid = f32::max(account.get_quantity_by_kind(EUR) * (percentage + 0.01), MIN_LIMIT);
                // this will happen if the trader does not currently have enough EUR to trade
                if maximum_bid == minimum_bid {
                    markets.wait_one_day();
                    return;
                }

                let exchange_rate_buy = market.borrow().get_goods()
                    .into_iter()
                    .filter(|label| label.good_kind == kind)
                    .map(|label| label.exchange_rate_buy)
                    .next()
                    .unwrap();
                let quantity_to_buy = minimum_bid * exchange_rate_buy;

                /*
                // This does not work because markets.wait_one_day() borrows mutably
                // market which has been borrowed to get the buy price (run time
                // BorrowMutError) so I had to go with a less elegant solution
                let bid = if let Ok(price) = market.borrow().get_buy_price(kind,quantity_to_buy){
                    price
                }else{
                    markets.wait_one_day();
                    return;
                };*/

                let buy_price_result = market.borrow().get_buy_price(kind, quantity_to_buy);
                let bid = if let Ok(price) = buy_price_result {
                    price
                } else {
                    markets.wait_one_day();
                    return;
                };

                if bid > maximum_bid {
                    markets.wait_one_day();
                    return;
                }

                let result = market.borrow_mut().lock_buy(kind, quantity_to_buy, bid, DOGE_TRADER_NAME.to_owned());

                for handler in &mut self.handlers {
                    let event = TradingEvent {
                        account: account.clone(),
                        market: market.borrow().get_name().to_owned(),
                        event: LockBuy { kind, quantity: quantity_to_buy, bid, result: result.clone() },
                    };
                    handler.handle_event(event);
                };

                if let Ok(token) = result {
                    let tx = Tx {
                        token,
                        market: Rc::clone(&market),
                        tx: TxType::Buy { kind, quantity: quantity_to_buy, bid },
                    };

                    self.pending.push(tx)
                }
            }
            ActionToApply::LockSell { market, kind, percentage } => {
                let quantity_to_sell = account.get_quantity_by_kind(kind) * percentage;

                if account.get_quantity_by_kind(kind) - quantity_to_sell <= MIN_LIMIT {
                    markets.wait_one_day();
                    return;
                }

                let highest_offer = if let Ok(price) = market.borrow().get_sell_price(kind, quantity_to_sell) {
                    price
                } else {
                    markets.wait_one_day();
                    return;
                };

                let result = market.borrow_mut().lock_sell(kind, quantity_to_sell, highest_offer, DOGE_TRADER_NAME.to_owned());

                for handler in &mut self.handlers {
                    let event = TradingEvent {
                        account: account.clone(),
                        market: market.borrow().get_name().to_owned(),
                        event: LockSell { kind, quantity: quantity_to_sell, offer: highest_offer, result: result.clone() },
                    };
                    handler.handle_event(event);
                };

                if let Ok(token) = result {
                    let tx = Tx {
                        token,
                        market,
                        tx: TxType::Sell { kind, quantity: quantity_to_sell, offer: highest_offer },
                    };

                    self.pending.push(tx);
                };
            }
            ActionToApply::CompleteTx(tx) => {
                match tx.tx {
                    TxType::Buy { kind, quantity, bid } => {
                        if let Ok(mut cash) = account.withdraw(EUR, bid) {
                            let result = tx.market.borrow_mut().buy(tx.token.clone(), &mut cash);

                            if let Ok(bought) = result.clone() {
                                account.deposit(bought);
                            } else {
                                account.deposit(cash);
                            }

                            for handler in &mut self.handlers {
                                let event = TradingEvent {
                                    account: account.clone(),
                                    market: tx.market.borrow().get_name().to_owned(),
                                    event: Buy { token: tx.token.clone(), kind, quantity, bid, result: result.clone() },
                                };
                                handler.handle_event(event);
                            };
                        } else {
                            // The completion order of transactions is random, thus sometimes
                            // they can fail because the account does not have enough good quanity.
                            // In such case, transactions are rescheduled.
                            self.pending.push(tx);
                        }
                    }
                    TxType::Sell { kind, quantity, offer } => {
                        if let Ok(mut sold) = account.withdraw(kind, quantity) {
                            let result = tx.market.borrow_mut().sell(tx.token.clone(), &mut sold);

                            if let Ok(cash) = result.clone() {
                                account.deposit(cash);
                            } else {
                                account.deposit(sold);
                            }

                            for handler in &mut self.handlers {
                                let event = TradingEvent {
                                    account: account.clone(),
                                    market: tx.market.borrow().get_name().to_owned(),
                                    event: Sell { token: tx.token.clone(), kind, quantity, offer, result: result.clone() },
                                };
                                handler.handle_event(event)
                            };
                        } else {
                            // The completion order of transactions is random, thus sometimes
                            // they can fail because the account does not have enough good quanity.
                            // In such case, transactions are rescheduled.
                            self.pending.push(tx);
                        }
                    }
                }
            }
        }
    }

    fn add_event_handler(&mut self, handler: Box<dyn EventHandler>) {
        self.handlers.push(handler);
    }

    fn get_name(&self) -> &'static str {
        "Random selection strategy"
    }
}