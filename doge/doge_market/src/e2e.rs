#[cfg(test)]
mod e2e_tests {
    use std::cell::RefMut;

    use unitn_market_2022::good::good::Good;
    use unitn_market_2022::good::good_kind::GoodKind;
    use unitn_market_2022::market::{BuyError, LockBuyError, Market};
    use unitn_market_2022::market::good_label::GoodLabel;
    use unitn_market_2022::market::market_test::*;

    use crate::dogemarket::DogeMarket;

    #[test]
    pub fn test_name_run() {
        test_name::<DogeMarket>();
    }

    #[test]
    pub fn test_get_buy_price_success_run() {
        test_get_buy_price_success::<DogeMarket>();
    }
    #[test]
    pub fn test_get_buy_price_non_positive_error_run() {
        test_get_buy_price_non_positive_error::<DogeMarket>();
    }
    #[test]
    pub fn test_get_buy_price_insufficient_qty_error_run() {
        test_get_buy_price_insufficient_qty_error::<DogeMarket>();
    }
    #[test]
    pub fn test_get_sell_price_success_run() {
        test_get_sell_price_success::<DogeMarket>();
    }
    #[test]
    pub fn test_get_sell_price_non_positive_error_run() {
        test_get_sell_price_non_positive_error::<DogeMarket>();
    }
    #[test]
    pub fn test_deadlock_prevention_run() {
        test_deadlock_prevention::<DogeMarket>();
    }
    #[test]
    pub fn test_test_new_randomname_run() {
        test_new_random::<DogeMarket>();
    }
    #[test]
    pub fn test_price_change_after_buy_run() {
        test_price_change_after_buy::<DogeMarket>();
    }
    #[test]
    pub fn price_changes_waiting_run() {
        // our prices don't change if the if there are no buy/sells (or locks)
        // price_changes_waiting::<DogeMarket>();
    }
    #[test]
    pub fn test_price_change_after_sell_run() {
        test_price_change_after_sell::<DogeMarket>();
    }
    #[test]
    pub fn should_initialize_with_right_quantity_run() {
        should_initialize_with_right_quantity::<DogeMarket>();
    }
    #[test]
    pub fn new_random_should_not_exceeed_starting_capital_run() {
        new_random_should_not_exceeed_starting_capital::<DogeMarket>();
    }
    #[test]
    pub fn test_sell_success_run() {
        test_sell_success::<DogeMarket>();
    }
    #[test]
    pub fn test_sell_unrecognized_token_run() {
        test_sell_unrecognized_token::<DogeMarket>();
    }
    #[test]
    pub fn test_sell_expired_token_run() {
        test_sell_expired_token::<DogeMarket>();
    }
    #[test]
    pub fn test_sell_wrong_good_kind_run() {
        test_sell_wrong_good_kind::<DogeMarket>();
    }
    #[test]
    pub fn test_sell_insufficient_good_quantity_run() {
        test_sell_insufficient_good_quantity::<DogeMarket>();
    }
    #[test]
    pub fn test_lock_sell_nonPositiveOffer_run() {
        test_lock_sell_nonPositiveOffer::<DogeMarket>();
    }
    #[test]
    pub fn test_lock_sell_defaultGoodAlreadyLocked_run() {
        // We don't test this because we allow any good to have multiple locks at the same time
        // test_lock_sell_defaultGoodAlreadyLocked::<DogeMarket>();
    }
    #[test]
    pub fn test_lock_sell_maxAllowedLocksReached_run() {
        // We don't test this because our market deadlock prevention strategy [1] is "time unlocking"
        // and not "lock counting"
        // [1] https://github.com/WG-AdvancedProgramming/market-protocol-specifications/blob/main/market-protocol-specifications.md#market-deadlock
        // test_lock_sell_maxAllowedLocksReached::<DogeMarket>();
    }
    #[test]
    pub fn test_lock_sell_insufficientDefaultGoodQuantityAvailable_run() {
        test_lock_sell_insufficientDefaultGoodQuantityAvailable::<DogeMarket>();
    }
    #[test]
    pub fn test_lock_sell_offerTooHigh_run() {
        test_lock_sell_offerTooHigh::<DogeMarket>();
    }
    #[test]
    pub fn test_working_function_lock_sell_token_run() {
        test_working_function_lock_sell_token::<DogeMarket>();
    }
    #[test]
    pub fn test_lock_buy_non_positive_quantity_to_buy_run() {
        test_lock_buy_non_positive_quantity_to_buy::<DogeMarket>();
    }
    #[test]
    pub fn test_lock_buy_non_positive_bid_run() {
        test_lock_buy_non_positive_bid::<DogeMarket>();
    }
    #[test]
    pub fn test_get_budget_run() {
        test_get_budget::<DogeMarket>();
    }

