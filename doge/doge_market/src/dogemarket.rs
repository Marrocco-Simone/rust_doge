use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;

use unitn_market_2022::event::event::Event;
use unitn_market_2022::event::event::EventKind::{Bought, LockedBuy, LockedSell, Sold};
use unitn_market_2022::event::notifiable::Notifiable;
use unitn_market_2022::good::consts::{DEFAULT_EUR_USD_EXCHANGE_RATE, DEFAULT_EUR_YEN_EXCHANGE_RATE, DEFAULT_EUR_YUAN_EXCHANGE_RATE, DEFAULT_GOOD_KIND, STARTING_CAPITAL};
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::good::good_kind::GoodKind::{EUR, USD, YEN, YUAN};
use unitn_market_2022::market::*;
use unitn_market_2022::market::{LockBuyError, LockSellError};
use unitn_market_2022::market::good_label::GoodLabel;
use uuid::Uuid;

use doge_common::account::Account;

use crate::account_ops::AccountOps;
use crate::buy_transaction::{BuyTxProposal, BuyTxState};
use crate::logger::Logger;
use crate::market::{DogeBuyError, DogeBuyReservationError, DogeGetBuyPriceError, DogeGetSellPriceError, DogeMarketImpl, DogeSellError, DogeSellReservationError};
use crate::mute_logger::MuteLogger;
use crate::sell_transaction::{SellTxProposal, SellTxState};
use crate::txt_file_logger::TxtFileLogger;

pub struct DogeMarket {
    doge_impl: DogeMarketImpl,
    subscribers: Vec<Box<(dyn Notifiable + 'static)>>,
    logger: Box<dyn Logger>,
}

impl DogeMarket {
    fn new_with_goods(eur: &Good, yen: &Good, usd: &Good, yuan: &Good) -> Rc<RefCell<dyn Market>> where Self: Sized {
        let assets = Account { eur: eur.clone(), usd: usd.clone(), yen: yen.clone(), yuan: yuan.clone() };
        let ops = AccountOps::of_assets(assets);

        let doge_impl = DogeMarketImpl::new(ops, 10);

        // Fixme: Why does this not compile?
        // let mut logger: Box<dyn Logger> = Box::new(TxtFileLogger::try_new("DogeMarket").unwrap_or(MuteLogger::new()));

        let mut logger: Box<dyn Logger> = match TxtFileLogger::try_new("DogeMarket") {
            Some(txt_file_logger) => {
                Box::new(txt_file_logger)
            }
            None => {
                Box::new(MuteLogger::new())
            }
        };

        logger.log_initialization(eur, usd, yen, yuan);

        Rc::new(RefCell::new(Self {
            doge_impl,
            subscribers: vec![],
            logger,
        }))
    }
}

impl Notifiable for DogeMarket {
    fn add_subscriber(&mut self, subscriber: Box<(dyn Notifiable + 'static)>) {
        self.subscribers.push(subscriber);
    }

    fn on_event(&mut self, _event: Event) {
        self.doge_impl.advance_a_day();
    }
}

impl Market for DogeMarket {
    fn new_random() -> Rc<RefCell<dyn Market>> where Self: Sized {
        let quantities = new_random_quantities();
        DogeMarket::new_with_goods(&quantities[0], &quantities[1], &quantities[2], &quantities[3])
    }


    fn new_with_quantities(eur: f32, yen: f32, usd: f32, yuan: f32) -> Rc<RefCell<dyn Market>> where Self: Sized {
        let eur = Good::new(EUR, eur);
        let usd = Good::new(USD, usd);
        let yen = Good::new(YEN, yen);
        let yuan = Good::new(YUAN, yuan);

        DogeMarket::new_with_goods(&eur, &yen, &usd, &yuan)
    }

    fn new_file(_path: &str) -> Rc<RefCell<dyn Market>> where Self: Sized {
        unimplemented!()
    }

