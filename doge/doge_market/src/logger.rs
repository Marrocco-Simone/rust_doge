use unitn_market_2022::good::good::Good;
use uuid::Uuid;

pub trait Logger {
    fn log_initialization(&mut self, eur: &Good, usd: &Good, jpy: &Good, cny: &Good);
    fn log_lock_buy(&mut self, locked: &Good, trader_name: &str, bid: &Good, token: Option<&Uuid>);
    fn log_lock_sell(&mut self, locked: &Good, trader_name: &str, offer: &Good, token: Option<&Uuid>);
    fn log_buy(&mut self, token: &Uuid, success: bool);
    fn log_sell(&mut self, token: &Uuid, success: bool);
}
