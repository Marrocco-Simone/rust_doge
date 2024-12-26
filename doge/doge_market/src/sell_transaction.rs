use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;

use doge_common::account::WithdrawError;

use crate::account_ops::{AccountOps, SellPriceComputationError};

#[derive(Debug)]
pub struct SellTx {
    pub sell: Good,
    pub offer: Good,
    pub state: SellTxState,
}

#[derive(Debug, Clone)]
pub enum SellTxState { Reserved, Paid, Expired }

#[derive(Debug)]
pub struct SellTxProposal {
    pub sell: Good,
    pub offer: Good,
}

#[derive(Debug, Clone)]
pub enum SellTxReservationError {
    NonPositiveSell,
    NonPositiveOffer,
    ExceedsReservableQuantity { reservable: f32 },
    OfferTooHigh { highest: f32 },
}

#[derive(Debug, Clone)]
pub enum SellTxPaymentError {
    InvalidState { current_state: SellTxState },
    WrongGoodKind { pre_agreed: GoodKind },
    InsufficientGoodQuantity { pre_agreed: f32 },
}

#[derive(Debug, Clone)]
pub struct ExpireError;

impl SellTx {
    pub fn reserve(ops: &mut AccountOps, proposal: &SellTxProposal) -> Result<SellTx, SellTxReservationError> {
        if proposal.sell.get_qty() <= 0. {
            Err(SellTxReservationError::NonPositiveSell)
        } else if proposal.offer.get_qty() <= 0. {
            Err(SellTxReservationError::NonPositiveOffer)
        } else if proposal.offer.get_qty() > ops.get_reservable_quantity_by_kind(proposal.offer.get_kind()) {
            Err(SellTxReservationError::ExceedsReservableQuantity { reservable: ops.get_reservable_quantity_by_kind(proposal.offer.get_kind()) })
        } else {
            match ops.compute_sell_price(proposal.sell.get_kind(), proposal.sell.get_qty(), if let DEFAULT_GOOD_KIND = proposal.sell.get_kind() { 0. } else { 1. }) {
                Ok(sell_price) => {
                    if proposal.offer.get_qty() > sell_price {
                        Err(SellTxReservationError::OfferTooHigh { highest: sell_price })
                    } else {
                        ops.assets.withdraw(proposal.offer.get_kind(), proposal.offer.get_qty()).unwrap();
                        ops.reservations.deposit(proposal.offer.clone());
                        ops.futures.deposit(proposal.sell.clone());

                        Ok(SellTx { sell: proposal.sell.clone(), offer: proposal.offer.clone(), state: SellTxState::Reserved })
                    }
                }
                Err(err) => match err {
                    SellPriceComputationError::NegativeExchangeEarnRatePercentage => unreachable!(),
                    SellPriceComputationError::NonPositiveQuantity => unreachable!()
                }
            }
        }
    }

    pub fn sell(&mut self, ops: &mut AccountOps, with: &mut Good) -> Result<Good, SellTxPaymentError> {
        match &self.state {
            SellTxState::Reserved => {
                if with.get_kind() != self.sell.get_kind() {
                    Err(SellTxPaymentError::WrongGoodKind { pre_agreed: self.sell.get_kind() })
                } else if with.get_qty() < self.sell.get_qty() {
                    Err(SellTxPaymentError::InsufficientGoodQuantity { pre_agreed: self.sell.get_qty() })
                } else {
                    let from_seller = with.split(self.sell.get_qty()).unwrap();
                    ops.assets.deposit(from_seller);

                    let from_us = match ops.reservations.withdraw(self.offer.get_kind(), self.offer.get_qty()) {
                        Ok(good) => good,
                        Err(err) => match err {
                            WithdrawError::NonPositiveWithdrawQuantity => unreachable!(),
                            WithdrawError::WithdrawExcessiveQuantity { withdrawable } => {
                                // Reached 0 quantity, but went a little below 0 due to approximation errors
                                if withdrawable > 0. {
                                    ops.reservations.withdraw(self.offer.get_kind(), withdrawable).unwrap();
                                }
                                Good::new(self.offer.get_kind(), self.offer.get_qty())
                            }
                        }
                    };

                    if let Err(WithdrawError::WithdrawExcessiveQuantity { withdrawable }) = ops.futures.withdraw(self.sell.get_kind(), self.sell.get_qty()) {
                        // Reached 0 quantity, but went a little below 0 due to approximation errors
                        if withdrawable > 0. {
                            ops.futures.withdraw(self.sell.get_kind(), withdrawable).unwrap();
                        }
                    };

                    self.state = SellTxState::Paid;

                    Ok(from_us)
                }
            }
            other => Err(SellTxPaymentError::InvalidState { current_state: other.clone() })
        }
    }

    pub fn expire(&mut self, ops: &mut AccountOps) {
        if let SellTxState::Reserved = self.state {
            let reservation = match ops.reservations.withdraw(self.offer.get_kind(), self.offer.get_qty()) {
                Ok(good) => good,
                Err(err) => match err {
                    WithdrawError::NonPositiveWithdrawQuantity => unreachable!(),
                    WithdrawError::WithdrawExcessiveQuantity { withdrawable } => {
                        // Reached 0 quantity, but went a little below 0 due to approximation errors
                        if withdrawable > 0. {
                            ops.reservations.withdraw(self.offer.get_kind(), withdrawable).unwrap();
                        }
                        Good::new(self.offer.get_kind(), self.offer.get_qty())
                    }
                }
            };

            ops.assets.deposit(reservation);

            if let Err(WithdrawError::WithdrawExcessiveQuantity { withdrawable }) = ops.futures.withdraw(self.sell.get_kind(), self.sell.get_qty()) {
                // Reached 0 quantity, but went a little below 0 due to approximation errors
                if withdrawable > 0. {
                    ops.futures.withdraw(self.sell.get_kind(), withdrawable).unwrap();
                }
            };

            self.state = SellTxState::Expired;
        }
    }
}
