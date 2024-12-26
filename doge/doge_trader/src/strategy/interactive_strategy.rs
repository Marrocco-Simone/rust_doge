use std::cell::RefMut;
use std::io;

use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::good::good_kind::GoodKind::{EUR, USD, YEN, YUAN};
use unitn_market_2022::market::Market;

use doge_common::account::Account;

use crate::DOGE_TRADER_NAME;
use crate::event::TradingEvent;
use crate::event::TradingEventType::{Buy, LockBuy, LockSell, Sell};
use crate::event_handler::EventHandler;
use crate::logger::noop_logger::NoOpLogger;
use crate::logger::stdout_logger::StdoutLogger;
use crate::markets::MarketWrapper;
use crate::strategy::Strategy;

pub struct InteractiveStrategy {
    handlers: Vec<Box<dyn EventHandler>>,
}

impl InteractiveStrategy {
    pub fn new() -> InteractiveStrategy {
        InteractiveStrategy { handlers: vec![] }
    }
}

impl Strategy for InteractiveStrategy {
    fn apply(&mut self, account: &mut Account, markets: &mut MarketWrapper) {
        println!("Account contains: {account}");

        println!("SOL has the following goods:");
        for gl in markets.sol.borrow().get_goods() {
            println!("\t{} {:.2}, Buy exchange rate: {:.2}, Sell exchange rate: {:.2}", gl.good_kind, gl.quantity, gl.exchange_rate_buy, gl.exchange_rate_sell);
        }

        println!("RCNZ has the following goods:");
        for gl in markets.rcnz.borrow().get_goods() {
            println!("\t{} {:.2}, Buy exchange rate: {:.2}, Sell exchange rate: {:.2}", gl.good_kind, gl.quantity, gl.exchange_rate_buy, gl.exchange_rate_sell);
        }

        println!("BOSE has the following goods:");
        for gl in markets.rcnz.borrow().get_goods() {
            println!("\t{} {:.2}, Buy exchange rate: {:.2}, Sell exchange rate: {:.2}", gl.good_kind, gl.quantity, gl.exchange_rate_buy, gl.exchange_rate_sell);
        }

        let interaction = select_market(markets)
            .map(|market| (market, select_operation()))
            .map(|(market, op)| (market, op));

        match interaction {
            Some((mut market, Op::Buy { kind, quantity, bid })) => {
                println!("Performing buy operation...");

                match account.withdraw(EUR, bid) {
                    Ok(mut cash) => {
                        let result = market.lock_buy(kind, quantity, bid, DOGE_TRADER_NAME.to_owned());

                        for handler in &mut self.handlers {
                            let event = TradingEvent {
                                account: account.clone(),
                                market: market.get_name().to_owned(),
                                event: LockBuy { kind, quantity, bid, result: result.clone() },
                            };
                            handler.handle_event(event);
                        };

                        if let Ok(token) = result {
                            let result = market.buy(token.clone(), &mut cash);

                            if let Ok(bought) = result.clone() {
                                account.deposit(bought);
                            } else {
                                account.deposit(cash);
                            }

                            for handler in &mut self.handlers {
                                let event = TradingEvent {
                                    account: account.clone(),
                                    market: market.get_name().to_owned(),
                                    event: Buy { token: token.clone(), kind, quantity, bid, result: result.clone() },
                                };
                                handler.handle_event(event);
                            };
                        } else {
                            account.deposit(cash);
                        }
                    }
                    Err(err) => println!("Account withdraw error: {err:?}"),
                }
            }
            Some((mut market, Op::Sell { kind, quantity, offer })) => {
                println!("Performing sell operation...");

                match account.withdraw(kind, quantity) {
                    Ok(mut sold) => {
                        let result = market.lock_sell(kind, quantity, offer, DOGE_TRADER_NAME.to_owned());

                        for handler in &mut self.handlers {
                            let event = TradingEvent {
                                account: account.clone(),
                                market: market.get_name().to_owned(),
                                event: LockSell { kind, quantity, offer, result: result.clone() },
                            };
                            handler.handle_event(event);
                        };

                        if let Ok(token) = result {
                            let result = market.sell(token.clone(), &mut sold);

                            if let Ok(cash) = result.clone() {
                                account.deposit(cash);
                            } else {
                                account.deposit(sold);
                            }

                            for handler in &mut self.handlers {
                                let event = TradingEvent {
                                    account: account.clone(),
                                    market: market.get_name().to_owned(),
                                    event: Sell { token: token.clone(), kind, quantity, offer, result: result.clone() },
                                };
                                handler.handle_event(event);
                            };
                        } else {
                            account.deposit(sold);
                        }
                    }
                    Err(err) => println!("Account withdraw error: {err:?}"),
                }
            }
            None => return,
        }
    }

    fn add_event_handler(&mut self, handler: Box<dyn EventHandler>) {
        self.handlers.push(handler);
    }

    fn get_name(&self) -> &'static str {
        "Interactive strategy"
    }
}

enum Op {
    Buy { kind: GoodKind, quantity: f32, bid: f32 },
    Sell { kind: GoodKind, quantity: f32, offer: f32 },
}

fn select_market(markets: &mut MarketWrapper) -> Option<RefMut<dyn Market>> {
    loop {
        println!("Select a market for an operation:");
        println!("0: No operation");
        println!("1: SOL");
        println!("2: RCNZ");
        println!("3: BOSE");

        let mut input = String::new();

        io::stdin().read_line(&mut input).unwrap();

        break match input.trim().parse() {
            Ok(0) => None,
            Ok(1) => Some(markets.sol.borrow_mut()),
            Ok(2) => Some(markets.rcnz.borrow_mut()),
            Ok(3) => Some(markets.bose.borrow_mut()),
            _ => continue,
        };
    }
}

fn select_operation() -> Op {
    loop {
        println!("Select the operation:");
        println!("1: Buy");
        println!("2: Sell");

        let mut input = String::new();

        io::stdin().read_line(&mut input).unwrap();

        break match input.trim().parse() {
            Ok(1) => Op::Buy {
                kind: select_good_kind(),
                quantity: select_f32("Specify the quantity to buy:"),
                bid: select_f32("Specify the bid:"),
            },
            Ok(2) => Op::Sell {
                kind: select_good_kind(),
                quantity: select_f32("Specify the quantity to sell:"),
                offer: select_f32("Specify the offer:"),
            },
            _ => continue,
        };
    }
}

fn select_good_kind() -> GoodKind {
    loop {
        println!("Select a GoodKind:");
        println!("1: EUR");
        println!("2: USD");
        println!("3: YEN");
        println!("4: YUAN");

        let mut input = String::new();

        io::stdin().read_line(&mut input).unwrap();

        break match input.trim().parse() {
            Ok(1) => EUR,
            Ok(2) => USD,
            Ok(3) => YEN,
            Ok(4) => YUAN,
            _ => continue,
        };
    }
}

fn select_f32(query: &'static str) -> f32 {
    loop {
        println!("{query}");

        let mut input = String::new();

        io::stdin().read_line(&mut input).unwrap();

        if let Ok(quantity) = input.trim().parse::<f32>() {
            if quantity > 0. {
                return quantity;
            }
        }
    }
}
