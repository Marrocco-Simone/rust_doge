use std::f32;

use rand::Rng;
use unitn_market_2022::good::consts::STARTING_CAPITAL;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::good::good_kind::GoodKind::{EUR, USD, YEN, YUAN};

use doge_common::account::Account;

use crate::refiller::TrackerState::{Exporter, Importer, ImporterExporter, Shortage};

const MIN_DAYS_AS_IMPORTER: u32 = 100;
const MIN_DAYS_AS_EXPORTER: u32 = 100;
const MIN_DAYS_IN_SHORTAGE: u32 = 100;

const IMPORT_TAX: f32 = 0.25;
const SHORTAGE_PROBABILITY_PERCENT: i32 = 5;
const CAREFUL_FRACTION: f32 = 8.;

fn compute_importer_exporter_threshold(kind: GoodKind) -> f32 {
    STARTING_CAPITAL / CAREFUL_FRACTION * kind.get_default_exchange_rate()
}

#[derive(Debug)]
pub enum TrackerState {
    Importer { days_passed_as_importer: u32 },
    Exporter { days_passed_as_exporter: u32 },
    ImporterExporter,
    Shortage { days_passed_in_shortage: u32 },
}

#[derive(Debug)]
pub struct GoodTracker {
    kind: GoodKind,
    mode: TrackerState,
}


impl GoodTracker {
    fn new(kind: GoodKind) -> GoodTracker {
        GoodTracker {
            kind,
            mode: ImporterExporter,
        }
    }

