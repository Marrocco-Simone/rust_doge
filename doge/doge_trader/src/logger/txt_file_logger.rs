use std::cell::RefCell;
use std::fs::File;
use std::io::Write;

use crate::event::TradingEvent;
use crate::event_handler::EventHandler;

pub struct TxtFileLogger {
    file: RefCell<File>,
}

impl TxtFileLogger {
    pub fn new(file: File) -> TxtFileLogger {
        TxtFileLogger { file: RefCell::new(file) }
    }
}

impl EventHandler for TxtFileLogger {
    fn handle_event(&self, event: TradingEvent) {
        writeln!(self.file.borrow_mut(), "{event}").unwrap();
    }

    fn get_name(&self) -> &'static str {
        "Txt file logger"
    }
}