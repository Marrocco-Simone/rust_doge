use core::f32;
use std::cell::RefCell;

use plotters::coord::Shift;
use plotters::prelude::*;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::good::good_kind::GoodKind::{EUR, USD, YEN, YUAN};

use doge_common::account::Account;

use crate::chart::make_chart;
use crate::DOGE_TRADER_NAME;
use crate::event::TradingEvent;
use crate::event_handler::EventHandler;

pub struct SvgDisplayer {
    accounts: RefCell<Vec<Account>>,
}

impl SvgDisplayer {
    pub fn new() -> SvgDisplayer {
        SvgDisplayer { accounts: RefCell::new(vec![]) }
    }
}

impl EventHandler for SvgDisplayer {
    fn handle_event(&self, event: TradingEvent) {
        self.accounts.borrow_mut().push(event.account);
    }

    fn get_name(&self) -> &'static str {
        "SVG displayer"
    }
}

impl Drop for SvgDisplayer {
    fn drop(&mut self) {
        let root = SVGBackend::new("simulation.svg", (1920, 1080)).into_drawing_area();
        root.fill(&WHITE).unwrap();

        let split = root.split_evenly((2, 1));
        make_comparing_chart(&self.accounts.borrow(), &split[0]);

        let split = split[1].split_evenly((2, 2));
        make_chart(self.accounts.borrow().iter().map(|account| account.get_quantity_by_kind(EUR)), &split[0], &format!("{DOGE_TRADER_NAME} EUR quantity variation"));
        make_chart(self.accounts.borrow().iter().map(|account| account.get_quantity_by_kind(USD)), &split[1], &format!("{DOGE_TRADER_NAME} USD quantity variation"));
        make_chart(self.accounts.borrow().iter().map(|account| account.get_quantity_by_kind(YEN)), &split[2], &format!("{DOGE_TRADER_NAME} YEN quantity variation"));
        make_chart(self.accounts.borrow().iter().map(|account| account.get_quantity_by_kind(YUAN)), &split[3], &format!("{DOGE_TRADER_NAME} YUAN quantity variation"));

        root.present().unwrap();
    }
}

/// Produce un diagramma cartesiano della fluttuazione delle quantità di tutti i GoodKind nella corso simulazione:
/// - L'asse x del diagramma rappresenta le iterazioni della simulazione.
/// - L'asse y del diagramma rappresenta la quantità dei GoodKind.
fn make_comparing_chart(accounts: &Vec<Account>, drawing_area: &DrawingArea<SVGBackend, Shift>) {
    // La lunghezza dell'asse x è pari al numero di iterazioni (-1, in quanto la prima ha indice 0)
    let x_axis_len = accounts.len() as f32 - 1.;

    let lowest_quantity_of_kind = |kind: GoodKind| accounts.iter()
        .map(|account| account.get_quantity_by_kind(kind))
        .min_by(|first, second| first.total_cmp(second))
        .unwrap_or(0f32);

    let highest_quantity_of_kind = |kind: GoodKind| accounts.iter()
        .map(|account| account.get_quantity_by_kind(kind))
        .max_by(|first, second| first.total_cmp(second))
        .unwrap_or(0f32);

    // L'altezza dell'asse y è pari al valore più alto raggiunto da una qualsiasi quantità di GoodKind nella simulazione
    let y_min = lowest_quantity_of_kind(EUR)
        .min(lowest_quantity_of_kind(USD))
        .min(lowest_quantity_of_kind(YEN))
        .min(lowest_quantity_of_kind(YUAN));
    let y_max = highest_quantity_of_kind(EUR)
        .max(highest_quantity_of_kind(USD))
        .max(highest_quantity_of_kind(YEN))
        .max(highest_quantity_of_kind(YUAN));

    let mut chart = ChartBuilder::on(drawing_area)
        .caption("Fluctuation of good kinds over time", ("sans-serif", 20))
        .x_label_area_size(20)
        .y_label_area_size(80)
        .margin(20)
        .build_cartesian_2d(0f32..x_axis_len, y_min..y_max)
        .unwrap();

    chart
        .configure_mesh()
        .bold_line_style(BLACK.mix(0.25)) // big grid lines
        .light_line_style(TRANSPARENT) // little grid lines
        .draw()
        .unwrap();

    let mut draw_series = |kind: GoodKind, color: &RGBColor| {
        let points = accounts.iter()
            .map(|account| account.get_quantity_by_kind(kind))
            .enumerate()
            .map(|(iter, quantity)| (iter as f32, quantity));
        let series = LineSeries::new(points, color);
        chart.draw_series(series).unwrap();
    };

    draw_series(EUR, &RED);
    draw_series(USD, &GREEN);
    draw_series(YEN, &BLUE);
    draw_series(YUAN, &YELLOW);
}

#[cfg(test)]
mod tests {
    use unitn_market_2022::good::good::Good;
    use unitn_market_2022::good::good_kind::GoodKind::{EUR, USD, YEN, YUAN};

    use doge_common::account::Account;

    use super::*;

    #[test]
    fn test_make_comparing_chart() {
        let accounts = vec![
            Account {
                eur: Good::new(EUR, 1000.),
                usd: Good::new(USD, 260.),
                yen: Good::new(YEN, 743.),
                yuan: Good::new(YUAN, 500.),
            },
            Account {
                eur: Good::new(EUR, 2500.),
                usd: Good::new(USD, 800.),
                yen: Good::new(YEN, 550.),
                yuan: Good::new(YUAN, 980.),
            },
            Account {
                eur: Good::new(EUR, 1750.),
                usd: Good::new(USD, 135.),
                yen: Good::new(YEN, 223.),
                yuan: Good::new(YUAN, 100.),
            },
            Account {
                eur: Good::new(EUR, 1600.),
                usd: Good::new(USD, 226.),
                yen: Good::new(YEN, 114.),
                yuan: Good::new(YUAN, 0.5),
            },
            Account {
                eur: Good::new(EUR, 2340.),
                usd: Good::new(USD, 167.),
                yen: Good::new(YEN, 2.9),
                yuan: Good::new(YUAN, 10.),
            },
            Account {
                eur: Good::new(EUR, 1750.),
                usd: Good::new(USD, 135.),
                yen: Good::new(YEN, 223.),
                yuan: Good::new(YUAN, 100.),
            },
        ];

        let root = SVGBackend::new("make_comparing_chart_test.svg", (1920, 1080)).into_drawing_area();
        root.fill(&WHITE).unwrap();

        make_comparing_chart(&accounts, &root);

        root.present().unwrap();
    }
}
