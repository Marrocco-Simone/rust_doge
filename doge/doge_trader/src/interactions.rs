use std::fs::File;
use std::io;

use crate::displayer::svg_displayer::SvgDisplayer;
use crate::event_handler::EventHandler;
use crate::logger::noop_logger::NoOpLogger;
use crate::logger::stdout_logger::StdoutLogger;
use crate::logger::txt_file_logger::TxtFileLogger;
use crate::strategy::interactive_strategy::InteractiveStrategy;
use crate::strategy::min_max_strategy::MinMaxStrategy;
use crate::strategy::noop_strategy::NoOpStrategy;
use crate::strategy::random_action_strategy::RandomActionStrategy;
use crate::strategy::Strategy;

pub fn select_event_handler() -> Box<dyn EventHandler> {
    loop {
        println!("Select an event handler:");
        println!("1: No-op logger");
        println!("2: Stdout logger");
        println!("3: Txt file logger");
        println!("4: SVG displayer");

        let mut input = String::new();

        io::stdin().read_line(&mut input).unwrap();

        break match input.trim().parse() {
            Ok(1) => Box::new(NoOpLogger::new()),
            Ok(2) => Box::new(StdoutLogger::new()),
            Ok(3) => Box::new(TxtFileLogger::new(File::create("simulation.txt").unwrap())),
            Ok(4) => Box::new(SvgDisplayer::new()),
            _ => continue,
        };
    }
}

pub fn select_strategy() -> Box<dyn Strategy> {
    loop {
        println!("Select a trading strategy:");
        println!("1: No-op strategy");
        println!("2: Interactive strategy");
        println!("3: Random action strategy");
        println!("4: Min-Max strategy");

        let mut input = String::new();

        io::stdin().read_line(&mut input).unwrap();

        break match input.trim().parse() {
            Ok(1) => Box::new(NoOpStrategy::new()),
            Ok(2) => Box::new(InteractiveStrategy::new()),
            Ok(3) => Box::new(RandomActionStrategy::new()),
            Ok(4) => Box::new(MinMaxStrategy::new()),
            _ => continue,
        };
    }
}

pub struct LoopResult(pub Box<dyn Strategy>);

pub fn do_main_loop(current_strategy: Box<dyn Strategy>) -> LoopResult {
    let mut new_strategy;

    'strategy: loop {
        println!("Change the trader strategy? (Current: '{}') (y/N)", current_strategy.get_name());

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        match input.trim().parse().ok() {
            Some('y' | 'Y') => {
                new_strategy = select_strategy();

                'handler: loop {
                    println!("Add an event handler to it? (y/N)");

                    input.clear();
                    io::stdin().read_line(&mut input).unwrap();

                    match input.trim().parse().ok() {
                        Some('y' | 'Y') => {
                            let handler = select_event_handler();
                            new_strategy.add_event_handler(handler);

                            break 'strategy;
                        }
                        Some('n' | 'N') | None => break 'strategy,
                        _ => continue 'handler,
                    };
                }
            }
            Some('n' | 'N') | None => {
                new_strategy = current_strategy;

                break 'strategy;
            }
            _ => continue 'strategy,
        };
    }

    LoopResult(new_strategy)
}
