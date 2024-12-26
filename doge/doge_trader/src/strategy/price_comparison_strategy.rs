use std::{cell::RefCell, rc::Rc};

use doge_common::account::Account;
use unitn_market_2022::{good::good_kind::GoodKind, market::Market, wait_one_day};

use crate::{
    event_handler::EventHandler,
    iteraction_logger::create_account_graphs::{max_f32, min_f32},
    logger::stdout_logger::StdoutLogger,
    markets::MarketWrapper,
    strategy::common::{direct_buy, direct_sell},
};

use super::Strategy;

// TODO spostare in common.rs
pub fn get_buy_price(
    market: Rc<RefCell<dyn Market>>,
    quantity: f32,
    kind: GoodKind,
) -> Result<f32, String> {
    match market.borrow().get_buy_price(kind, quantity) {
        Ok(price) => Ok(price),
        Err(msg) => Err(format!("error in get buy price: {:#?}", msg)),
    }
}

// TODO spostare in common.rs
pub fn get_sell_price(
    market: Rc<RefCell<dyn Market>>,
    quantity: f32,
    kind: GoodKind,
) -> Result<f32, String> {
    match market.borrow().get_sell_price(kind, quantity) {
        Ok(price) => Ok(price),
        Err(msg) => Err(format!("error in get sell price: {:#?}", msg)),
    }
}

pub fn my_get_buy_price(market: Rc<RefCell<dyn Market>>, kind: GoodKind) -> f32 {
    get_buy_price(market, QUANTITY, kind).unwrap_or(IMPOSSIBLE_BUY_PRICE)
}
pub fn my_get_sell_price(market: Rc<RefCell<dyn Market>>, kind: GoodKind) -> f32 {
    get_sell_price(market, QUANTITY, kind).unwrap_or(IMPOSSIBLE_SELL_PRICE)
}

const QUANTITY: f32 = 10000.0;
const IMPOSSIBLE_BUY_PRICE: f32 = f32::INFINITY;
const IMPOSSIBLE_SELL_PRICE: f32 = -f32::INFINITY;

#[derive(Debug)]
struct GoodPriceTable {
    sol: f32,
    rcnz: f32,
    bose: f32,
}

#[derive(Debug)]
struct BuySellTable {
    buy: GoodPriceTable,
    sell: GoodPriceTable,
}

impl BuySellTable {
    fn new(markets: &mut MarketWrapper, kind: GoodKind) -> BuySellTable {
        BuySellTable {
            buy: GoodPriceTable {
                sol: my_get_buy_price(markets.sol.clone(), kind),
                rcnz: my_get_buy_price(markets.rcnz.clone(), kind),
                bose: my_get_buy_price(markets.bose.clone(), kind),
            },
            sell: GoodPriceTable {
                sol: my_get_sell_price(markets.sol.clone(), kind),
                rcnz: my_get_sell_price(markets.rcnz.clone(), kind),
                bose: my_get_sell_price(markets.bose.clone(), kind),
            },
        }
    }

    fn get_best_operation(&self, kind: GoodKind) -> BestOperation {
        let min_buy = min_f32(self.buy.sol, min_f32(self.buy.rcnz, self.buy.bose));
        let buy_from = if min_buy == self.buy.sol {
            MarketToChoose::Sol
        } else if min_buy == self.buy.rcnz {
            MarketToChoose::Rcnz
        } else {
            MarketToChoose::Bose
        };

        let max_sell = max_f32(self.sell.sol, max_f32(self.sell.rcnz, self.sell.bose));
        let sell_to = if max_sell == self.sell.sol {
            MarketToChoose::Sol
        } else if max_sell == self.sell.rcnz {
            MarketToChoose::Rcnz
        } else {
            MarketToChoose::Bose
        };

        BestOperation {
            kind,
            buy_from,
            sell_to,
            expected_gain: (max_sell - min_buy) / kind.get_default_exchange_rate(),
        }
    }
}

#[derive(Debug)]
struct PriceTable {
    eur: BuySellTable,
    usd: BuySellTable,
    yen: BuySellTable,
    yuan: BuySellTable,
}

impl PriceTable {
    fn new(markets: &mut MarketWrapper) -> PriceTable {
        PriceTable {
            eur: BuySellTable::new(markets, GoodKind::EUR),
            usd: BuySellTable::new(markets, GoodKind::USD),
            yen: BuySellTable::new(markets, GoodKind::YEN),
            yuan: BuySellTable::new(markets, GoodKind::YUAN),
        }
    }

    fn get_best_operation(&self) -> BestOperation {
        let mut best_operation = self.eur.get_best_operation(GoodKind::EUR);

        let new_best_operation = self.usd.get_best_operation(GoodKind::USD);
        if new_best_operation.expected_gain > best_operation.expected_gain {
            best_operation = new_best_operation;
        }

        let new_best_operation = self.yen.get_best_operation(GoodKind::YEN);
        if new_best_operation.expected_gain > best_operation.expected_gain {
            best_operation = new_best_operation;
        }

        let new_best_operation = self.yuan.get_best_operation(GoodKind::YUAN);
        if new_best_operation.expected_gain > best_operation.expected_gain {
            best_operation = new_best_operation;
        }

        return best_operation;
    }
}

#[derive(Debug)]
enum MarketToChoose {
    Sol,
    Rcnz,
    Bose,
}

#[derive(Debug)]
struct BestOperation {
    kind: GoodKind,
    buy_from: MarketToChoose,
    sell_to: MarketToChoose,
    expected_gain: f32,
}

pub struct PriceComparison {
    handler: Box<dyn EventHandler>,
}

impl PriceComparison {
    pub fn new() -> PriceComparison {
        PriceComparison {
            handler: Box::new(StdoutLogger::new()),
        }
    }
}

impl Strategy for PriceComparison {
    fn add_event_handler(&mut self, handler: Box<dyn EventHandler>) {}

    fn apply(&mut self, account: &mut Account, markets: &mut MarketWrapper) {
        let price_table = PriceTable::new(markets);
        // println!("{:#?}", price_table);
        let best_operation = price_table.get_best_operation();
        // println!("{:#?}", best_operation);

        if best_operation.expected_gain < 2.0 {
            wait_one_day!();
            return;
        }

        let success = direct_buy(
            &self.handler,
            account,
            match best_operation.buy_from {
                MarketToChoose::Sol => markets.sol.clone(),
                MarketToChoose::Rcnz => markets.rcnz.clone(),
                MarketToChoose::Bose => markets.bose.clone(),
            },
            best_operation.kind,
            QUANTITY,
        );

        if success.is_ok() {
            let success = direct_sell(
                &self.handler,
                account,
                match best_operation.sell_to {
                    MarketToChoose::Sol => markets.sol.clone(),
                    MarketToChoose::Rcnz => markets.rcnz.clone(),
                    MarketToChoose::Bose => markets.bose.clone(),
                },
                best_operation.kind,
                QUANTITY,
            );

            if success.is_err() {
                wait_one_day!();
            }
        } else {
            wait_one_day!();
        }
    }

    fn get_name(&self) -> &'static str {
        "Table Comparison"
    }
}