    #[test]
    pub fn test_get_goods_run() {
        test_get_goods::<DogeMarket>();
    }

    #[test]
    pub fn test_get_name_run() {
        test_get_name::<DogeMarket>();
    }

    #[test]
    pub fn test_lock_buy_insufficient_good_quantity_available() {
        let market = DogeMarket::new_with_quantities(100000.0, 100000.0, 100000.0, 100000.0);
        // let quantity = 1000.0;
        // let lowest_bid = f32::MIN_POSITIVE; // it's temporary because Lowest Bid must be the last error to verify
        let trader_name = String::from("TEST");
        let labels = market.borrow().get_goods();

        for good_label in labels {
            //LockBuyError::InsufficientGoodQuantityAvailable
            let token = market.borrow_mut().lock_buy(
                good_label.good_kind,
                good_label.quantity * 2.,
                f32::MAX,
                trader_name.clone(),
            );
            assert_eq!(
                token,
                Err(LockBuyError::InsufficientGoodQuantityAvailable {
                    requested_good_kind: good_label.good_kind,
                    requested_good_quantity: good_label.quantity * 2.0,
                    available_good_quantity: good_label.quantity,
                })
            );
        }
    }

    #[test]
    pub fn test_lock_buy_bid_too_low() {
        let market = DogeMarket::new_with_quantities(100000.0, 100000.0, 100000.0, 100000.0);
        let quantity = 1000.0;
        let lowest_bid = f32::MIN_POSITIVE; // it's temporary because Lowest Bid must be the last error to verify
        let trader_name = String::from("TEST");
        let labels = market.borrow().get_goods();

        for good_label in labels {
            let token = market.borrow_mut().lock_buy(
                good_label.good_kind,
                quantity,
                lowest_bid,
                trader_name.clone(),
            );
            assert!(match token {
                Err(LockBuyError::BidTooLow {
                        requested_good_kind,
                        requested_good_quantity,
                        low_bid,
                        ..
                    }) => {
                    requested_good_kind == good_label.good_kind
                        && requested_good_quantity == quantity
                        && low_bid == lowest_bid
                }
                _ => false,
            });
        }
    }

    #[test]
    pub fn test_buy_unrecognized_token() {
        let market = DogeMarket::new_with_quantities(100000.0, 100000.0, 100000.0, 100000.0);
        let labels = market.borrow().get_goods();

        for good_label in labels {
            let (_, price) = get_lock_token(market.borrow_mut(), &good_label, 5000.0);
            let mut cash = Good::new(GoodKind::EUR, price);
            // let gk = good_label.good_kind;

            // BuyError::UnrecognizedToken { unrecognized_token: String }
            let g = market
                .borrow_mut()
                .buy("STRANGE_TOKEN".to_string(), &mut cash);
            assert_eq!(
                g,
                Err(BuyError::UnrecognizedToken {
                    unrecognized_token: String::from("STRANGE_TOKEN")
                })
            );
        }
    }

    #[test]
    pub fn test_buy_good_kind_not_default() {
        let market = DogeMarket::new_with_quantities(100000.0, 100000.0, 100000.0, 100000.0);
        let labels = market.borrow().get_goods();

        for good_label in labels {
            let (token, price) = get_lock_token(market.borrow_mut(), &good_label, 5000.0);
            // let mut cash = Good::new(GoodKind::EUR, price);
            // let gk = good_label.good_kind;

            // BuyError::GoodKindNotDefault { non_default_good_kind: GoodKind }
            let mut not_default_good = if GoodKind::YEN != GoodKind::EUR {
                Good::new(GoodKind::YEN, price)
            } else {
                Good::new(GoodKind::EUR, price)
            };
            let g = market
                .borrow_mut()
                .buy(token.clone(), &mut not_default_good);
            assert_eq!(
                g,
                Err(BuyError::GoodKindNotDefault {
                    non_default_good_kind: not_default_good.get_kind()
                })
            );
        }
    }

    #[test]
    pub fn test_buy_insufficient_good_quantity() {
        let market = DogeMarket::new_with_quantities(100000.0, 100000.0, 100000.0, 100000.0);
        let labels = market.borrow().get_goods();

        for good_label in labels {
            let (token, price) = get_lock_token(market.borrow_mut(), &good_label, 5000.0);
            // let mut cash = Good::new(GoodKind::EUR, price);
            // let gk = good_label.good_kind;

            // BuyError::InsufficientGoodQuantity { contained_quantity: f32, pre_agreed_quantity: f32 }
            let mut insufficient_good = Good::new(GoodKind::EUR, price / 2.0);
            let g = market
                .borrow_mut()
                .buy(token.clone(), &mut insufficient_good);
            assert_eq!(
                g,
                Err(BuyError::InsufficientGoodQuantity {
                    contained_quantity: price / 2.0,
                    pre_agreed_quantity: price,
                })
            );
        }
    }