    fn get_name(&self) -> &'static str {
        self.doge_impl.market_name
    }

    fn get_budget(&self) -> f32 {
        self.doge_impl.get_tx_service()
            .get_account_ops()
            .get_reservable_quantity_by_kind(DEFAULT_GOOD_KIND)
    }

    fn get_buy_price(&self, kind: GoodKind, quantity: f32) -> Result<f32, MarketGetterError> {
        self.doge_impl.get_buy_price(kind, quantity)
            .map_err(|err| match err {
                DogeGetBuyPriceError::NonPositiveRequest =>
                    MarketGetterError::NonPositiveQuantityAsked,
                DogeGetBuyPriceError::ExceedsReservableQuantity { reservable } =>
                    MarketGetterError::InsufficientGoodQuantityAvailable { requested_good_kind: kind, requested_good_quantity: quantity, available_good_quantity: reservable }
            })
    }

    fn get_sell_price(&self, kind: GoodKind, quantity: f32) -> Result<f32, MarketGetterError> {
        self.doge_impl.get_sell_price(kind, quantity)
            .map_err(|err| match err {
                DogeGetSellPriceError::NonPositiveRequest => MarketGetterError::NonPositiveQuantityAsked,
            })
    }

    fn get_goods(&self) -> Vec<GoodLabel> {
        [self.doge_impl.make_label_for_kind(EUR),
            self.doge_impl.make_label_for_kind(USD),
            self.doge_impl.make_label_for_kind(YEN),
            self.doge_impl.make_label_for_kind(YUAN)]
            .into_iter()
            .collect()
    }


    fn lock_buy(&mut self, kind_to_buy: GoodKind, quantity_to_buy: f32, bid: f32, trader_name: String) -> Result<String, LockBuyError> {
        let proposal = BuyTxProposal { buy: Good::new(kind_to_buy, quantity_to_buy), bid: Good::new(DEFAULT_GOOD_KIND, bid) };

        match self.doge_impl.do_buy_reservation(&proposal) {
            Ok(uuid) => {
                self.logger.log_lock_buy(&proposal.buy, &trader_name, &proposal.bid, Some(&uuid));

                self.doge_impl.advance_a_day();

                self.subscribers.iter_mut().for_each(|sub| {
                    let event = Event {
                        kind: LockedBuy,
                        good_kind: kind_to_buy,
                        quantity: quantity_to_buy,
                        price: bid,
                    };
                    sub.on_event(event);
                });

                Ok(uuid.to_string())
            }
            Err(err) => Err(match err {
                DogeBuyReservationError::NonPositiveBuy =>
                    LockBuyError::NonPositiveQuantityToBuy { negative_quantity_to_buy: quantity_to_buy },
                DogeBuyReservationError::NonPositiveBid =>
                    LockBuyError::NonPositiveBid { negative_bid: bid },
                DogeBuyReservationError::ExceedsReservableQuantity { reservable } =>
                    LockBuyError::InsufficientGoodQuantityAvailable { requested_good_kind: kind_to_buy, requested_good_quantity: quantity_to_buy, available_good_quantity: reservable },
                DogeBuyReservationError::BidTooLow { lowest } =>
                    LockBuyError::BidTooLow { requested_good_kind: kind_to_buy, requested_good_quantity: quantity_to_buy, low_bid: bid, lowest_acceptable_bid: lowest }
            }),
        }
    }

