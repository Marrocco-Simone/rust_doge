use std::fs;

use plotters::prelude::*;
use unitn_market_2022::good::good_kind::GoodKind;

use crate::simulation_fs::general_functions::max_f32;
use crate::simulation_fs::structs::goods_wallet::GoodsWallet;

use super::draw_chart::draw_chart;

pub fn create_wallet_graphs(json_path: &str, svg_path: &str, caption: &str) {
    let wallet_json = fs::read_to_string(json_path.to_string())
        .expect(format!("cannot read file {} to create graph", json_path).as_str());
    let mut wallets: Vec<GoodsWallet> = serde_json::from_str(&wallet_json)
        .expect(format!("cannot parse file {} to a valid json", json_path).as_str());

    let root = SVGBackend::new(svg_path, (1920, 1080)).into_drawing_area();
    root.fill(&WHITE).expect("cannot fill root to white");
    let areas = root.split_evenly((2, 1));

    let mut wallets_max_value: f32 = 0.;
    for w in &mut wallets {
        wallets_max_value = max_f32(wallets_max_value, w.get_value());
    }

    draw_chart(
        &areas[0],
        &wallets,
        wallets_max_value,
        format!("{} Total Wallet Value in EUR", caption).as_str(),
        "Time",
        "Total value in EUR",
        "Total Value in Eur",
        |(i, w): (usize, &GoodsWallet)| (i, w.get_value()),
        0,
        5,
    );

    let quadrants = areas[1].split_evenly((2, 2));
    let mut wallets_max_good: f32 = 0.;
    for w in &mut wallets {
        wallets_max_good = max_f32(wallets_max_good, w.get_good(GoodKind::EUR));
    }
    draw_chart(
        &quadrants[0],
        &wallets,
        wallets_max_good,
        "EUR Good Quantity",
        "Time",
        "",
        "EUR",
        |(i, w): (usize, &GoodsWallet)| (i, w.eur),
        1,
        5,
    );

    let mut wallets_max_good: f32 = 0.;
    for w in &mut wallets {
        wallets_max_good = max_f32(wallets_max_good, w.get_good(GoodKind::USD));
    }
    draw_chart(
        &quadrants[1],
        &wallets,
        wallets_max_good,
        "USD Good Quantity",
        "Time",
        "",
        "USD",
        |(i, w): (usize, &GoodsWallet)| (i, w.usd),
        3,
        5,
    );

    let mut wallets_max_good: f32 = 0.;
    for w in &mut wallets {
        wallets_max_good = max_f32(wallets_max_good, w.get_good(GoodKind::YEN));
    }
    draw_chart(
        &quadrants[2],
        &wallets,
        wallets_max_good,
        "YEN Good Quantity",
        "Time",
        "",
        "YEN",
        |(i, w): (usize, &GoodsWallet)| (i, w.yen),
        4,
        5,
    );

    let mut wallets_max_good: f32 = 0.;
    for w in &mut wallets {
        wallets_max_good = max_f32(wallets_max_good, w.get_good(GoodKind::YUAN));
    }
    draw_chart(
        &quadrants[3],
        &wallets,
        wallets_max_good,
        "YUAN Good Quantity",
        "Time",
        "",
        "YUAN",
        |(i, w): (usize, &GoodsWallet)| (i, w.yuan),
        5,
        5,
    );

    root.present().expect("Unable to create graph");
}
