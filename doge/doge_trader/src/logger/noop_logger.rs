use crate::event::TradingEvent;
use crate::event_handler::EventHandler;

pub struct NoOpLogger;

impl NoOpLogger {
    pub fn new() -> NoOpLogger {
        NoOpLogger {}
    }
}

impl EventHandler for NoOpLogger {
    fn handle_event(&self, _: TradingEvent) {}

    fn get_name(&self) -> &'static str {
        "No-op logger"
    }
}