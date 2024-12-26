use std::fmt::{Display, Formatter};

use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::market::{BuyError, LockBuyError, LockSellError, SellError};

use doge_common::account::Account;

#[derive(Debug)]
pub struct TradingEvent {
    /// Status of account at the time of the event
    pub account: Account,
    /// Market with which the trader interacted
    pub market: String,
    /// Actual event
    pub event: TradingEventType,
}

impl Display for TradingEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Current state: {} {} {}", self.account, self.market, self.event)
    }
}

#[derive(Debug)]
pub enum TradingEventType {
    LockBuy { kind: GoodKind, quantity: f32, bid: f32, result: Result<String, LockBuyError> },
    LockSell { kind: GoodKind, quantity: f32, offer: f32, result: Result<String, LockSellError> },
    Buy { token: String, kind: GoodKind, quantity: f32, bid: f32, result: Result<Good, BuyError> },
    Sell { token: String, kind: GoodKind, quantity: f32, offer: f32, result: Result<Good, SellError> },
}

impl Display for TradingEventType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TradingEventType::LockBuy { kind, quantity, bid, result } => {
                match result {
                    Ok(token) => write!(f, "[LOCK BUY SUCCESS] {} Locked {} {} with bid {} {}",
                                        token,
                                        kind,
                                        quantity,
                                        GoodKind::EUR,
                                        bid),
                    Err(err) => write!(f, "[LOCK BUY ERROR] {:?}", err),
                }
            }
            TradingEventType::LockSell { kind, quantity, offer, result } => {
                match result {
                    Ok(token) => write!(f, "[LOCK SELL SUCCESS] {} Locked {} {} with offer {} {}",
                                        token,
                                        kind,
                                        quantity,
                                        GoodKind::EUR,
                                        offer),
                    Err(err) => write!(f, "[LOCK SELL ERROR] {:?}", err),
                }
            }
            TradingEventType::Buy { token, kind, quantity, bid, result } => {
                if let Err(err) = result {
                    write!(f, "[BUY ERROR] {} {:?}", token, err)
                } else {
                    write!(f, "[BUY SUCCESS] {} Bought {} {} with bid {} {}",
                           token,
                           kind,
                           quantity,
                           GoodKind::EUR,
                           bid)
                }
            }
            TradingEventType::Sell { token, kind, quantity, offer, result } => {
                if let Err(err) = result {
                    write!(f, "[SELL ERROR] {} {:?}", token, err)
                } else {
                    write!(f, "[SELL SUCCESS] {} Sold {} {} with bid {} {}",
                           token,
                           kind,
                           quantity,
                           GoodKind::EUR,
                           offer)
                }
            }
        }
    }
}
