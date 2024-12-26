use crate::event::TradingEvent;

pub trait EventHandler {
    fn handle_event(&self, event: TradingEvent);
    fn get_name(&self) -> &'static str;
}
