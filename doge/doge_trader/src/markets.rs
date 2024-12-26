use std::cell::RefCell;
use std::rc::Rc;

use bose::market::BoseMarket;
use market_sol::SOLMarket;
use rcnz_market::rcnz::RCNZ;
use unitn_market_2022::{subscribe_each_other, wait_one_day};
use unitn_market_2022::market::Market;

pub struct MarketWrapper {
    pub sol: Rc<RefCell<dyn Market>>,
    pub rcnz: Rc<RefCell<dyn Market>>,
    pub bose: Rc<RefCell<dyn Market>>,
}

#[derive(Clone)]
pub struct Iter {
    markets: Vec<Rc<RefCell<dyn Market>>>,
}

impl Iterator for Iter {
    type Item = Rc<RefCell<dyn Market>>;

    fn next(&mut self) -> Option<Rc<RefCell<dyn Market>>> {
        self.markets.pop()
    }
}

impl IntoIterator for &MarketWrapper {
    type Item = Rc<RefCell<dyn Market>>;
    type IntoIter = Iter;

    fn into_iter(self) -> Iter {
        self.iter()
    }
}

impl MarketWrapper {
    pub fn new() -> MarketWrapper {
        let sol = SOLMarket::new_random();
        let rcnz = RCNZ::new_random();
        let bose = BoseMarket::new_random();

        subscribe_each_other!(sol, rcnz, bose);

        MarketWrapper { sol, rcnz, bose }
    }

    pub fn wait_one_day(&self) {
        wait_one_day!(self.bose,self.rcnz,self.sol);
    }

    pub fn iter(&self) -> Iter {
        Iter { markets: vec![Rc::clone(&self.sol), Rc::clone(&self.rcnz), Rc::clone(&self.bose)] }
    }
}

#[test]
fn test_iterator() {
    let wrapper = MarketWrapper::new();
    wrapper.iter().for_each(|market| { println!("{}", market.borrow().get_name()); });
    wrapper.iter().for_each(|market| { println!("{}", market.borrow().get_name()); });
}
