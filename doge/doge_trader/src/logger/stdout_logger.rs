use crate::event::TradingEvent;
use crate::event_handler::EventHandler;

/// Logger that logs to standard output.
pub struct StdoutLogger;

impl StdoutLogger {
    pub fn new() -> StdoutLogger {
        StdoutLogger {}
    }
}

impl EventHandler for StdoutLogger {
    fn handle_event(&self, event: TradingEvent) {
        println!("{event}")
    }

    fn get_name(&self) -> &'static str {
        "Stdout logger"
    }
}

#[cfg(test)]
mod tests {
    use unitn_market_2022::good::good::Good;
    use unitn_market_2022::good::good_kind::GoodKind;
    use unitn_market_2022::good::good_kind::GoodKind::{EUR, USD};
    use unitn_market_2022::market::{BuyError, LockBuyError, LockSellError, SellError};

    use doge_common::account::Account;

    use crate::event::TradingEventType;

    use super::*;

    #[test]
    fn log_lock_buy() {
        let account = Account {
            eur: Good::new(GoodKind::EUR, 1234.5678),
            usd: Good::new(GoodKind::USD, 9876.5432),
            yen: Good::new(GoodKind::YEN, 1111.2222),
            yuan: Good::new(GoodKind::YUAN, 1357.2468),
        };

        let logger = StdoutLogger {};

        logger.handle_event(TradingEvent {
            account: account.clone(),
            market: "FakeMarket".to_owned(),
            event: TradingEventType::LockBuy {
                kind: GoodKind::USD,
                quantity: 69.1234,
                bid: 96.4321,
                result: Ok("123e4567-e89b-12d3-a456-426614174000".to_owned()),
            },
        });

        logger.handle_event(TradingEvent {
            account,
            market: "FakeMarket".to_owned(),
            event: TradingEventType::LockBuy {
                kind: GoodKind::USD,
                quantity: 69.1234,
                bid: -96.4321,
                result: Err(LockBuyError::NonPositiveBid { negative_bid: -96.4321 }),
            },
        });
    }

    #[test]
    fn log_lock_sell() {
        let account = Account {
            eur: Good::new(GoodKind::EUR, 1234.5678),
            usd: Good::new(GoodKind::USD, 9876.5432),
            yen: Good::new(GoodKind::YEN, 1111.2222),
            yuan: Good::new(GoodKind::YUAN, 1357.2468),
        };

        let logger = StdoutLogger {};

        logger.handle_event(TradingEvent {
            market: "FakeMarket".to_owned(),
            account: account.clone(),
            event: TradingEventType::LockSell {
                kind: GoodKind::USD,
                quantity: 69.1234,
                offer: 96.4321,
                result: Ok("123e4567-e89b-12d3-a456-426614174000".to_owned()),
            },
        });

        logger.handle_event(TradingEvent {
            market: "FakeMarket".to_owned(),
            account: account.clone(),
            event: TradingEventType::LockSell {
                kind: GoodKind::USD,
                quantity: 69.1234,
                offer: -96.4321,
                result: Err(LockSellError::NonPositiveOffer { negative_offer: -96.4321 }),
            },
        });
    }

    #[test]
    fn log_buy() {
        let account = Account {
            eur: Good::new(GoodKind::EUR, 1234.5678),
            usd: Good::new(GoodKind::USD, 9876.5432),
            yen: Good::new(GoodKind::YEN, 1111.2222),
            yuan: Good::new(GoodKind::YUAN, 1357.2468),
        };

        let logger = StdoutLogger {};

        logger.handle_event(TradingEvent {
            market: "FakeMarket".to_owned(),
            account: account.clone(),
            event: TradingEventType::Buy {
                token: "123e4567-e89b-12d3-a456-426614174000".to_owned(),
                kind: GoodKind::USD,
                quantity: 69.1234,
                bid: 96.4321,
                result: Ok(Good::new(USD, 69.1234)),
            },
        });

        logger.handle_event(TradingEvent {
            market: "FakeMarket".to_owned(),
            account: account.clone(),
            event: TradingEventType::Buy {
                token: "123e4567-e89b-12d3-a456-426614174000".to_owned(),
                kind: GoodKind::USD,
                quantity: 69.1234,
                bid: 96.4321,
                result: Err(BuyError::ExpiredToken { expired_token: "123e4567-e89b-12d3-a456-426614174000".to_owned() }),
            },
        });
    }

    #[test]
    fn lock_sell() {
        let account = Account {
            eur: Good::new(GoodKind::EUR, 1234.5678),
            usd: Good::new(GoodKind::USD, 9876.5432),
            yen: Good::new(GoodKind::YEN, 1111.2222),
            yuan: Good::new(GoodKind::YUAN, 1357.2468),
        };

        let logger = StdoutLogger {};

        logger.handle_event(TradingEvent {
            market: "FakeMarket".to_owned(),
            account: account.clone(),
            event: TradingEventType::Sell {
                kind: GoodKind::USD,
                quantity: 69.1234,
                offer: 96.4321,
                token: "123e4567-e89b-12d3-a456-426614174000".to_owned(),
                result: Ok(Good::new(EUR, 96.4321)),
            },
        });

        logger.handle_event(TradingEvent {
            market: "FakeMarket".to_owned(),
            account: account.clone(),
            event: TradingEventType::Sell {
                kind: GoodKind::USD,
                quantity: 69.1234,
                offer: 96.4321,
                token: "123e4567-e89b-12d3-a456-426614174000".to_owned(),
                result: Err(SellError::ExpiredToken { expired_token: "123e4567-e89b-12d3-a456-426614174000".to_owned() }),
            },
        });
    }
}

