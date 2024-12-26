use std::collections::HashMap;

use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;
use uuid::Uuid;

use crate::account_ops::AccountOps;
use crate::buy_transaction::{BuyTx, BuyTxPaymentError, BuyTxProposal, BuyTxReservationError, BuyTxState};
use crate::sell_transaction::{SellTx, SellTxPaymentError, SellTxProposal, SellTxReservationError, SellTxState};
use crate::tick_deque::TickDeque;

pub struct TxService {
    ops: AccountOps,
    buys: HashMap<Uuid, BuyTx>,
    sells: HashMap<Uuid, SellTx>,
    deque: TickDeque<Uuid>,
}

#[derive(Debug)]
pub enum ServiceBuyReservationError {
    NonPositiveBuy,
    NonPositiveBid,
    ExceedsReservableQuantity { reservable: f32 },
    BidTooLow { lowest: f32 },
}

#[derive(Debug)]
pub enum ServiceSellReservationError {
    NonPositiveSell,
    NonPositiveOffer,
    ExceedsReservableQuantity { reservable: f32 },
    OfferTooHigh { highest: f32 },
}

#[derive(Debug)]
pub enum ServiceBuyError {
    UnrecognizedUuid,
    InvalidState { current_state: BuyTxState },
    WrongGoodKind { pre_agreed: GoodKind },
    InsufficientGoodQuantity { pre_agreed: f32 },
}

#[derive(Debug)]
pub enum ServiceSellError {
    UnrecognizedUuid,
    InvalidState { current_state: SellTxState },
    WrongGoodKind { pre_agreed: GoodKind },
    InsufficientGoodQuantity { pre_agreed: f32 },
}

impl TxService {
    pub fn new(ops: AccountOps, max_ticks: u32) -> TxService {
        TxService {
            ops,
            buys: HashMap::new(),
            sells: HashMap::new(),
            deque: TickDeque::new(max_ticks),
        }
    }

    pub fn get_account_ops(&self) -> &AccountOps {
        &self.ops
    }

    pub fn get_account_ops_mut(&mut self) -> &mut AccountOps {
        &mut self.ops
    }

    pub fn do_buy_reservation(&mut self, proposal: &BuyTxProposal) -> Result<Uuid, ServiceBuyReservationError> {
        match BuyTx::reserve(&mut self.ops, proposal) {
            Ok(reservation) => {
                let uuid = Uuid::new_v4();
                self.buys.insert(uuid, reservation);
                self.deque.push_back(uuid);
                Ok(uuid)
            }
            Err(err) => match err {
                BuyTxReservationError::NonPositiveBuy => Err(ServiceBuyReservationError::NonPositiveBuy),
                BuyTxReservationError::NonPositiveBid => Err(ServiceBuyReservationError::NonPositiveBid),
                BuyTxReservationError::ExceedsReservableQuantity { reservable } => Err(ServiceBuyReservationError::ExceedsReservableQuantity { reservable }),
                BuyTxReservationError::BidTooLow { lowest } => Err(ServiceBuyReservationError::BidTooLow { lowest })
            }
        }
    }

    pub fn do_sell_reservation(&mut self, proposal: &SellTxProposal) -> Result<Uuid, ServiceSellReservationError> {
        match SellTx::reserve(&mut self.ops, proposal) {
            Ok(reservation) => {
                let uuid = Uuid::new_v4();
                self.sells.insert(uuid, reservation);
                self.deque.push_back(uuid);
                Ok(uuid)
            }
            Err(err) => match err {
                SellTxReservationError::NonPositiveSell => Err(ServiceSellReservationError::NonPositiveSell),
                SellTxReservationError::NonPositiveOffer => Err(ServiceSellReservationError::NonPositiveOffer),
                SellTxReservationError::ExceedsReservableQuantity { reservable } => Err(ServiceSellReservationError::ExceedsReservableQuantity { reservable }),
                SellTxReservationError::OfferTooHigh { highest } => Err(ServiceSellReservationError::OfferTooHigh { highest }),
            }
        }
    }

    pub fn do_buy(&mut self, uuid: &Uuid, with: &mut Good) -> Result<Good, ServiceBuyError> {
        if let Some(tx) = self.buys.get_mut(uuid) {
            tx.buy(&mut self.ops, with).map_err(|err| match err {
                BuyTxPaymentError::InvalidState { current_state } => ServiceBuyError::InvalidState { current_state },
                BuyTxPaymentError::WrongGoodKind { pre_agreed } => ServiceBuyError::WrongGoodKind { pre_agreed },
                BuyTxPaymentError::InsufficientGoodQuantity { pre_agreed } => ServiceBuyError::InsufficientGoodQuantity { pre_agreed }
            })
        } else {
            Err(ServiceBuyError::UnrecognizedUuid)
        }
    }

    pub fn do_sell(&mut self, uuid: &Uuid, with: &mut Good) -> Result<Good, ServiceSellError> {
        if let Some(tx) = self.sells.get_mut(uuid) {
            tx.sell(&mut self.ops, with).map_err(|err| match err {
                SellTxPaymentError::InvalidState { current_state } => ServiceSellError::InvalidState { current_state },
                SellTxPaymentError::WrongGoodKind { pre_agreed } => ServiceSellError::WrongGoodKind { pre_agreed },
                SellTxPaymentError::InsufficientGoodQuantity { pre_agreed } => ServiceSellError::InsufficientGoodQuantity { pre_agreed }
            })
        } else {
            Err(ServiceSellError::UnrecognizedUuid)
        }
    }

    pub fn get_buy(&self, uuid: &Uuid) -> Option<&BuyTx> {
        self.buys.get(uuid)
    }

    pub fn get_sell(&self, uuid: &Uuid) -> Option<&SellTx> {
        self.sells.get(uuid)
    }

    pub fn tick_all(&mut self) {
        self.deque.tick().into_iter()
            .for_each(|uuid| {
                if let Some(tx) = self.buys.get_mut(&uuid) {
                    tx.expire(&mut self.ops);
                } else if let Some(tx) = self.sells.get_mut(&uuid) {
                    tx.expire(&mut self.ops);
                } else {
                    unreachable!()
                }
            });
    }
}
