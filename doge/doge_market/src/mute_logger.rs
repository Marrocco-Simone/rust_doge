use unitn_market_2022::good::good::Good;
use uuid::Uuid;

use crate::logger::Logger;

/// Logger that does not log anything.
pub struct MuteLogger {}

impl MuteLogger {
    pub fn new() -> MuteLogger {
        MuteLogger {}
    }
}

impl Logger for MuteLogger {
    fn log_initialization(&mut self, _eur: &Good, _usd: &Good, _yen: &Good, _yuan: &Good) {}

    fn log_lock_buy(&mut self, _locked: &Good, _trader_name: &str, _bid: &Good, _token: Option<&Uuid>) {}

    fn log_lock_sell(&mut self, _locked: &Good, _trader_name: &str, _offer: &Good, _token: Option<&Uuid>) {}

    fn log_buy(&mut self, _token: &Uuid, _success: bool) {}

    fn log_sell(&mut self, _token: &Uuid, _success: bool) {}
}