use doge_common::account::Account;

use crate::event_handler::EventHandler;
use crate::markets::MarketWrapper;

mod common;
pub mod interactive_strategy;
pub mod min_max_strategy;
pub mod noop_strategy;
pub mod price_comparison_strategy;
pub mod random_action_strategy;
pub mod single_market_random_strategy;

pub trait Strategy {
    fn apply(&mut self, account: &mut Account, markets: &mut MarketWrapper);
    fn add_event_handler(&mut self, handler: Box<dyn EventHandler>);
    fn get_name(&self) -> &'static str;
}