    #[test]
    pub fn test_buy_success() {
        let market = DogeMarket::new_with_quantities(100000.0, 100000.0, 100000.0, 100000.0);
        let labels = market.borrow().get_goods();

        for good_label in labels {
            // buy
            let (token, price) = get_lock_token(market.borrow_mut(), &good_label, 5000.0);
            let mut cash = Good::new(GoodKind::EUR, price);
            let gk = good_label.good_kind;

            //SUCCESS
            let g = market.borrow_mut().buy(token.clone(), &mut cash).unwrap();
            assert_eq!(g.get_kind(), gk);
            assert_eq!(g.get_qty(), 5000.0);
        }
    }

    #[test]
    pub fn test_price_increase() {
        //test for different market initial values
        for initial_value in (100_000..1_000_000).step_by(20_000) {
            //test for different quantities
            for quantity_to_buy in (1_000..9_000).step_by(1000) {
                let initial_value = initial_value as f32;
                let quantity_to_buy = quantity_to_buy as f32;

                let market =
                    DogeMarket::new_with_quantities(initial_value, initial_value, initial_value, initial_value);

                //test for each GoodKind
                for good_kind in vec![GoodKind::YEN, GoodKind::USD, GoodKind::YUAN] {
                    let initial_price = market
                        .borrow()
                        .get_buy_price(good_kind, quantity_to_buy)
                        .expect("TEST_PRICE_INCREASE: cannot get buy price");
                    let incash = f32::MAX / (market.borrow().get_goods().len() as f32) / 10.0;
                    let token = market.borrow_mut().lock_buy(
                        good_kind,
                        quantity_to_buy,
                        incash,
                        "TEST_PRICE_INCREASE".to_string(),
                    );
                    market.borrow_mut()
                        .buy(token.expect("TEST_PRICE_INCREASE: Something went wrong when locking a buy"), &mut Good::new(GoodKind::EUR, incash))
                        .expect("TEST_PRICE_INCREASE: a problem occurred while buying from the market (got a BuyError)");
                    let final_price = market.borrow().get_buy_price(good_kind, quantity_to_buy);

                    let error_message = format!(
                        "TEST_PRICE_INCREASE: price has not increased. Arguments:\n\
                    get_buy_price({}, {})",
                        good_kind.to_string(),
                        quantity_to_buy
                    );

                    assert!(final_price.unwrap() > initial_price, "{}", error_message);
                }

                let good_kind = GoodKind::EUR;
                let initial_price = market
                    .borrow()
                    .get_buy_price(good_kind, quantity_to_buy)
                    .expect("TEST_PRICE_INCREASE: cannot get buy price");
                let incash = initial_price * 2.0;
                let token = market.borrow_mut().lock_buy(
                    good_kind,
                    quantity_to_buy,
                    incash,
                    "TEST_PRICE_INCREASE".to_string(),
                );
                market.borrow_mut()
                    .buy(token.expect("TEST_PRICE_INCREASE: Something went wrong when locking a buy"), &mut Good::new(GoodKind::EUR, incash))
                    .expect("TEST_PRICE_INCREASE: a problem occurred while buying from the market (got a BuyError)");
                let final_price = market.borrow().get_buy_price(good_kind, quantity_to_buy);

                let error_message = format!(
                    "TEST_PRICE_INCREASE: price of DEFAULT_GOOD_KIND changed unexpectedly. Arguments:\n\
                    get_buy_price({}, {})",
                    good_kind.to_string(),
                    quantity_to_buy
                );

                assert_eq!(final_price.unwrap(), initial_price, "{}", error_message);
            }
        }
    }

    //function used in several tests
    //default version uses f32::MAX as bid but this is not okay for our market because our prices depend also on bids
    fn get_lock_token(
        mut market: RefMut<dyn Market>,
        good_label: &GoodLabel,
        quantity: f32,
    ) -> (String, f32) {
        let gk = good_label.good_kind;
        let bid = market.get_buy_price(gk, quantity).unwrap();
        let token = market.lock_buy(gk, quantity, bid, String::from("TEST"));
        (
            token.expect(format!("TEST_BUY: cannot lock good kind {}", gk).as_str()),
            bid,
        )
    }
}
