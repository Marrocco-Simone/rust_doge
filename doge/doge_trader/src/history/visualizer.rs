use plotters::prelude::*;

use doge_common::account::Account;

use crate::chart::make_chart;
use crate::DOGE_TRADER_NAME;
use crate::history::HistoryPoint;

/// Produce un file SVG contenente cinque diagrammi cartesiani:
/// - Il diagramma in alto mostra la variazione della quantità complessiva dei GoodKind.
/// La quantità complessiva è la somma della quantità di EUR e delle quantità degli altri GoodKind
// moltiplicate per i rispettivi tassi di cambio di default.
/// - I quattro diagrammi in basso mostrano le variazioni delle quantità dei quattro diversi GoodKind.
fn make_account_history_charts<'a>(market_name: &'static str, account_history: impl Iterator<Item=&'a Account> + Clone) {
    let filename = format!("{market_name}_account_history.svg");

    let root = SVGBackend::new(std::path::Path::new(&filename), (1920, 1080)).into_drawing_area();
    root.fill(&WHITE).unwrap();

    // Divisione in 2 righe, una colonna.
    let areas = root.split_evenly((2, 1));

    // La prima riga conterrà al diagramma comparativo...
    make_chart(account_history.clone().map(|account| account.get_total_account_value_in_eur()), &areas[0], &format!("{market_name} total good quantity variation"));

    // ...la seconda i quattro rimanenti diagrammi.
    let areas = areas[1].split_evenly((2, 2));

    make_chart(account_history.clone().map(|account| account.eur.get_qty()), &areas[0], &format!("{market_name} EUR quantity variation"));
    make_chart(account_history.clone().map(|account| account.usd.get_qty()), &areas[1], &format!("{market_name} USD quantity variation"));
    make_chart(account_history.clone().map(|account| account.yen.get_qty()), &areas[2], &format!("{market_name} YEN quantity variation"));
    make_chart(account_history.map(|account| account.yuan.get_qty()), &areas[3], &format!("{market_name} YUAN quantity variation"));

    root.present().unwrap();
}

pub fn make_history_points_charts<'a>(history: impl Iterator<Item=&'a HistoryPoint> + Clone) {
    make_account_history_charts(DOGE_TRADER_NAME, history.clone().map(|point| &point.trader));
    make_account_history_charts("SOL", history.clone().map(|point| &point.sol));
    make_account_history_charts("RCNZ", history.clone().map(|point| &point.rcnz));
    make_account_history_charts("BOSE", history.map(|point| &point.bose));
}
