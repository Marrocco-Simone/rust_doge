use doge_common::account::Account;
use plotters::prelude::*;

use super::draw_chart::draw_chart;

// TODO move somewhere else, but where?
pub fn max_f32(v1: f32, v2: f32) -> f32 {
    if v1 > v2 {
        v1
    } else {
        v2
    }
}

// TODO move somewhere else, but where?
pub fn min_f32(v1: f32, v2: f32) -> f32 {
    if v1 < v2 {
        v1
    } else {
        v2
    }
}

pub fn create_account_graphs(accounts: &Vec<Account>, svg_path: &str, caption: &str) {
    let root = SVGBackend::new(svg_path, (1920, 1080)).into_drawing_area();
    root.fill(&WHITE).expect("cannot fill root to white");
    let areas = root.split_evenly((2, 1));

    let mut accounts_max_value: f32 = 0.;
    for a in accounts {
        accounts_max_value = max_f32(accounts_max_value, a.get_total_account_value_in_eur());
    }

    draw_chart(
        &areas[0],
        &accounts,
        accounts_max_value,
        format!("{} Total Wallet Value in EUR", caption).as_str(),
        "Time",
        "Total value in EUR",
        "Total Value in Eur",
        |(i, a): (usize, &Account)| (i, a.get_total_account_value_in_eur()),
        0,
        5,
    );

    let quadrants = areas[1].split_evenly((2, 2));
    let mut accounts_max_good: f32 = 0.;
    for a in accounts {
        accounts_max_good = max_f32(accounts_max_good, a.eur.get_qty());
    }
    draw_chart(
        &quadrants[0],
        &accounts,
        accounts_max_good,
        "EUR Good Quantity",
        "Time",
        "",
        "EUR",
        |(i, a): (usize, &Account)| (i, a.eur.get_qty()),
        1,
        5,
    );

    let mut accounts_max_good: f32 = 0.;
    for a in accounts {
        accounts_max_good = max_f32(accounts_max_good, a.usd.get_qty());
    }
    draw_chart(
        &quadrants[1],
        &accounts,
        accounts_max_good,
        "USD Good Quantity",
        "Time",
        "",
        "USD",
        |(i, a): (usize, &Account)| (i, a.usd.get_qty()),
        3,
        5,
    );

    let mut accounts_max_good: f32 = 0.;
    for a in accounts {
        accounts_max_good = max_f32(accounts_max_good, a.yen.get_qty());
    }
    draw_chart(
        &quadrants[2],
        &accounts,
        accounts_max_good,
        "YEN Good Quantity",
        "Time",
        "",
        "YEN",
        |(i, a): (usize, &Account)| (i, a.yen.get_qty()),
        4,
        5,
    );

    let mut accounts_max_good: f32 = 0.;
    for a in accounts {
        accounts_max_good = max_f32(accounts_max_good, a.yuan.get_qty());
    }
    draw_chart(
        &quadrants[3],
        &accounts,
        accounts_max_good,
        "YUAN Good Quantity",
        "Time",
        "",
        "YUAN",
        |(i, a): (usize, &Account)| (i, a.yuan.get_qty()),
        5,
        5,
    );

    root.present().expect("Unable to create graph");
}
