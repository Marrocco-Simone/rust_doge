//common functions that can be used in strategies to interact with markets

use std::cell::RefCell;
use std::rc::Rc;

use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::market::{BuyError, LockBuyError, LockSellError, Market, MarketGetterError, SellError};

use doge_common::account::{Account, WithdrawError};

use crate::DOGE_TRADER_NAME;
use crate::event::TradingEvent;
use crate::event::TradingEventType::{Buy, LockBuy, LockSell, Sell};
use crate::event_handler::EventHandler;

#[derive(Debug)]
pub enum TraderError {
    LockBuyTraderError(LockBuyError),
    LockSellTraderError(LockSellError),
    BuyTraderError(BuyError),
    SellTraderError(SellError),
    WithdrawTraderError(WithdrawError),
    PriceTraderError(MarketGetterError),
    RcnzBUG,
}

#[derive(Clone, Debug)]
pub struct ReservedLock {
    pub token: String,
    pub good_kind: GoodKind,
    pub bid_offer: f32,
    pub quantity: f32,
}

//lock buy a good quantity with lowest bid, return the token or error
pub fn lock_buy(handler: &Box<dyn EventHandler>, account: &mut Account, market: Rc<RefCell<dyn Market>>, good_kind: GoodKind, quantity: f32) -> Result<ReservedLock, TraderError> {
    let mut bid;
    match market.borrow_mut().get_buy_price(good_kind, quantity) {
        Ok(price) => {
            //todo remove once RCNZ fixes the bug
            if price < 0. {
                return Err(TraderError::RcnzBUG)
            }
            bid = price
        }
        Err(err) => { return Err(TraderError::PriceTraderError(err)) }
    }
    let result = market.borrow_mut().lock_buy(good_kind, quantity, bid, DOGE_TRADER_NAME.to_string());
    let event = TradingEvent {
        account: account.clone(),
        market: DOGE_TRADER_NAME.to_string(),
        event: LockBuy { kind: good_kind, quantity, bid, result: result.clone() },
    };
    handler.handle_event(event);
    match result {
        Ok(token) => {
            Ok(ReservedLock {
                token,
                good_kind,
                bid_offer: bid,
                quantity,
            })
        }
        Err(err) => { Err(TraderError::LockBuyTraderError(err)) }
    }
}

//lock sell with the highest offer, return the token or error
pub fn lock_sell(handler: &Box<dyn EventHandler>, account: &mut Account, market: Rc<RefCell<dyn Market>>, good_kind: GoodKind, quantity: f32) -> Result<ReservedLock, TraderError> {
    let offer;
    match market.borrow_mut().get_sell_price(good_kind, quantity) {
        Ok(price) => {
            //todo remove once RCNZ fixes the bug
            if price < 0. {
                return Err(TraderError::RcnzBUG)
            }
            offer = price;
        }
        Err(err) => { return Err(TraderError::PriceTraderError(err)) }
    }
    let result = market.borrow_mut().lock_sell(good_kind, quantity, offer, DOGE_TRADER_NAME.to_string());
    let event = TradingEvent {
        account: account.clone(),
        market: DOGE_TRADER_NAME.to_string(),
        event: LockSell { kind: good_kind, quantity, offer, result: result.clone() },
    };
    handler.handle_event(event);
    match result {
        Ok(token) => {
            Ok(ReservedLock {
                token,
                good_kind,
                bid_offer: offer,
                quantity,
            })
        }
        Err(err) => { Err(TraderError::LockSellTraderError(err)) }
    }
}

//buy with the token and the right amount, return void or an error
pub fn buy(handler: &Box<dyn EventHandler>, account: &mut Account, market: Rc<RefCell<dyn Market>>, reservation: ReservedLock) -> Result<(), TraderError> {
    let mut cash;
    match account.withdraw(DEFAULT_GOOD_KIND, reservation.bid_offer) {
        Ok(res) => {
            cash = res
        }
        Err(err) => { return Err(TraderError::WithdrawTraderError(err)) }
    };
    let result = market.borrow_mut().buy(reservation.token.clone(), &mut cash);
    //before reporting the event, update the account
    match result.clone() {
        Ok(good_obtained) => { account.deposit(good_obtained) }
        Err(_) => { account.deposit(cash) }
    }
    let event = TradingEvent {
        account: account.clone(),
        market: DOGE_TRADER_NAME.to_string(),
        event: Buy {
            token: reservation.token,
            kind: reservation.good_kind,
            quantity: reservation.quantity,
            bid: reservation.bid_offer,
            result: result.clone(),
        },
    };
    handler.handle_event(event);
    return match result {
        Ok(_) => { Ok(()) }
        Err(err) => { Err(TraderError::BuyTraderError(err)) }
    }
}

