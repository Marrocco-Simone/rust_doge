use std::convert::identity;

use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::market::good_label::GoodLabel;
use uuid::Uuid;

use crate::account_ops::{AccountOps, BuyExchangeRateComputationError, BuyPriceComputationError, SellExchangeRateComputationError, SellPriceComputationError};
use crate::buy_transaction::{BuyTxProposal, BuyTxState};
use crate::refiller::GoodRefiller;
use crate::sell_transaction::{SellTxProposal, SellTxState};
use crate::service::{ServiceBuyError, ServiceBuyReservationError, ServiceSellError, ServiceSellReservationError, TxService};

pub struct DogeMarketImpl {
    pub market_name: &'static str,
    service: TxService,
    refiller: GoodRefiller,
}

#[derive(Debug)]
pub enum DogeBuyReservationError {
    NonPositiveBuy,
    NonPositiveBid,
    ExceedsReservableQuantity { reservable: f32 },
    BidTooLow { lowest: f32 },
}

#[derive(Debug)]
pub enum DogeSellReservationError {
    NonPositiveSell,
    NonPositiveOffer,
    ExceedsReservableQuantity { reservable: f32 },
    OfferTooHigh { highest: f32 },
}

#[derive(Debug)]
pub enum DogeBuyError {
    UnrecognizedUuid,
    InvalidState { current_state: BuyTxState },
    WrongGoodKind { pre_agreed: GoodKind },
    InsufficientGoodQuantity { pre_agreed: f32 },
}

#[derive(Debug)]
pub enum DogeSellError {
    UnrecognizedUuid,
    InvalidState { current_state: SellTxState },
    WrongGoodKind { pre_agreed: GoodKind },
    InsufficientGoodQuantity { pre_agreed: f32 },
}

#[derive(Debug)]
pub enum DogeGetBuyPriceError {
    NonPositiveRequest,
    ExceedsReservableQuantity { reservable: f32 },
}

#[derive(Debug)]
pub enum DogeGetSellPriceError {
    NonPositiveRequest,
}

impl DogeMarketImpl {
    pub fn new(ops: AccountOps, max_ticks: u32) -> DogeMarketImpl {
        DogeMarketImpl {
            market_name: "DogeMarket",
            service: TxService::new(ops, max_ticks),
            refiller: GoodRefiller::new(),
        }
    }

    pub fn do_buy_reservation(&mut self, proposal: &BuyTxProposal) -> Result<Uuid, DogeBuyReservationError> {
        self.service.do_buy_reservation(proposal).map_err(|err| match err {
            ServiceBuyReservationError::NonPositiveBuy => DogeBuyReservationError::NonPositiveBuy,
            ServiceBuyReservationError::NonPositiveBid => DogeBuyReservationError::NonPositiveBid,
            ServiceBuyReservationError::ExceedsReservableQuantity { reservable } => DogeBuyReservationError::ExceedsReservableQuantity { reservable },
            ServiceBuyReservationError::BidTooLow { lowest } => DogeBuyReservationError::BidTooLow { lowest }
        })
    }

    pub fn do_sell_reservation(&mut self, proposal: &SellTxProposal) -> Result<Uuid, DogeSellReservationError> {
        self.service.do_sell_reservation(proposal).map_err(|err| match err {
            ServiceSellReservationError::NonPositiveSell => DogeSellReservationError::NonPositiveSell,
            ServiceSellReservationError::NonPositiveOffer => DogeSellReservationError::NonPositiveOffer,
            ServiceSellReservationError::ExceedsReservableQuantity { reservable } => DogeSellReservationError::ExceedsReservableQuantity { reservable },
            ServiceSellReservationError::OfferTooHigh { highest } => DogeSellReservationError::OfferTooHigh { highest },
        })
    }

    pub fn buy(&mut self, uuid: &Uuid, cash: &mut Good) -> Result<Good, DogeBuyError> {
        self.service.do_buy(uuid, cash).map_err(|err| match err {
            ServiceBuyError::UnrecognizedUuid => DogeBuyError::UnrecognizedUuid,
            ServiceBuyError::InvalidState { current_state } => DogeBuyError::InvalidState { current_state },
            ServiceBuyError::WrongGoodKind { pre_agreed } => DogeBuyError::WrongGoodKind { pre_agreed },
            ServiceBuyError::InsufficientGoodQuantity { pre_agreed } => DogeBuyError::InsufficientGoodQuantity { pre_agreed }
        })
    }

    pub fn sell(&mut self, uuid: &Uuid, good: &mut Good) -> Result<Good, DogeSellError> {
        self.service.do_sell(uuid, good).map_err(|err| match err {
            ServiceSellError::UnrecognizedUuid => DogeSellError::UnrecognizedUuid,
            ServiceSellError::InvalidState { current_state } => DogeSellError::InvalidState { current_state },
            ServiceSellError::WrongGoodKind { pre_agreed } => DogeSellError::WrongGoodKind { pre_agreed },
            ServiceSellError::InsufficientGoodQuantity { pre_agreed } => DogeSellError::InsufficientGoodQuantity { pre_agreed }
        })
    }

