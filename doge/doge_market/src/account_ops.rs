use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::good::good_kind::GoodKind::EUR;

use doge_common::account::Account;

#[derive(Debug, PartialEq)]
pub enum BuyPriceComputationError {
    NonPositiveQuantity,
    NegativeExchangeRateEarnPercentage,
    ExceedsReservableQuantity { reservable: f32 },
}

#[derive(Debug, PartialEq)]
pub enum SellPriceComputationError {
    NonPositiveQuantity,
    NegativeExchangeEarnRatePercentage,
}

#[derive(Debug, PartialEq)]
pub enum BuyExchangeRateComputationError {
    NonPositiveExchangeRateEarnPercentage,
    ExceedsReservableQuantity { reservable: f32 },
}

#[derive(Debug, PartialEq)]
pub enum SellExchangeRateComputationError {
    NonPositiveExchangeRateEarnPercentage,
}

#[derive(Debug, Clone)]
pub struct AccountOps {
    pub assets: Account,
    pub reservations: Account,
    pub futures: Account,
}

impl AccountOps {
    pub fn new_empty() -> AccountOps {
        AccountOps { assets: Account::new_empty(), reservations: Account::new_empty(), futures: Account::new_empty() }
    }

    pub fn of_assets(assets: Account) -> AccountOps {
        AccountOps { assets, reservations: Account::new_empty(), futures: Account::new_empty() }
    }

    pub fn get_reservable_quantity_by_kind(&self, kind: GoodKind) -> f32 {
        self.assets.get_quantity_by_kind(kind)
    }

    pub fn get_reserved_quantity_by_kind(&self, kind: GoodKind) -> f32 {
        self.reservations.get_quantity_by_kind(kind)
    }

    pub fn get_total_quantity_by_kind(&self, kind: GoodKind) -> f32 {
        self.assets.get_quantity_by_kind(kind) + self.reservations.get_quantity_by_kind(kind)
    }

    pub fn get_future_quantity_by_kind(&self, kind: GoodKind) -> f32 {
        self.futures.get_quantity_by_kind(kind)
    }

    pub fn compute_buy_price(&self, of_kind: GoodKind, of_quantity: f32, exchange_rate_earn_percentage: f32) -> Result<f32, BuyPriceComputationError> {
        if of_quantity <= 0. {
            return Err(BuyPriceComputationError::NonPositiveQuantity);
        }

        if exchange_rate_earn_percentage < 0. {
            return Err(BuyPriceComputationError::NegativeExchangeRateEarnPercentage);
        }

        let reservable = self.get_reservable_quantity_by_kind(of_kind);
        if of_quantity > reservable {
            return Err(BuyPriceComputationError::ExceedsReservableQuantity { reservable });
        }

        let exchange_rate = if of_kind == EUR {
            1.
        } else {
            (self.get_reservable_quantity_by_kind(EUR) + self.get_reserved_quantity_by_kind(EUR) + self.get_future_quantity_by_kind(EUR)) /
                (self.get_reservable_quantity_by_kind(of_kind) - of_quantity)
        };

        let buy_price = of_quantity * exchange_rate * (100. + exchange_rate_earn_percentage) / 100.;
        assert!(buy_price >= 0.);

        Ok(buy_price)
    }

    pub fn compute_sell_price(&self, of_kind: GoodKind, of_quantity: f32, exchange_rate_earn_percentage: f32) -> Result<f32, SellPriceComputationError> {
        if of_quantity <= 0. {
            return Err(SellPriceComputationError::NonPositiveQuantity);
        }

        if exchange_rate_earn_percentage < 0. {
            return Err(SellPriceComputationError::NegativeExchangeEarnRatePercentage);
        }

        let exchange_rate = if of_kind == EUR {
            1.
        } else {
            self.get_reservable_quantity_by_kind(EUR) / (self.get_reservable_quantity_by_kind(of_kind) + self.get_reserved_quantity_by_kind(of_kind) + self.get_future_quantity_by_kind(of_kind) + of_quantity)
        };

        let sell_price = of_quantity * exchange_rate * (100. - exchange_rate_earn_percentage) / 100.;

        Ok(sell_price)
    }