//sell with the token and the right quantity, return void or an error
pub fn sell(handler: &Box<dyn EventHandler>, account: &mut Account, market: Rc<RefCell<dyn Market>>, reservation: ReservedLock) -> Result<(), TraderError> {
    let mut good_to_sell;
    match account.withdraw(reservation.good_kind, reservation.quantity) {
        Ok(res) => { good_to_sell = res }
        Err(err) => { return Err(TraderError::WithdrawTraderError(err)) }
    };
    let result = market.borrow_mut().sell(reservation.token.clone(), &mut good_to_sell);
    //before reporting the event, update the account
    match result.clone() {
        Ok(cash) => { account.deposit(cash) }
        Err(_) => { account.deposit(good_to_sell) }
    }
    let event = TradingEvent {
        account: account.clone(),
        market: DOGE_TRADER_NAME.to_string(),
        event: Sell {
            token: reservation.token,
            kind: reservation.good_kind,
            quantity: reservation.quantity,
            result: result.clone(),
            offer: reservation.bid_offer,
        },
    };
    handler.handle_event(event);
    return match result {
        Ok(_) => {
            Ok(())
        }
        Err(err) => { Err(TraderError::SellTraderError(err)) }
    }
}

//lock buy and buy directly, return void or error
pub fn direct_buy(handler: &Box<dyn EventHandler>, account: &mut Account, market: Rc<RefCell<dyn Market>>, good_kind: GoodKind, quantity: f32) -> Result<(), TraderError> {
    match lock_buy(&handler, account, market.clone(), good_kind, quantity) {
        Ok(reservation) => {
            match buy(&handler, account, market.clone(), reservation) {
                Ok(_) => { Ok(()) }
                Err(err) => { Err(err) }
            }
        }
        Err(err) => { Err(err) }
    }
}

//lock sell and sell directly, return void or error
pub fn direct_sell(handler: &Box<dyn EventHandler>, account: &mut Account, market: Rc<RefCell<dyn Market>>, good_kind: GoodKind, quantity: f32) -> Result<(), TraderError> {
    match lock_sell(&handler, account, market.clone(), good_kind, quantity) {
        Ok(reservation) => {
            match sell(&handler, account, market.clone(), reservation) {
                Ok(_) => { Ok(()) }
                Err(err) => { Err(err) }
            }
        }
        Err(err) => { Err(err) }
    }
}

pub fn get_market_good_quantity(market: Rc<RefCell<dyn Market>>, good_kind: GoodKind) -> Option<f32> {
    let goods = market.borrow().get_goods();
    for good in goods {
        if good.good_kind == good_kind {
            return Some(good.quantity);
        }
    }
    return None;
}

#[cfg(test)]
mod tests {
    use unitn_market_2022::good::good::Good;
    use unitn_market_2022::good::good_kind::GoodKind::{EUR, USD, YEN, YUAN};

    use doge_common::account::Account;

    use crate::event_handler::EventHandler;
    use crate::logger::stdout_logger::StdoutLogger;
    use crate::markets::MarketWrapper;
    use crate::strategy::common::{direct_buy, direct_sell, lock_buy, lock_sell, TraderError};

    const STARTING_QUANTITY: f32 = 100_000.;

    #[test]
    fn locks() {
        let mut account = Account {
            eur: Good::new(EUR, STARTING_QUANTITY),
            usd: Good::new(USD, STARTING_QUANTITY),
            yen: Good::new(YEN, STARTING_QUANTITY),
            yuan: Good::new(YUAN, STARTING_QUANTITY),
        };
        let mut markets = MarketWrapper::new();
        let handler: Box<dyn EventHandler> = Box::new(StdoutLogger::new());
        assert!(lock_buy(&handler, &mut account, markets.bose.clone(), USD, 1.).is_ok());
        assert!(lock_sell(&handler, &mut account, markets.bose.clone(), USD, 1.).is_ok());
        assert!(lock_buy(&handler, &mut account, markets.bose.clone(), USD, 10_000_000.).is_err());
        assert!(lock_sell(&handler, &mut account, markets.bose.clone(), USD, 10_000_000.).is_ok());
        assert_eq!(account.eur.get_qty(), STARTING_QUANTITY);
        assert_eq!(account.usd.get_qty(), STARTING_QUANTITY);
        assert_eq!(account.yen.get_qty(), STARTING_QUANTITY);
        assert_eq!(account.yuan.get_qty(), STARTING_QUANTITY);
    }

