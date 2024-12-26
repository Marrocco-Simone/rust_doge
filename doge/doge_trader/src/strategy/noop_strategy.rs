use doge_common::account::Account;

use crate::event_handler::EventHandler;
use crate::markets::MarketWrapper;
use crate::strategy::Strategy;

pub struct NoOpStrategy;

impl NoOpStrategy {
    pub fn new() -> NoOpStrategy {
        NoOpStrategy {}
    }
}

impl Strategy for NoOpStrategy {
    fn apply(&mut self, _: &mut Account, wrapper: &mut MarketWrapper) {
        wrapper.wait_one_day();
    }

    fn add_event_handler(&mut self, _: Box<dyn EventHandler>) {}

    fn get_name(&self) -> &'static str {
        "No-op strategy"
    }
}