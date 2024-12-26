use doge_common::account::Account;

use crate::markets::MarketWrapper;

use super::create_account_graphs::create_account_graphs;

pub struct IteractionLogger {
    pub trader: Vec<Account>,
    pub sol: Vec<Account>,
    pub rcnz: Vec<Account>,
    pub bose: Vec<Account>,
}

fn get_last_item<T>(vec: &Vec<T>, offset: usize) -> Option<&T> {
    if vec.len() < 1 + offset {
        return None;
    }
    return vec.get(vec.len() - 1 - offset);
}

fn get_account_gain_string(vec: &Vec<Account>) -> String {
    let current_account = get_last_item(&vec, 0);
    let previous_account = get_last_item(&vec, 1);

    if current_account.is_none() || previous_account.is_none() {
        return String::new();
    }

    let current_value = current_account.unwrap().get_total_account_value_in_eur();
    let previous_value = previous_account.unwrap().get_total_account_value_in_eur();

    let diff = current_value - previous_value;
    let percenteage = diff / previous_value * 100.;

    if diff < 0.000001 {
        return String::new();
    }

    return format!("(gain: {:.2} = {:.2}%)", diff, percenteage);
}

fn get_final_account_gain_string(vec: &Vec<Account>) -> String {
    let current_account = get_last_item(&vec, 0);
    let previous_account = get_last_item(&vec, vec.len() - 1);

    if current_account.is_none() || previous_account.is_none() {
        return String::new();
    }

    let current_value = current_account.unwrap().get_total_account_value_in_eur();
    let previous_value = previous_account.unwrap().get_total_account_value_in_eur();

    let diff = current_value - previous_value;
    let percenteage = diff / previous_value * 100.;

    return format!(
        "started with {:.2} EUR and ended with {:.2} EUR, for a total gain of {} EUR ({}%)",
        current_value, previous_value, diff, percenteage
    );
}

impl IteractionLogger {
    pub fn new() -> IteractionLogger {
        IteractionLogger {
            trader: vec![],
            sol: vec![],
            rcnz: vec![],
            bose: vec![],
        }
    }

    pub fn update(&mut self, trader: Account, sol: Account, rcnz: Account, bose: Account) {
        self.trader.push(trader);
        self.sol.push(sol);
        self.rcnz.push(rcnz);
        self.bose.push(bose);
    }

    /// parses a MarketWrapper before calling update
    pub fn update_from_market_wrapper(&mut self, account: &Account, markets: &MarketWrapper) {
        let trader = account.clone();
        let sol = Account::new_from_good_labels(markets.sol.borrow().get_goods());
        let rcnz = Account::new_from_good_labels(markets.rcnz.borrow().get_goods());
        let bose = Account::new_from_good_labels(markets.bose.borrow().get_goods());

        self.update(trader, sol, rcnz, bose);
    }

    pub fn log_last_difference(&self) {
        // TODO BETTER WAY TO CYCLE THIS
        let account_vecs = [&self.trader, &self.sol, &self.rcnz, &self.bose];
        let account_names = ["trader", "sol", "rcnz", "bose"];
        for i in 0..4 {
            let account = get_last_item(account_vecs[i], 0);
            if account.is_some() {
                // this way we can have a minimum of characters
                let s = format!("[{}]", account.unwrap());
                println!(
                    "{}:\t{:<65}\t{}",
                    account_names[i],
                    s,
                    get_account_gain_string(account_vecs[i])
                );
            }
        }

        println!();
    }

    pub fn log_final_difference(&self) {
        // TODO BETTER WAY TO CYCLE THIS
        let account_vecs = [&self.trader, &self.sol, &self.rcnz, &self.bose];
        let account_names = ["trader", "sol", "rcnz", "bose"];
        for i in 0..4 {
            let account = get_last_item(account_vecs[i], 0);
            if account.is_some() {
                println!(
                    "{} {}",
                    account_names[i],
                    get_final_account_gain_string(account_vecs[i])
                );
            }
        }
    }

    pub fn generate_svgs(&self) {
        // TODO BETTER WAY TO CYCLE THIS
        let account_vecs = [&self.trader, &self.sol, &self.rcnz, &self.bose];
        let account_names = ["Trader", "SOL", "RCNZ", "BOSE"];
        for i in 0..4 {
            create_account_graphs(
                account_vecs[i],
                format!("graph_{}.svg", account_names[i]).as_str(),
                format!("{} Account", account_names[i]).as_str(),
            )
        }
    }
}
