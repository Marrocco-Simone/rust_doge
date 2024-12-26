use std::fs;

use plotters::prelude::*;

use crate::simulation_fs::{
    general_functions::max_f32,
    structs::{file_pointers::TRANSACTIONS_JSON_FILENAME, transaction_info::TransactionInfo},
};

use super::draw_chart::draw_chart;

pub fn create_transactions_graphs(svg_path: &str) {
    let transactions_json = fs::read_to_string(TRANSACTIONS_JSON_FILENAME.to_string()).expect(
        format!(
            "cannot read file {} to create graph",
            TRANSACTIONS_JSON_FILENAME
        )
        .as_str(),
    );
    let transactions_data: Vec<TransactionInfo> = serde_json::from_str(&transactions_json).expect(
        format!(
            "cannot pase file {} to valid json",
            TRANSACTIONS_JSON_FILENAME
        )
        .as_str(),
    );

    let mut transactions_data_max_gain = 0.;
    let mut transactions_data_max_value = 0.;
    let mut transactions_data_max_euro_payed = 0.;
    for t in &transactions_data {
        // TODO change to percenteage
        let difference = t.final_market_value - t.initial_market_value;
        transactions_data_max_gain = max_f32(transactions_data_max_gain, difference);
        transactions_data_max_value = max_f32(transactions_data_max_value, t.final_market_value);
        transactions_data_max_euro_payed = max_f32(transactions_data_max_euro_payed, t.euro_payed);
    }

    let root = SVGBackend::new(svg_path, (1920, 1080)).into_drawing_area();
    root.fill(&WHITE).expect("cannot fill root white");

    let areas = root.split_evenly((2, 1));
    draw_chart(
        &areas[0],
        &transactions_data,
        transactions_data_max_gain,
        "Transaction Market Gain",
        "Time",
        "Market Gain in EUR",
        "EUR",
        |(i, t): (usize, &TransactionInfo)| (i, (t.final_market_value - t.initial_market_value)),
        1,
        10,
    );
    draw_chart(
        &areas[1],
        &transactions_data,
        transactions_data_max_euro_payed,
        "Transaction EUR payed",
        "Time",
        "EUR payed for transaction",
        "EUR",
        |(i, t): (usize, &TransactionInfo)| (i, t.euro_payed),
        3,
        10,
    );

    root.present().expect("Unable to create graph");
}