    pub fn compute_buy_exchange_rate(&self, kind: GoodKind, exchange_rate_earn_percentage: f32) -> Result<f32, BuyExchangeRateComputationError> {
        self.compute_buy_price(kind, 1., exchange_rate_earn_percentage)
            .map_err(|err| match err {
                BuyPriceComputationError::NonPositiveQuantity => unreachable!(),
                BuyPriceComputationError::NegativeExchangeRateEarnPercentage => BuyExchangeRateComputationError::NonPositiveExchangeRateEarnPercentage,
                BuyPriceComputationError::ExceedsReservableQuantity { reservable } =>
                    BuyExchangeRateComputationError::ExceedsReservableQuantity { reservable }
            })
    }

    pub fn compute_sell_exchange_rate(&self, kind: GoodKind, exchange_rate_earn_percentage: f32) -> Result<f32, SellExchangeRateComputationError> {
        self.compute_sell_price(kind, 1., exchange_rate_earn_percentage)
            .map_err(|err| match err {
                SellPriceComputationError::NonPositiveQuantity => unreachable!(),
                SellPriceComputationError::NegativeExchangeEarnRatePercentage =>
                    SellExchangeRateComputationError::NonPositiveExchangeRateEarnPercentage
            })
    }
}

#[cfg(test)]
mod tests {
    use assert_approx_eq::assert_approx_eq;
    use unitn_market_2022::good::good::Good;
    use unitn_market_2022::good::good_kind::GoodKind::{USD, YEN, YUAN};

    use super::*;

    #[test]
    fn eur_eur_exchange_buy() {
        let assets = Account {
            eur: Good::new(EUR, 120_000.),
            usd: Good::new(USD, 0.),
            yen: Good::new(YEN, 0.),
            yuan: Good::new(YUAN, 0.),
        };

        let ops = AccountOps::of_assets(assets);


        let computed = ops.compute_buy_price(EUR, 77., 1.).unwrap();
        let expected = 77. + (77. * 0.01);

        assert_eq!(computed, expected);
    }

    #[test]
    fn eur_eur_exchange_sell() {
        let assets = Account {
            eur: Good::new(EUR, 120_000.),
            usd: Good::new(USD, 0.),
            yen: Good::new(YEN, 0.),
            yuan: Good::new(YUAN, 0.),
        };

        let ops = AccountOps::of_assets(assets);

        let computed = ops.compute_sell_price(EUR, 325., 1.).unwrap();
        let expected = 325. - (325. * 0.01);

        assert_eq!(computed, expected);
    }

    #[test]
    fn buy_not_enough_quantity_eur() {
        let assets = Account {
            eur: Good::new(EUR, 2_000.),
            usd: Good::new(USD, 0.),
            yen: Good::new(YEN, 0.),
            yuan: Good::new(YUAN, 0.),
        };
        let reservations = Account {
            eur: Good::new(EUR, 5_000.),
            usd: Good::new(USD, 0.),
            yen: Good::new(YEN, 0.),
            yuan: Good::new(YUAN, 0.),
        };

        let ops = AccountOps { assets, reservations, futures: Account::new_empty() };

        let computed = ops.compute_buy_price(EUR, 5_000., 1.).unwrap_err();
        let expected = BuyPriceComputationError::ExceedsReservableQuantity { reservable: 2_000. };

        assert_eq!(computed, expected)
    }

    #[test]
    fn buy_not_enough_quantity_1() {
        let assets = Account {
            eur: Good::new(EUR, 10_000.),
            usd: Good::new(USD, 4_000.),
            yen: Good::new(YEN, 0.),
            yuan: Good::new(YUAN, 0.),
        };

        let ops = AccountOps::of_assets(assets);

        let computed = ops.compute_buy_price(USD, 9_874., 1.).unwrap_err();
        let expected = BuyPriceComputationError::ExceedsReservableQuantity { reservable: 4_000. };

        assert_eq!(computed, expected);
    }

