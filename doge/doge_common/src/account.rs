use std::fmt::{Debug, Display, Formatter};

use unitn_market_2022::good::consts::{DEFAULT_EUR_USD_EXCHANGE_RATE, DEFAULT_EUR_YEN_EXCHANGE_RATE, DEFAULT_EUR_YUAN_EXCHANGE_RATE};
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_error::GoodSplitError;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::good::good_kind::GoodKind::{EUR, USD, YEN, YUAN};
use unitn_market_2022::market::good_label::GoodLabel;

#[derive(Debug, Clone)]
pub struct Account {
    pub eur: Good,
    pub usd: Good,
    pub yen: Good,
    pub yuan: Good,
}

impl Display for Account {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "EUR: {:.2}, USD: {:.2}, YEN: {:.2}, YUAN: {:.2}",
            self.eur.get_qty(),
            self.usd.get_qty(),
            self.yen.get_qty(),
            self.yuan.get_qty()
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum WithdrawError {
    NonPositiveWithdrawQuantity,
    WithdrawExcessiveQuantity { withdrawable: f32 },
}

impl Account {
    pub fn new_empty() -> Account {
        Account {
            eur: Good::new(EUR, 0.),
            usd: Good::new(USD, 0.),
            yen: Good::new(YEN, 0.),
            yuan: Good::new(YUAN, 0.),
        }
    }

    pub fn new_from_good_labels(labels: Vec<GoodLabel>) -> Account {
        let mut account = Account::new_empty();

        for label in labels {
            account.deposit(Good::new(label.good_kind, label.quantity));
        }

        account
    }

    pub fn deposit(&mut self, good: Good) {
        match good.get_kind() {
            EUR => self.eur.merge(good).unwrap(),
            YEN => self.yen.merge(good).unwrap(),
            USD => self.usd.merge(good).unwrap(),
            YUAN => self.yuan.merge(good).unwrap(),
        };
    }

    pub fn withdraw(&mut self, kind: GoodKind, quantity: f32) -> Result<Good, WithdrawError> {
        self.get_good_by_kind_mut(kind)
            .split(quantity)
            .map_err(|err| match err {
                GoodSplitError::NonPositiveSplitQuantity => {
                    WithdrawError::NonPositiveWithdrawQuantity
                }
                GoodSplitError::NotEnoughQuantityToSplit => {
                    WithdrawError::WithdrawExcessiveQuantity {
                        withdrawable: self.get_quantity_by_kind(kind),
                    }
                }
            })
    }

    pub fn get_good_by_kind(&self, kind: GoodKind) -> &Good {
        match kind {
            EUR => &self.eur,
            USD => &self.usd,
            YEN => &self.yen,
            YUAN => &self.yuan,
        }
    }

    fn get_good_by_kind_mut(&mut self, kind: GoodKind) -> &mut Good {
        match kind {
            EUR => &mut self.eur,
            YEN => &mut self.yen,
            USD => &mut self.usd,
            YUAN => &mut self.yuan,
        }
    }

    pub fn get_quantity_by_kind(&self, kind: GoodKind) -> f32 {
        self.get_good_by_kind(kind).get_qty()
    }

    pub fn get_total_account_value_in_eur(&self) -> f32 {
        self.get_quantity_by_kind(EUR) +
            self.get_quantity_by_kind(USD) / DEFAULT_EUR_USD_EXCHANGE_RATE +
            self.get_quantity_by_kind(YEN) / DEFAULT_EUR_YEN_EXCHANGE_RATE +
            self.get_quantity_by_kind(YUAN) / DEFAULT_EUR_YUAN_EXCHANGE_RATE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_println() {
        let account = Account {
            eur: Good::new(EUR, 1234.5678),
            usd: Good::new(USD, 9876.5432),
            yen: Good::new(YEN, 1111.2222),
            yuan: Good::new(YUAN, 1357.2468),
        };

        println!("{}", account);
    }
}