    fn buy(&mut self, token: String, cash: &mut Good) -> Result<Good, BuyError> {
        let uuid = if let Ok(uuid) = Uuid::from_str(&token) { uuid } else { return Err(BuyError::UnrecognizedToken { unrecognized_token: token }); };

        match self.doge_impl.buy(&uuid, cash) {
            Ok(bought) => {
                self.logger.log_buy(&uuid, true);

                self.doge_impl.advance_a_day();

                let tx = self.doge_impl.get_tx_service().get_buy(&uuid).unwrap();

                self.subscribers.iter_mut().for_each(|sub| {
                    let event = Event {
                        kind: Bought,
                        good_kind: tx.buy.get_kind(),
                        quantity: tx.buy.get_qty(),
                        price: tx.bid.get_qty(),
                    };
                    sub.on_event(event);
                });

                Ok(bought)
            }
            Err(err) => {
                self.logger.log_buy(&uuid, false);

                Err(match err {
                    DogeBuyError::UnrecognizedUuid => BuyError::UnrecognizedToken { unrecognized_token: token },
                    DogeBuyError::InvalidState { current_state } => match current_state {
                        BuyTxState::Reserved => unreachable!(),
                        BuyTxState::Paid => BuyError::UnrecognizedToken { unrecognized_token: token },
                        BuyTxState::Expired => BuyError::ExpiredToken { expired_token: token }
                    },
                    DogeBuyError::WrongGoodKind { .. } => BuyError::GoodKindNotDefault { non_default_good_kind: cash.get_kind() },
                    DogeBuyError::InsufficientGoodQuantity { pre_agreed } => BuyError::InsufficientGoodQuantity { contained_quantity: cash.get_qty(), pre_agreed_quantity: pre_agreed }
                })
            }
        }
    }

    fn lock_sell(&mut self, kind_to_sell: GoodKind, quantity_to_sell: f32, offer: f32, trader_name: String) -> Result<String, LockSellError> {
        let proposal = SellTxProposal { sell: Good::new(kind_to_sell, quantity_to_sell), offer: Good::new(DEFAULT_GOOD_KIND, offer) };

        match self.doge_impl.do_sell_reservation(&proposal) {
            Ok(uuid) => {
                self.logger.log_lock_sell(&proposal.sell, &trader_name, &proposal.offer, Some(&uuid));

                self.doge_impl.advance_a_day();

                self.subscribers.iter_mut().for_each(|sub| {
                    let event = Event {
                        kind: LockedSell,
                        good_kind: kind_to_sell,
                        quantity: quantity_to_sell,
                        price: offer,
                    };
                    sub.on_event(event);
                });

                Ok(uuid.to_string())
            }
            Err(err) => Err(match err {
                DogeSellReservationError::NonPositiveSell =>
                    LockSellError::NonPositiveQuantityToSell { negative_quantity_to_sell: quantity_to_sell },
                DogeSellReservationError::NonPositiveOffer =>
                    LockSellError::NonPositiveOffer { negative_offer: offer },
                DogeSellReservationError::ExceedsReservableQuantity { reservable } =>
                    LockSellError::InsufficientDefaultGoodQuantityAvailable { offered_good_kind: kind_to_sell, offered_good_quantity: quantity_to_sell, available_good_quantity: reservable },
                DogeSellReservationError::OfferTooHigh { highest } =>
                    LockSellError::OfferTooHigh { offered_good_kind: kind_to_sell, offered_good_quantity: quantity_to_sell, high_offer: offer, highest_acceptable_offer: highest }
            }),
        }
    }