    #[test]
    fn buy_not_enough_quantity_2() {
        let assets = Account {
            eur: Good::new(EUR, 10_000.),
            usd: Good::new(USD, 4_000.),
            yen: Good::new(YEN, 0.),
            yuan: Good::new(YUAN, 0.),
        };
        let reservations = Account {
            eur: Good::new(EUR, 100_000.),
            usd: Good::new(USD, 40_000.),
            yen: Good::new(YEN, 0.),
            yuan: Good::new(YUAN, 0.),
        };

        let ops = AccountOps { assets, reservations, futures: Account::new_empty() };

        let computed = ops.compute_buy_price(USD, 9_874., 1.).unwrap_err();
        let expected = BuyPriceComputationError::ExceedsReservableQuantity { reservable: 4_000. };

        assert_eq!(computed, expected);
    }

    #[test]
    fn test_buy_computed_1() {
        let assets = Account {
            eur: Good::new(EUR, 1_000.),
            usd: Good::new(USD, 2_000.),
            yen: Good::new(YEN, 0.),
            yuan: Good::new(YUAN, 0.),
        };

        let ops = AccountOps::of_assets(assets);

        let computed = ops.compute_buy_price(USD, 1_800., 1.).unwrap();

        assert_eq!(computed, 9_090.);
    }

    #[test]
    fn test_buy_computed_1_1() {
        let assets = Account {
            eur: Good::new(EUR, 500.),
            usd: Good::new(USD, 1_900.),
            yen: Good::new(YEN, 0.),
            yuan: Good::new(YUAN, 0.),
        };
        let reservations = Account {
            eur: Good::new(EUR, 500.),
            usd: Good::new(USD, 100.),
            yen: Good::new(YEN, 0.),
            yuan: Good::new(YUAN, 0.),
        };

        let ops = AccountOps { assets, reservations, futures: Account::new_empty() };

        let computed = ops.compute_buy_price(USD, 1_800., 1.).unwrap();

        assert_eq!(computed, 18_180.);
    }

    #[test]
    fn test_buy_computed_1_2() {
        let assets = Account {
            eur: Good::new(EUR, 500.),
            usd: Good::new(USD, 1_900.),
            yen: Good::new(YEN, 0.),
            yuan: Good::new(YUAN, 0.),
        };
        let reservations = Account {
            eur: Good::new(EUR, 500.),
            usd: Good::new(USD, 100.),
            yen: Good::new(YEN, 0.),
            yuan: Good::new(YUAN, 0.),
        };
        let futures = Account {
            eur: Good::new(EUR, 200.),
            usd: Good::new(USD, 300.),
            yen: Good::new(YEN, 0.),
            yuan: Good::new(YUAN, 0.),
        };

        let ops = AccountOps { assets, reservations, futures };

        let computed = ops.compute_buy_price(USD, 1_800., 1.).unwrap();

        assert_eq!(computed, 21_816.);
    }

    #[test]
    fn test_buy_computed_2() {
        let assets = Account {
            eur: Good::new(EUR, 53_000.),
            usd: Good::new(USD, 0.),
            yen: Good::new(YEN, 400_000.),
            yuan: Good::new(YUAN, 0.),
        };

        let ops = AccountOps::of_assets(assets);

        let computed = ops.compute_buy_price(YEN, 54_879., 1.).unwrap();

        assert_approx_eq!(computed, 8_512., 1.);
    }

