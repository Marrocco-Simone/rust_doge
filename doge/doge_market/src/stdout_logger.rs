use unitn_market_2022::good::good::Good;
use uuid::Uuid;

use crate::logger::Logger;

/// Logger that logs to standard output.
pub struct StdoutLogger {}

impl StdoutLogger {
    pub fn new() -> StdoutLogger {
        StdoutLogger {}
    }
}

impl Logger for StdoutLogger {
    fn log_initialization(&mut self, eur: &Good, usd: &Good, jpy: &Good, cny: &Good) {
        println!("INITIALIZED MARKET:\n\
                EUR: {}\n\
                USD: {}\n\
                YEN: {}\n\
                YUAN: {}\n",
                 eur.get_qty(),
                 usd.get_qty(),
                 jpy.get_qty(),
                 cny.get_qty());
    }

    fn log_lock_buy(&mut self, locked: &Good, trader_name: &str, bid: &Good, token: Option<&Uuid>) {
        match token {
            Some(token) => {
                println!("{trader_name} LOCK_BUY {locked} WITH BID {bid} AND TOKEN {token}");
            }
            None => {
                println!("{trader_name} FAILED LOCK_BUY {locked} WITH BID {bid}");
            }
        }
    }

    fn log_lock_sell(&mut self, locked: &Good, trader_name: &str, offer: &Good, token: Option<&Uuid>) {
        match token {
            Some(token) => {
                println!("{trader_name} LOCK_SELL {locked} WITH OFFER {offer} AND TOKEN {token}");
            }
            None => {
                println!("{trader_name} FAILED LOCK_SELL {locked} WITH OFFER {offer}");
            }
        }
    }

    fn log_buy(&mut self, token: &Uuid, success: bool) {
        match success {
            true => {
                println!("SUCCESSFUL BUY WITH TOKEN {token}")
            }
            false => {
                println!("FAILED BUY WITH TOKEN {token}")
            }
        }
    }

    fn log_sell(&mut self, token: &Uuid, success: bool) {
        match success {
            true => {
                println!("SUCCESSFUL SELL WITH TOKEN {token}")
            }
            false => {
                println!("FAILED SELL WITH TOKEN {token}")
            }
        }
    }
}