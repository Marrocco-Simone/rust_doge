use clap::{Parser, ValueEnum};

#[derive(Parser)]
pub struct Args {
    /// Number of trading iterations to simulate
    #[arg(long, default_value_t = 100)]
    pub n_iterations: u32,

    /// Seconds of pause between each iteration
    #[arg(long, default_value_t = 0.5)]
    pub pause_s: f32,

    /// Whether the trader should be interactive (e.g., allows to change strategy)
    #[arg(long, default_value_t = false)]
    pub interactive: bool,

    /// If set, it does not produce the charts for the trader and the markets' accounts.
    #[arg(long, default_value_t = false)]
    pub suppress_charts: bool,

    /// Initial EUR amount in the trader's account
    #[arg(long, default_value_t = 100_000.)]
    pub eur: f32,

    /// Initial USD amount in the trader's account
    #[arg(long, default_value_t = 100_000.)]
    pub usd: f32,

    /// Initial YEN amount in the trader's account
    #[arg(long, default_value_t = 100_000.)]
    pub yen: f32,

    /// Initial YUAN amount in the trader's account
    #[arg(long, default_value_t = 100_000.)]
    pub yuan: f32,

    /// Trading strategy that is applied to the trader at start
    #[arg(long, default_value_t = ArgStrategy::RandomAction, value_enum)]
    pub strategy: ArgStrategy,

    /// Logger that is applied to the strategy at start
    #[arg(long, default_value_t = ArgLogger::Stdout, value_enum)]
    pub logger: ArgLogger,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ArgStrategy {
    NoOp,
    Interactive,
    RandomAction,
    MinMax,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ArgLogger {
    NoOp,
    Stdout,
    TxtFile,
    Svg,
}