    #[test]
    fn test_buy_computed_2_1() {
        let assets = Account {
            eur: Good::new(EUR, 49_000.),
            usd: Good::new(USD, 0.),
            yen: Good::new(YEN, 325_000.),
            yuan: Good::new(YUAN, 0.),
        };
        let reservations = Account {
            eur: Good::new(EUR, 4_000.),
            usd: Good::new(USD, 75_000.),
            yen: Good::new(YEN, 0.),
            yuan: Good::new(YUAN, 0.),
        };

        let ops = AccountOps { assets, reservations, futures: Account::new_empty() };

        let computed = ops.compute_buy_price(YEN, 54_879., 1.).unwrap();

        assert_approx_eq!(computed, 10_875., 1.);
    }

    #[test]
    fn test_buy_computed_2_2() {
        let assets = Account {
            eur: Good::new(EUR, 49_000.),
            usd: Good::new(USD, 0.),
            yen: Good::new(YEN, 325_000.),
            yuan: Good::new(YUAN, 0.),
        };
        let reservations = Account {
            eur: Good::new(EUR, 4_000.),
            usd: Good::new(USD, 75_000.),
            yen: Good::new(YEN, 0.),
            yuan: Good::new(YUAN, 0.),
        };
        let futures = Account {
            eur: Good::new(EUR, 8_000.),
            usd: Good::new(USD, 18_000.),
            yen: Good::new(YEN, 0.),
            yuan: Good::new(YUAN, 0.),
        };

        let ops = AccountOps { assets, reservations, futures };

        let computed = ops.compute_buy_price(YEN, 54_879., 1.).unwrap();

        assert_approx_eq!(computed, 12_516., 1.);
    }

    #[test]
    fn test_sell_computed_1() {
        let assets = Account {
            eur: Good::new(EUR, 35_000.),
            usd: Good::new(USD, 0.),
            yen: Good::new(YEN, 0.),
            yuan: Good::new(YUAN, 75_000.),
        };

        let ops = AccountOps::of_assets(assets);

        let computed = ops.compute_sell_price(YUAN, 10_500., 1.).unwrap();

        assert_approx_eq!(computed, 4_255., 1.);
    }

    #[test]
    fn test_sell_computed_1_1() {
        let assets = Account {
            eur: Good::new(EUR, 20_000.),
            usd: Good::new(USD, 0.),
            yen: Good::new(YEN, 0.),
            yuan: Good::new(YUAN, 40_000.),
        };
        let reservations = Account {
            eur: Good::new(EUR, 15_000.),
            usd: Good::new(USD, 0.),
            yen: Good::new(YEN, 0.),
            yuan: Good::new(YUAN, 35_000.),
        };

        let ops = AccountOps { assets, reservations, futures: Account::new_empty() };

        let computed = ops.compute_sell_price(YUAN, 10_500., 1.).unwrap();

        assert_approx_eq!(computed, 2_431., 1.);
    }

    #[test]
    fn test_sell_computed_1_2() {
        let assets = Account {
            eur: Good::new(EUR, 20_000.),
            usd: Good::new(USD, 0.),
            yen: Good::new(YEN, 0.),
            yuan: Good::new(YUAN, 40_000.),
        };
        let reservations = Account {
            eur: Good::new(EUR, 15_000.),
            usd: Good::new(USD, 0.),
            yen: Good::new(YEN, 0.),
            yuan: Good::new(YUAN, 35_000.),
        };
        let futures = Account {
            eur: Good::new(EUR, 5_000.),
            usd: Good::new(USD, 0.),
            yen: Good::new(YEN, 0.),
            yuan: Good::new(YUAN, 14_000.),
        };

        let ops = AccountOps { assets, reservations, futures };

        let computed = ops.compute_sell_price(YUAN, 10_500., 1.).unwrap();

        assert_approx_eq!(computed, 2_089., 1.);
    }

    #[test]
    fn test_sell_computed_2() {
        let assets = Account {
            eur: Good::new(EUR, 98_478.),
            usd: Good::new(USD, 35_454.),
            yen: Good::new(YEN, 0.),
            yuan: Good::new(YUAN, 0.),
        };

        let ops = AccountOps::of_assets(assets);

        let computed = ops.compute_sell_price(USD, 36_000., 1.).unwrap();

        assert_approx_eq!(computed, 49_119., 1.);
    }

