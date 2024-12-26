use doge_common::account::Account;

pub mod visualizer;

pub struct HistoryPoint {
    pub trader: Account,
    pub sol: Account,
    pub rcnz: Account,
    pub bose: Account,
}