    fn advance_a_day(&mut self) {
        match self.mode {
            Importer { days_passed_as_importer } => {
                if days_passed_as_importer < MIN_DAYS_AS_IMPORTER {
                    self.mode = Importer { days_passed_as_importer: days_passed_as_importer + 1 }
                } else {
                    self.mode = ImporterExporter
                }
            }
            Exporter { days_passed_as_exporter } => {
                if days_passed_as_exporter < MIN_DAYS_AS_EXPORTER {
                    self.mode = Exporter { days_passed_as_exporter: days_passed_as_exporter + 1 }
                } else {
                    self.mode = ImporterExporter
                }
            }
            ImporterExporter => {} // remain in this state
            Shortage { days_passed_in_shortage } => {
                if days_passed_in_shortage < MIN_DAYS_IN_SHORTAGE {
                    self.mode = Shortage { days_passed_in_shortage: days_passed_in_shortage + 1 }
                } else {
                    self.mode = ImporterExporter
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct GoodRefiller {
    eur_tracker: GoodTracker,
    usd_tracker: GoodTracker,
    yen_tracker: GoodTracker,
    yuan_tracker: GoodTracker,
}

impl GoodRefiller {
    pub fn new() -> GoodRefiller {
        GoodRefiller {
            eur_tracker: GoodTracker::new(EUR),
            usd_tracker: GoodTracker::new(USD),
            yen_tracker: GoodTracker::new(YEN),
            yuan_tracker: GoodTracker::new(YUAN),
        }
    }

    fn get_tracker_from_kind_mut(&mut self, kind: GoodKind) -> &mut GoodTracker {
        match kind {
            EUR => &mut self.eur_tracker,
            YEN => &mut self.yen_tracker,
            USD => &mut self.usd_tracker,
            YUAN => &mut self.yuan_tracker,
        }
    }

    fn increase_days(&mut self) {
        self.eur_tracker.advance_a_day();
        self.usd_tracker.advance_a_day();
        self.yen_tracker.advance_a_day();
        self.yuan_tracker.advance_a_day();
    }

    fn get_total_quantity_of_kind(assets: &Account, reservations: &Account, kind: GoodKind) -> f32 {
        assets.get_quantity_by_kind(kind) + reservations.get_quantity_by_kind(kind)
    }

    fn search_least_abundant_kind(&mut self, assets: &Account, reservations: &Account) -> Option<GoodKind> {
        [&mut self.eur_tracker, &mut self.usd_tracker, &mut self.yen_tracker, &mut self.yuan_tracker].into_iter()
            .filter(|tracker| matches!(tracker.mode, Importer { .. } | ImporterExporter))
            .filter(|tracker| GoodRefiller::get_total_quantity_of_kind(assets, reservations, tracker.kind) < compute_importer_exporter_threshold(tracker.kind))
            .map(|tracker| tracker.kind)
            .reduce(|min_so_far, curr| {
                if GoodRefiller::get_total_quantity_of_kind(assets, reservations, min_so_far) / min_so_far.get_default_exchange_rate() <=
                    GoodRefiller::get_total_quantity_of_kind(assets, reservations, curr) / curr.get_default_exchange_rate() {
                    min_so_far
                } else {
                    curr
                }
            })
    }

    fn search_most_abundant_kind(&mut self, assets: &Account, reservations: &Account) -> Option<GoodKind> {
        [&mut self.eur_tracker, &mut self.usd_tracker, &mut self.yen_tracker, &mut self.yuan_tracker].into_iter()
            .filter(|tracker| matches!(tracker.mode, Exporter { .. } | ImporterExporter))
            .filter(|tracker| GoodRefiller::get_total_quantity_of_kind(assets, reservations, tracker.kind) >= compute_importer_exporter_threshold(tracker.kind) * 2.)
            .map(|tracker| tracker.kind)
            .reduce(|max_so_far, curr| {
                if GoodRefiller::get_total_quantity_of_kind(assets, reservations, max_so_far) / max_so_far.get_default_exchange_rate() >=
                    GoodRefiller::get_total_quantity_of_kind(assets, reservations, curr) / curr.get_default_exchange_rate() {
                    max_so_far
                } else {
                    curr
                }
            })
    }

    pub fn refill_goods(&mut self, assets: &mut Account, reservations: &Account) {
        self.increase_days();

        let least_abundant_good = if let Some(good) = self.search_least_abundant_kind(assets, reservations) { good } else { return; };
        let most_abundant_good = if let Some(good) = self.search_most_abundant_kind(assets, reservations) { good } else { return; };

        if rand::thread_rng().gen_range(0..100) < SHORTAGE_PROBABILITY_PERCENT {
            self.get_tracker_from_kind_mut(least_abundant_good).mode = Shortage { days_passed_in_shortage: 0 };
            return;
        }

        // quantity needed by the least abundant good to reach the careful value
        let least_abundant_good_needed_quantity = compute_importer_exporter_threshold(least_abundant_good) - GoodRefiller::get_total_quantity_of_kind(assets, reservations, least_abundant_good);
        if least_abundant_good_needed_quantity < 0. {
            return;
        }

        // quantity that the most abundant good can cede, remaining above the careful value
        let most_abundant_good_available_quantity = GoodRefiller::get_total_quantity_of_kind(assets, reservations, most_abundant_good) - compute_importer_exporter_threshold(most_abundant_good);
        if most_abundant_good_available_quantity < 0. {
            return;
        }

        if let ImporterExporter = self.get_tracker_from_kind_mut(least_abundant_good).mode {
            self.get_tracker_from_kind_mut(least_abundant_good).mode = Importer { days_passed_as_importer: 0 }
        }

        if let ImporterExporter = self.get_tracker_from_kind_mut(most_abundant_good).mode {
            self.get_tracker_from_kind_mut(most_abundant_good).mode = Exporter { days_passed_as_exporter: 0 }
        }

        // first convert least abundant_good_needed_quantity to euro
        let least_abundant_good_needed_quantity_eur = least_abundant_good_needed_quantity / least_abundant_good.get_default_exchange_rate();
        // now convert it to most_abundant_good_quantity kind
        let least_abundant_good_needed_quantity_kind_most = least_abundant_good_needed_quantity_eur * most_abundant_good.get_default_exchange_rate();

        // example 1:
        // least_abundant_good__needed_quantity_kind_most = 3
        // most_abundant_good__available_quantity = 7
        // most_abundant_good__quantity_to_withdraw => 3
        // example 2:
        // least_abundant_good__needed_quantity_kind_most = 7
        // most_abundant_good__available_quantity = 3
        // most_abundant_good__quantity_to_withdraw => 3
        let most_abundant_good_quantity_to_withdraw = f32::min(least_abundant_good_needed_quantity_kind_most / (1. - IMPORT_TAX), most_abundant_good_available_quantity);
        let most_abundant_good_quantity_to_withdraw_eur = most_abundant_good_quantity_to_withdraw / most_abundant_good.get_default_exchange_rate();

        let least_abundant_good_quantity_to_deposit = most_abundant_good_quantity_to_withdraw_eur * least_abundant_good.get_default_exchange_rate() * (1. - IMPORT_TAX);

        assets.withdraw(most_abundant_good, most_abundant_good_quantity_to_withdraw).unwrap();

        assets.deposit(Good::new(least_abundant_good, least_abundant_good_quantity_to_deposit));
    }
}

#[cfg(test)]
mod tests {
    use unitn_market_2022::good::consts::{DEFAULT_EUR_USD_EXCHANGE_RATE, DEFAULT_EUR_YEN_EXCHANGE_RATE, DEFAULT_EUR_YUAN_EXCHANGE_RATE};

    use super::*;

    #[test]
    fn days_should_pass() {
        let mut tracker = GoodTracker::new(EUR);
        tracker.mode = Importer { days_passed_as_importer: 0 };
        tracker.advance_a_day();
        if let Importer { days_passed_as_importer } = tracker.mode {
            assert_eq!(days_passed_as_importer, 1);
        } else {
            panic!()
        }
    }

    #[test]
    pub fn test() {
        let mut assets = Account {
            eur: Good::new(EUR, 500_000.),
            usd: Good::new(USD, 25_000. * DEFAULT_EUR_USD_EXCHANGE_RATE),
            yen: Good::new(YEN, 250_000. * DEFAULT_EUR_YEN_EXCHANGE_RATE),
            yuan: Good::new(YUAN, 250_000. * DEFAULT_EUR_YUAN_EXCHANGE_RATE),
        };
        println!("Initial assets content: {}", assets);

        let reservations = Account::new_empty();
        println!("Initial reservations content: {}", assets);

        let mut refiller = GoodRefiller::new();

        refiller.refill_goods(&mut assets, &reservations);
        println!("Refill assets #1: {}", assets);

        refiller.refill_goods(&mut assets, &reservations);
        println!("Refill assets #2: {}", assets);

        refiller.refill_goods(&mut assets, &reservations);
        println!("Refill assets #3: {}", assets);

        refiller.refill_goods(&mut assets, &reservations);
        println!("Refill assets #4: {}", assets);
    }
}