    pub fn get_tx_service(&self) -> &TxService {
        &self.service
    }

    pub fn advance_a_day(&mut self) {
        self.service.tick_all();
        let ops = self.service.get_account_ops_mut();
        self.refiller.refill_goods(&mut ops.assets, &ops.reservations);
    }

    pub fn get_buy_price(&self, of_kind: GoodKind, of_quantity: f32) -> Result<f32, DogeGetBuyPriceError> {
        self.get_tx_service()
            .get_account_ops()
            .compute_buy_price(of_kind, of_quantity, if let DEFAULT_GOOD_KIND = of_kind { 0. } else { 1. })
            .map_err(|err| match err {
                BuyPriceComputationError::NonPositiveQuantity =>
                    DogeGetBuyPriceError::NonPositiveRequest,
                BuyPriceComputationError::NegativeExchangeRateEarnPercentage => unreachable!(),
                BuyPriceComputationError::ExceedsReservableQuantity { reservable } =>
                    DogeGetBuyPriceError::ExceedsReservableQuantity { reservable }
            })
    }

    pub fn get_sell_price(&self, of_kind: GoodKind, of_quantity: f32) -> Result<f32, DogeGetSellPriceError> {
        self.get_tx_service()
            .get_account_ops()
            .compute_sell_price(of_kind, of_quantity, if let DEFAULT_GOOD_KIND = of_kind { 0. } else { 1. })
            .map_err(|err| match err {
                SellPriceComputationError::NonPositiveQuantity =>
                    DogeGetSellPriceError::NonPositiveRequest,
                SellPriceComputationError::NegativeExchangeEarnRatePercentage => unreachable!()
            })
    }

    pub fn make_label_for_kind(&self, kind: GoodKind) -> GoodLabel {
        let ops = self.get_tx_service().get_account_ops();

        let exchange_rate_buy = ops.compute_buy_exchange_rate(kind, if let DEFAULT_GOOD_KIND = kind { 0. } else { 1. })
            .map_or_else(|err| match err {
                BuyExchangeRateComputationError::NonPositiveExchangeRateEarnPercentage => unreachable!(),
                BuyExchangeRateComputationError::ExceedsReservableQuantity { .. } => f32::MAX
            }, &identity);

        let exchange_rate_sell = ops.compute_sell_exchange_rate(kind, if let DEFAULT_GOOD_KIND = kind { 0. } else { 1. })
            .map_or_else(|err| match err {
                SellExchangeRateComputationError::NonPositiveExchangeRateEarnPercentage => unreachable!()
            }, &identity);

        GoodLabel { good_kind: kind, quantity: ops.get_reservable_quantity_by_kind(kind), exchange_rate_buy, exchange_rate_sell }
    }
}

#[cfg(test)]
mod tests {
    use unitn_market_2022::good::good_kind::GoodKind::{EUR, USD, YEN, YUAN};

    use doge_common::account::Account;

    use super::*;

    //checks what happens when a good contains max quantity and we add to it
    //result: the quantity becomes infinite
    #[test]
    fn test_maximum_capacity() {
        let assets = Account {
            eur: Good::new(EUR, f32::MAX),
            usd: Good::new(USD, 500_000.),
            yen: Good::new(YEN, 500_000.),
            yuan: Good::new(YUAN, 500_000.),
        };

        let mut market = DogeMarketImpl::new(AccountOps::of_assets(assets), 10);

        let buy_price = market.get_buy_price(USD, 50.).unwrap();

        let proposal = BuyTxProposal {
            buy: Good::new(USD, 50.),
            bid: Good::new(EUR, buy_price),
        };

        let uuid = market.do_buy_reservation(&proposal).unwrap();

        market.buy(&uuid, &mut Good::new(EUR, buy_price)).unwrap();

        assert_eq!(market.make_label_for_kind(EUR).quantity, f32::INFINITY);
    }

    //checks what happens when we try to buy all quantity of a good:
    //result: buy price is inf, if we try to lock buy with inf the program crashes
    #[test]
    fn test_buy_everything() {
        let assets = Account {
            eur: Good::new(EUR, 500_000.),
            usd: Good::new(USD, 500_000.),
            yen: Good::new(YEN, 500_000.),
            yuan: Good::new(YUAN, 500_000.),
        };

        let mut market = DogeMarketImpl::new(AccountOps::of_assets(assets), 10);

        let buy_price = market.get_buy_price(YEN, 500_000.).unwrap();

        let proposal = BuyTxProposal {
            buy: Good::new(YEN, 500_000.),
            bid: Good::new(EUR, buy_price),
        };

        let uuid = market.do_buy_reservation(&proposal).unwrap();

        market.buy(&uuid, &mut Good::new(EUR, buy_price)).unwrap();

        market.get_buy_price(USD, 1.).unwrap(); // this fails
    }
}