    fn sell(&mut self, token: String, good: &mut Good) -> Result<Good, SellError> {
        let uuid = if let Ok(uuid) = Uuid::from_str(&token) { uuid } else { return Err(SellError::UnrecognizedToken { unrecognized_token: token }); };

        match self.doge_impl.sell(&uuid, good) {
            Ok(sold) => {
                self.logger.log_sell(&uuid, true);

                self.doge_impl.advance_a_day();

                let tx = self.doge_impl.get_tx_service().get_sell(&uuid).unwrap();

                self.subscribers.iter_mut().for_each(|sub| {
                    let event = Event {
                        kind: Sold,
                        good_kind: tx.sell.get_kind(),
                        quantity: tx.sell.get_qty(),
                        price: tx.offer.get_qty(),
                    };
                    sub.on_event(event);
                });

                Ok(sold)
            }
            Err(err) => {
                self.logger.log_sell(&uuid, false);

                Err(match err {
                    DogeSellError::UnrecognizedUuid => SellError::UnrecognizedToken { unrecognized_token: token },
                    DogeSellError::InvalidState { current_state } => match current_state {
                        SellTxState::Reserved => unreachable!(),
                        SellTxState::Paid => SellError::UnrecognizedToken { unrecognized_token: token },
                        SellTxState::Expired => SellError::ExpiredToken { expired_token: token }
                    }
                    DogeSellError::WrongGoodKind { pre_agreed } => SellError::WrongGoodKind { wrong_good_kind: good.get_kind(), pre_agreed_kind: pre_agreed },
                    DogeSellError::InsufficientGoodQuantity { pre_agreed } => SellError::InsufficientGoodQuantity { contained_quantity: good.get_qty(), pre_agreed_quantity: pre_agreed },
                })
            }
        }
    }
}

fn new_random_quantities() -> [Good; 4] {
    const EUR_TO_USD_RATE: f32 = DEFAULT_EUR_USD_EXCHANGE_RATE;
    const USD_TO_EUR_RATE: f32 = 1. / EUR_TO_USD_RATE;
    const EUR_TO_YEN_RATE: f32 = DEFAULT_EUR_YEN_EXCHANGE_RATE;
    const YEN_TO_EUR_RATE: f32 = 1. / EUR_TO_YEN_RATE;
    const EUR_TO_YUAN_RATE: f32 = DEFAULT_EUR_YUAN_EXCHANGE_RATE;
    const YUAN_TO_EUR_RATE: f32 = 1. / EUR_TO_YUAN_RATE;

    const EARNING_RATE: f32 = 101. / 100.;

    // compute starting values
    const EUR_BUDGET_WITHOUT_ERROR: f32 = (STARTING_CAPITAL - USD_TO_EUR_RATE - YEN_TO_EUR_RATE - YUAN_TO_EUR_RATE) / (1. + 3. * EARNING_RATE);
    const USD_BUDGET_WITHOUT_ERROR: f32 = EUR_BUDGET_WITHOUT_ERROR * EARNING_RATE * EUR_TO_USD_RATE + 1.;
    const YEN_BUDGET_WITHOUT_ERROR: f32 = EUR_BUDGET_WITHOUT_ERROR * EARNING_RATE * EUR_TO_YEN_RATE + 1.;
    const YUAN_BUDGET_WITHOUT_ERROR: f32 = EUR_BUDGET_WITHOUT_ERROR * EARNING_RATE * EUR_TO_YUAN_RATE + 1.;

    // remove error from f32 approximation
    const EUR_BUDGET: f32 = EUR_BUDGET_WITHOUT_ERROR - EUR_BUDGET_WITHOUT_ERROR * 1. / 10_000_000.;
    const USD_BUDGET: f32 = USD_BUDGET_WITHOUT_ERROR - USD_BUDGET_WITHOUT_ERROR * 1. / 10_000_000.;
    const YEN_BUDGET: f32 = YEN_BUDGET_WITHOUT_ERROR - YEN_BUDGET_WITHOUT_ERROR * 1. / 10_000_000.;
    const YUAN_BUDGET: f32 = YUAN_BUDGET_WITHOUT_ERROR - YUAN_BUDGET_WITHOUT_ERROR * 1. / 10_000_000.;

    [Good::new(EUR, EUR_BUDGET), Good::new(YEN, YEN_BUDGET), Good::new(USD, USD_BUDGET), Good::new(YUAN, YUAN_BUDGET)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_starting_capital() {
        let market = DogeMarket::new_random();
        let sum: f32 = market.borrow().get_goods().into_iter()
            .map(|label| match label.good_kind {
                DEFAULT_GOOD_KIND => label.quantity,
                _ => label.quantity / label.good_kind.get_default_exchange_rate()
            })
            .sum();
        assert!(sum <= 1_000_000.);
    }
}