    #[test]
    fn test_sell_computed_2_1() {
        let assets = Account {
            eur: Good::new(EUR, 90_000.),
            usd: Good::new(USD, 30_000.),
            yen: Good::new(YEN, 0.),
            yuan: Good::new(YUAN, 0.),
        };
        let reservations = Account {
            eur: Good::new(EUR, 8_478.),
            usd: Good::new(USD, 5_454.),
            yen: Good::new(YEN, 0.),
            yuan: Good::new(YUAN, 0.),
        };

        let ops = AccountOps { assets, reservations, futures: Account::new_empty() };

        let computed = ops.compute_sell_price(USD, 36_000., 1.).unwrap();

        assert_approx_eq!(computed, 44_890., 1.);
    }

    #[test]
    fn test_sell_computed_2_2() {
        let assets = Account {
            eur: Good::new(EUR, 90_000.),
            usd: Good::new(USD, 30_000.),
            yen: Good::new(YEN, 0.),
            yuan: Good::new(YUAN, 0.),
        };
        let reservations = Account {
            eur: Good::new(EUR, 8_478.),
            usd: Good::new(USD, 5_454.),
            yen: Good::new(YEN, 0.),
            yuan: Good::new(YUAN, 0.),
        };
        let futures = Account {
            eur: Good::new(EUR, 25_000.),
            usd: Good::new(USD, 15_000.),
            yen: Good::new(YEN, 0.),
            yuan: Good::new(YUAN, 0.),
        };

        let ops = AccountOps { assets, reservations, futures };

        let computed = ops.compute_sell_price(USD, 36_000., 1.).unwrap();

        assert_approx_eq!(computed, 37_101., 1.);
    }

    #[test]
    fn requested_is_zero() {
        let assets = Account {
            eur: Good::new(EUR, 90_000.),
            usd: Good::new(USD, 30_000.),
            yen: Good::new(YEN, 0.),
            yuan: Good::new(YUAN, 0.),
        };

        let ops = AccountOps::of_assets(assets);

        let computed = ops.compute_sell_price(USD, 0., 1.).unwrap_err();

        assert_eq!(computed, SellPriceComputationError::NonPositiveQuantity);
    }

    #[test]
    fn exceeds_reservable() {
        let assets = Account {
            eur: Good::new(EUR, 90_000.),
            usd: Good::new(USD, 30_000.),
            yen: Good::new(YEN, 0.),
            yuan: Good::new(YUAN, 0.),
        };

        let ops = AccountOps::of_assets(assets);

        let computed = ops.compute_buy_price(YEN, 29_000., 1.).unwrap_err();

        assert_eq!(computed, BuyPriceComputationError::ExceedsReservableQuantity { reservable: 0.0 });
    }

    #[test]
    fn requested_is_zero_and_exceeds_reservable() {
        let assets = Account {
            eur: Good::new(EUR, 90_000.),
            usd: Good::new(USD, 30_000.),
            yen: Good::new(YEN, 0.),
            yuan: Good::new(YUAN, 0.),
        };

        let ops = AccountOps::of_assets(assets);

        let computed = ops.compute_buy_price(YEN, 0., 1.).unwrap_err();

        assert_eq!(computed, BuyPriceComputationError::NonPositiveQuantity);
    }

    #[test]
    fn requested_is_minimum() {
        let assets = Account {
            eur: Good::new(EUR, 90_000.),
            usd: Good::new(USD, 30_000.),
            yen: Good::new(YEN, 0.),
            yuan: Good::new(YUAN, 0.),
        };

        let ops = AccountOps { assets, reservations: Account::new_empty(), futures: Account::new_empty() };

        let computed = ops.compute_buy_price(USD, f32::MIN_POSITIVE, 1.).unwrap();

        assert_ne!(computed, 0.);
    }
}
