extern crate core;

use std::io;
use std::thread::sleep;
use std::time::Duration;

use clap::Parser;
use doge_trader::iteraction_logger::iteraction_logger::IteractionLogger;
use doge_trader::logger::stdout_logger::StdoutLogger;
use doge_trader::strategy::price_comparison_strategy::PriceComparison;
use doge_trader::strategy::random_action_strategy::RandomActionStrategy;
use doge_trader::strategy::single_market_random_strategy::{BoseRandom, RcnzRandom, SolRandom};
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind::{EUR, USD, YEN, YUAN};

use doge_common::account::Account;
use doge_trader::markets::MarketWrapper;
use doge_trader::strategy::interactive_strategy::InteractiveStrategy;
use doge_trader::strategy::min_max_strategy::MinMaxStrategy;
use doge_trader::strategy::noop_strategy::NoOpStrategy;
use doge_trader::strategy::Strategy;

// ! remember to update select_strategy()
const MODE_CHOICE_DOCS: &str = "1: No-op strategy
2: Interactive strategy
3: Random selection strategy
4: Min-Max strategy
5: SOL Random strategy
6: RCNZ Random strategy
7: BOSE Random strategy
8: Price Comparison strategy";

#[derive(Parser, Debug)]
struct Args {
    /// Number of trading iterations to simulate
    #[arg(short, long, default_value_t = 100)]
    n_iterations: u32,

    // ! this comment should be the same as MODE_CHOICE_DOCS
    /**1: No-op strategy
    2: Interactive strategy
    3: Random selection strategy
    4: Min-Max strategy
    5: SOL Random strategy
    6: RCNZ Random strategy
    7: BOSE Random strategy
    8: Price Comparison strategy*/
    #[arg(short, long, verbatim_doc_comment, default_value_t = 0)]
    mode_choice: u32,

    /// Seconds of pause between each iteration
    #[arg(short, long, default_value_t = 0.5)]
    pause_s: f32,

    /// If you don't want the transaction logs in the console
    #[arg(short, long, default_value_t = false)]
    log_hide: bool,

    /// If you don't want to create the graphs at the end of the simulation
    #[arg(short, long, default_value_t = false)]
    graph_hide: bool,

    /// Whether the trader should be interactive (e.g., allows to change strategy)
    #[arg(short, long, default_value_t = false)]
    interactive: bool,

    /// Initial EUR amount in the trader's account
    #[arg(long, default_value_t = 100_000.)]
    eur: f32,

    /// Initial USD amount in the trader's account
    #[arg(long, default_value_t = 100_000.)]
    usd: f32,

    /// Initial YEN amount in the trader's account
    #[arg(long, default_value_t = 100_000.)]
    yen: f32,

    /// Initial YUAN amount in the trader's account
    #[arg(long, default_value_t = 100_000.)]
    yuan: f32,
}

fn select_strategy(mut mode_choice: u32) -> Box<dyn Strategy> {
    loop {
        // ! remember to update MODE_CHOICE_DOCS
        match mode_choice {
            1 => return Box::new(NoOpStrategy::new()),
            2 => return Box::new(InteractiveStrategy::new()),
            3 => return Box::new(RandomActionStrategy::new()),

            4 => return Box::new(MinMaxStrategy::new()),
            5 => return Box::new(SolRandom::new()),
            6 => return Box::new(RcnzRandom::new()),
            7 => return Box::new(BoseRandom::new()),
            8 => return Box::new(PriceComparison::new()),
            _ => (),
        }

        println!("Select a trading strategy:\n{}", MODE_CHOICE_DOCS);

        let mut input = String::new();

        io::stdin().read_line(&mut input).unwrap();

        match input.trim().parse() {
            Ok(m) => mode_choice = m,
            _ => mode_choice = 0,
        };
    }
}

fn main() {
    let args = Args::parse();

    let mut account = Account {
        eur: Good::new(EUR, args.eur),
        usd: Good::new(USD, args.usd),
        yen: Good::new(YEN, args.yen),
        yuan: Good::new(YUAN, args.yuan),
    };

    let mut markets = MarketWrapper::new();

    let mut strategy: Box<dyn Strategy> = select_strategy(args.mode_choice);
    let mut iteraction_logger = IteractionLogger::new();

    println!("Running the trader for {} iterations...", args.n_iterations);

    for it in 0..args.n_iterations {
        println!("Iteration {it}");

        if args.interactive {
            loop {
                println!(
                    "Change the trader strategy? (Current: {}) (y/N)",
                    strategy.get_name()
                );

                let mut input = String::new();

                io::stdin().read_line(&mut input).unwrap();

                break match input.trim().parse().ok() {
                    Some('y' | 'Y') => strategy = select_strategy(0),
                    Some('n' | 'N') | None => break,
                    _ => continue,
                };
            }
            println!("Applying {}", strategy.get_name());
        }
        strategy.apply(&mut account, &mut markets);

        if !args.log_hide || !args.graph_hide {
            iteraction_logger.update_from_market_wrapper(&account, &markets);
        }
        if !args.log_hide {
            iteraction_logger.log_last_difference();
        }

        sleep(Duration::from_secs_f32(args.pause_s));
    }

    if !args.log_hide {
        iteraction_logger.log_final_difference();
    }
    if !args.graph_hide {
        iteraction_logger.generate_svgs();
    }
}