    #[test]
    fn direct__buy() {
        let mut account = Account {
            eur: Good::new(EUR, STARTING_QUANTITY),
            usd: Good::new(USD, STARTING_QUANTITY),
            yen: Good::new(YEN, STARTING_QUANTITY),
            yuan: Good::new(YUAN, STARTING_QUANTITY),
        };
        let mut markets = MarketWrapper::new();
        let handler: Box<dyn EventHandler> = Box::new(StdoutLogger::new());

        let bid = markets.bose.clone().borrow_mut().get_buy_price(USD, 50.).unwrap();
        let res = direct_buy(&handler, &mut account, markets.bose.clone(), USD, 50.);
        assert!(res.is_ok());
        assert_eq!(account.usd.get_qty(), STARTING_QUANTITY + 50.);
        assert_eq!(account.eur.get_qty(), STARTING_QUANTITY - bid);
        assert_eq!(account.yen.get_qty(), STARTING_QUANTITY);
        assert_eq!(account.yuan.get_qty(), STARTING_QUANTITY);


        account = Account {
            eur: Good::new(EUR, STARTING_QUANTITY),
            usd: Good::new(USD, STARTING_QUANTITY),
            yen: Good::new(YEN, STARTING_QUANTITY),
            yuan: Good::new(YUAN, STARTING_QUANTITY),
        };
        let bid = markets.bose.clone().borrow_mut().get_buy_price(YEN, 150.).unwrap();
        let res = direct_buy(&handler, &mut account, markets.bose.clone(), YEN, 150.);
        assert!(res.is_ok());
        assert_eq!(account.yen.get_qty(), STARTING_QUANTITY + 150.);
        assert_eq!(account.eur.get_qty(), STARTING_QUANTITY - bid);
        assert_eq!(account.usd.get_qty(), STARTING_QUANTITY);
        assert_eq!(account.yuan.get_qty(), STARTING_QUANTITY);

        account = Account {
            eur: Good::new(EUR, STARTING_QUANTITY),
            usd: Good::new(USD, STARTING_QUANTITY),
            yen: Good::new(YEN, STARTING_QUANTITY),
            yuan: Good::new(YUAN, STARTING_QUANTITY),
        };
        let res = direct_buy(&handler, &mut account, markets.bose.clone(), YEN, 150_000_000_000.);
        assert!(res.is_err());
        assert_eq!(account.yen.get_qty(), STARTING_QUANTITY);
        assert_eq!(account.eur.get_qty(), STARTING_QUANTITY);
        assert_eq!(account.usd.get_qty(), STARTING_QUANTITY);
        assert_eq!(account.yuan.get_qty(), STARTING_QUANTITY);
    }

    #[test]
    fn direct__sell() {
        let mut account = Account {
            eur: Good::new(EUR, STARTING_QUANTITY),
            usd: Good::new(USD, STARTING_QUANTITY),
            yen: Good::new(YEN, STARTING_QUANTITY),
            yuan: Good::new(YUAN, STARTING_QUANTITY),
        };
        let mut markets = MarketWrapper::new();
        let handler: Box<dyn EventHandler> = Box::new(StdoutLogger::new());

        let offer = markets.bose.clone().borrow_mut().get_sell_price(USD, 77.).unwrap();
        let res = direct_sell(&handler, &mut account, markets.bose.clone(), USD, 77.);
        assert!(res.is_ok());
        assert_eq!(account.usd.get_qty(), STARTING_QUANTITY - 77.);
        assert_eq!(account.eur.get_qty(), STARTING_QUANTITY + offer);
        assert_eq!(account.yen.get_qty(), STARTING_QUANTITY);
        assert_eq!(account.yuan.get_qty(), STARTING_QUANTITY);


        account = Account {
            eur: Good::new(EUR, STARTING_QUANTITY),
            usd: Good::new(USD, STARTING_QUANTITY),
            yen: Good::new(YEN, STARTING_QUANTITY),
            yuan: Good::new(YUAN, STARTING_QUANTITY),
        };
        let offer = markets.bose.clone().borrow_mut().get_sell_price(YUAN, 777.).unwrap();
        let res = direct_sell(&handler, &mut account, markets.bose.clone(), YUAN, 777.);
        assert!(res.is_ok());
        assert_eq!(account.yuan.get_qty(), STARTING_QUANTITY - 777.);
        assert_eq!(account.eur.get_qty(), STARTING_QUANTITY + offer);
        assert_eq!(account.yen.get_qty(), STARTING_QUANTITY);
        assert_eq!(account.usd.get_qty(), STARTING_QUANTITY);


        account = Account {
            eur: Good::new(EUR, STARTING_QUANTITY),
            usd: Good::new(USD, STARTING_QUANTITY),
            yen: Good::new(YEN, STARTING_QUANTITY),
            yuan: Good::new(YUAN, STARTING_QUANTITY),
        };
        let res = direct_sell(&handler, &mut account, markets.bose.clone(), USD, 777_777_777.);
        assert!(res.is_err());
        assert_eq!(account.usd.get_qty(), STARTING_QUANTITY);
        assert_eq!(account.eur.get_qty(), STARTING_QUANTITY);
        assert_eq!(account.yen.get_qty(), STARTING_QUANTITY);
        assert_eq!(account.yuan.get_qty(), STARTING_QUANTITY);
    }
}
