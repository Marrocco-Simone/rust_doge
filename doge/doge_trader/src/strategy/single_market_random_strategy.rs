use std::{cell::RefCell, rc::Rc};

use doge_common::account::Account;
use rand::{distributions::Uniform, prelude::Distribution};
use unitn_market_2022::{good::good_kind::GoodKind, market::Market};

use crate::{
    event_handler::EventHandler, logger::stdout_logger::StdoutLogger, markets::MarketWrapper,
};

use super::{
    common::{direct_buy, direct_sell},
    Strategy,
};

// * structs
pub struct SolRandom {
    handler: Box<dyn EventHandler>,
}
pub struct RcnzRandom {
    handler: Box<dyn EventHandler>,
}
pub struct BoseRandom {
    handler: Box<dyn EventHandler>,
}

/// * general function
fn apply(account: &mut Account, market: Rc<RefCell<dyn Market>>, handler: &Box<dyn EventHandler>) {
    let mut rng = rand::thread_rng();

    let is_buy_uniform = Uniform::from(0..2);
    let is_buy = match is_buy_uniform.sample(&mut rng) {
        0 => false,
        1 => true,
        _ => panic!("ran gen id_buy not working (should not be here)"),
    };

    let good_kind_uniform = Uniform::from(0..4);
    let good_kind = match good_kind_uniform.sample(&mut rng) {
        0 => GoodKind::EUR,
        1 => GoodKind::USD,
        2 => GoodKind::YEN,
        3 => GoodKind::YUAN,
        _ => panic!("ran gen kind not working (should not be here)"),
    };

    const MAX_QUANTITY: f32 = 100000.0;
    let quantity_uniform = Uniform::from(0.0..MAX_QUANTITY);
    let quantity =
        quantity_uniform.sample(&mut rng) * GoodKind::get_default_exchange_rate(&good_kind);

    let result = match is_buy {
        true => direct_buy(handler, account, market, good_kind, quantity),
        false => direct_sell(handler, account, market, good_kind, quantity),
    };

    match result {
        Ok(_) => (),
        Err(msg) => println!("{:?}", msg),
    }
}

// * impl new()
impl SolRandom {
    pub fn new() -> SolRandom {
        SolRandom {
            handler: Box::new(StdoutLogger::new()),
        }
    }
}

impl RcnzRandom {
    pub fn new() -> RcnzRandom {
        RcnzRandom {
            handler: Box::new(StdoutLogger::new()),
        }
    }
}

impl BoseRandom {
    pub fn new() -> BoseRandom {
        BoseRandom {
            handler: Box::new(StdoutLogger::new()),
        }
    }
}

// * impl Strategy
impl Strategy for SolRandom {
    fn add_event_handler(&mut self, handler: Box<dyn EventHandler>) {}

    fn apply(&mut self, account: &mut Account, markets: &mut MarketWrapper) {
        apply(account, markets.sol.clone(), &self.handler)
    }

    fn get_name(&self) -> &'static str {
        "SOL Random"
    }
}

impl Strategy for RcnzRandom {
    fn add_event_handler(&mut self, handler: Box<dyn EventHandler>) {}

    fn apply(&mut self, account: &mut Account, markets: &mut MarketWrapper) {
        apply(account, markets.rcnz.clone(), &self.handler)
    }

    fn get_name(&self) -> &'static str {
        "RCNZ Random"
    }
}

impl Strategy for BoseRandom {
    fn add_event_handler(&mut self, handler: Box<dyn EventHandler>) {}

    fn apply(&mut self, account: &mut Account, markets: &mut MarketWrapper) {
        apply(account, markets.bose.clone(), &self.handler)
    }

    fn get_name(&self) -> &'static str {
        "BOSE Random"
    }
}
