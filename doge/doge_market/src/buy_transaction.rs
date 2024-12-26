use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;

use doge_common::account::WithdrawError;

use crate::account_ops::{AccountOps, BuyPriceComputationError};

#[derive(Debug)]
pub struct BuyTx {
    pub buy: Good,
    pub bid: Good,
    pub state: BuyTxState,
}

#[derive(Debug, Clone)]
pub enum BuyTxState { Reserved, Paid, Expired }

#[derive(Debug)]
pub struct BuyTxProposal {
    pub buy: Good,
    pub bid: Good,
}

#[derive(Debug, Clone)]
pub enum BuyTxReservationError {
    NonPositiveBuy,
    NonPositiveBid,
    ExceedsReservableQuantity { reservable: f32 },
    BidTooLow { lowest: f32 },
}

#[derive(Debug, Clone)]
pub enum BuyTxPaymentError {
    InvalidState { current_state: BuyTxState },
    WrongGoodKind { pre_agreed: GoodKind },
    InsufficientGoodQuantity { pre_agreed: f32 },
}

#[derive(Debug, Clone)]
pub struct ExpireError;

impl BuyTx {
    pub fn reserve(ops: &mut AccountOps, proposal: &BuyTxProposal) -> Result<BuyTx, BuyTxReservationError> {
        if proposal.buy.get_qty() <= 0. {
            Err(BuyTxReservationError::NonPositiveBuy)
        } else if proposal.bid.get_qty() <= 0. {
            Err(BuyTxReservationError::NonPositiveBid)
        } else if proposal.buy.get_qty() > ops.get_reservable_quantity_by_kind(proposal.buy.get_kind()) {
            Err(BuyTxReservationError::ExceedsReservableQuantity { reservable: ops.get_reservable_quantity_by_kind(proposal.buy.get_kind()) })
        } else {
            match ops.compute_buy_price(proposal.buy.get_kind(), proposal.buy.get_qty(), if let DEFAULT_GOOD_KIND = proposal.buy.get_kind() { 0. } else { 1. }) {
                Ok(buy_price) => {
                    if proposal.bid.get_qty() < buy_price {
                        Err(BuyTxReservationError::BidTooLow { lowest: buy_price })
                    } else {
                        ops.assets.withdraw(proposal.buy.get_kind(), proposal.buy.get_qty()).unwrap();
                        ops.reservations.deposit(proposal.buy.clone());
                        ops.futures.deposit(proposal.bid.clone());

                        Ok(BuyTx { buy: proposal.buy.clone(), bid: proposal.bid.clone(), state: BuyTxState::Reserved })
                    }
                }
                Err(err) => {
                    match err {
                        BuyPriceComputationError::NonPositiveQuantity => unreachable!(),
                        BuyPriceComputationError::NegativeExchangeRateEarnPercentage => unreachable!(),
                        BuyPriceComputationError::ExceedsReservableQuantity { reservable } =>
                            Err(BuyTxReservationError::ExceedsReservableQuantity { reservable })
                    }
                }
            }
        }
    }

    pub fn buy(&mut self, ops: &mut AccountOps, with: &mut Good) -> Result<Good, BuyTxPaymentError> {
        match &self.state {
            BuyTxState::Reserved => {
                if with.get_kind() != self.bid.get_kind() {
                    Err(BuyTxPaymentError::WrongGoodKind { pre_agreed: self.bid.get_kind() })
                } else if with.get_qty() < self.bid.get_qty() {
                    Err(BuyTxPaymentError::InsufficientGoodQuantity { pre_agreed: self.bid.get_qty() })
                } else {
                    let from_buyer = with.split(self.bid.get_qty()).unwrap();
                    ops.assets.deposit(from_buyer);

                    let from_us = match ops.reservations.withdraw(self.buy.get_kind(), self.buy.get_qty()) {
                        Ok(good) => good,
                        Err(err) => match err {
                            WithdrawError::NonPositiveWithdrawQuantity => unreachable!(),
                            WithdrawError::WithdrawExcessiveQuantity { withdrawable } => {
                                // Reached 0 quantity, but went a little below 0 due to approximation errors
                                if withdrawable > 0. {
                                    ops.reservations.withdraw(self.buy.get_kind(), withdrawable).unwrap();
                                }
                                Good::new(self.buy.get_kind(), self.buy.get_qty())
                            }
                        }
                    };

                    if let Err(WithdrawError::WithdrawExcessiveQuantity { withdrawable }) = ops.futures.withdraw(self.bid.get_kind(), self.bid.get_qty()) {
                        // Reached 0 quantity, but went a little below 0 due to approximation errors
                        if withdrawable > 0. {
                            ops.futures.withdraw(self.bid.get_kind(), withdrawable).unwrap();
                        }
                    };

                    self.state = BuyTxState::Paid;

                    Ok(from_us)
                }
            }
            other => Err(BuyTxPaymentError::InvalidState { current_state: other.clone() })
        }
    }

    pub fn expire(&mut self, ops: &mut AccountOps) {
        if let BuyTxState::Reserved = self.state {
            let reservation = match ops.reservations.withdraw(self.buy.get_kind(), self.buy.get_qty()) {
                Ok(good) => good,
                Err(err) => match err {
                    WithdrawError::NonPositiveWithdrawQuantity => unreachable!(),
                    WithdrawError::WithdrawExcessiveQuantity { withdrawable } => {
                        // Reached 0 quantity, but went a little below 0 due to approximation errors
                        if withdrawable > 0. {
                            ops.reservations.withdraw(self.buy.get_kind(), withdrawable).unwrap();
                        }
                        Good::new(self.buy.get_kind(), self.buy.get_qty())
                    }
                }
            };

            ops.assets.deposit(reservation);

            if let Err(WithdrawError::WithdrawExcessiveQuantity { withdrawable }) = ops.futures.withdraw(self.bid.get_kind(), self.bid.get_qty()) {
                // Reached 0 quantity, but went a little below 0 due to approximation errors
                if withdrawable > 0. {
                    ops.futures.withdraw(self.bid.get_kind(), withdrawable).unwrap();
                }
            }

            self.state = BuyTxState::Expired;
        }
    }
}
