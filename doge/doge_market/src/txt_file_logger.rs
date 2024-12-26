use std::fs::File;
use std::io::Write;

use chrono::{DateTime, Local};
use unitn_market_2022::good::good::Good;
use uuid::Uuid;

use crate::logger::Logger;

/// Text file logger that conforms to the [Market protocol specifications](https://github.com/WG-AdvancedProgramming/market-protocol-specifications/blob/main/market-protocol-specifications.md).
pub struct TxtFileLogger {
    file: File,
    market_name: String,
}

impl TxtFileLogger {
    pub fn try_new(market_name: &str) -> Option<TxtFileLogger> {
        match File::create(TxtFileLogger::format_filename(market_name)) {
            Ok(file) => {
                Some(TxtFileLogger {
                    file,
                    market_name: market_name.to_string(),
                })
            }
            Err(err) => {
                eprintln!("Could not create TxtFileLogger: {}", err);
                None
            }
        }
    }


    fn format_filename(market_name: &str) -> String {
        format!("log_{}.txt", market_name.to_lowercase())
    }

    /// Formats a log code according to the [Market protocol specifications](https://github.com/WG-AdvancedProgramming/market-protocol-specifications/blob/main/market-protocol-specifications.md#market-logs):
    /// `<market_name>|YY:MM:DD:HH:SEC:MSES|<log_code>\n`
    fn format_log_code(market_name: &str, datetime: DateTime<Local>, log_code: &str) -> String {
        // %T 00:34:60 Hour-minute-second format. Same as %H:%M:%S.
        // %f 026490000 The fractional seconds (in nanoseconds) since last whole second.
        format!("{}|{}|{}", market_name, datetime.format("%Y:%m:%d:%T:%3f"), log_code)
    }

    /// MARKET_INITIALIZATION
    /// EUR: <eur_good_quantity>
    /// USD: <usd_good_quantity>
    /// YEN: <yen_good_quantity>
    /// YUAN: <yuan_good_quantity>
    /// END_MARKET_INITIALIZATION
    fn initialization_code(eur: &Good, usd: &Good, jpy: &Good, cny: &Good) -> String {
        format!("MARKET_INITIALIZATION\n\
            EUR: {}\n\
            USD: {}\n\
            YEN: {}\n\
            YUAN: {}\n\
            END_MARKET_INITIALIZATION",
                eur.get_qty(),
                usd.get_qty(),
                jpy.get_qty(),
                cny.get_qty())
    }

    /// ```LOCK_BUY-<trader-name>-KIND_TO_BUY:<good_kind>-QUANTITY_TO_BUY:<quantity_to_buy>-BID:<bid>-TOKEN:<token>``` if lock_buy returns Ok
    /// ```LOCK_BUY-<trader-name>-KIND_TO_BUY:<good_kind>-QUANTITY_TO_BUY:<quantity_to_buy>-BID:<bid>-ERROR``` if lock_buy returns Err
    fn lock_buy_code(locked: &Good, trader_name: &str, bid: &Good, token: Option<&Uuid>) -> String {
        match token {
            Some(token) => {
                format!("LOCK_BUY-{}-KIND_TO_BUY:{}-QUANTITY_TO_BUY:{}-BID:{}-TOKEN:{}",
                        trader_name, locked.get_kind(), locked.get_qty(), bid.get_qty(), token.to_string())
            }
            None => {
                format!("LOCK_BUY-{}-KIND_TO_BUY:{}-QUANTITY_TO_BUY:{}-BID:{}-ERROR",
                        trader_name, locked.get_kind(), locked.get_qty(), bid.get_qty())
            }
        }
    }

    /// ```LOCK_SELL-<trader-name>-KIND_TO_SELL:<good_kind>-QUANTITY_TO_SELL:<quantity_to_sell>-OFFER:<offer>-TOKEN:<token>``` if lock_buy returns Ok
    /// ```LOCK_SELL-<trader-name>-KIND_TO_SELL:<good_kind>-QUANTITY_TO_SELL:<quantity_to_sell>-OFFER:<offer>-ERROR``` if lock_buy returns Err
    fn lock_sell_code(locked: &Good, trader_name: &str, offer: &Good, token: Option<&Uuid>) -> String {
        match token {
            Some(token) => {
                format!("LOCK_SELL-{}-KIND_TO_SELL:{}-QUANTITY_TO_SELL:{}-OFFER:{}-TOKEN:{}",
                        trader_name, locked.get_kind(), locked.get_qty(), offer.get_qty(), token.to_string())
            }
            None => {
                format!("LOCK_SELL-{}-KIND_TO_SELL:{}-QUANTITY_TO_SELL:{}-OFFER:{}-ERROR",
                        trader_name, locked.get_kind(), locked.get_qty(), offer.get_qty())
            }
        }
    }

    /// ```BUY-TOKEN:<token>-OK``` if the buy returns Ok
    /// ```BUY-TOKEN:<token>-ERROR``` if the buy returns Err
    fn buy_code(token: &Uuid, success: bool) -> String {
        match success {
            true => {
                format!("BUY-TOKEN:{}-OK", token.to_string())
            }
            false => {
                format!("BUY-TOKEN:{}-ERROR", token.to_string())
            }
        }
    }

    /// ```SELL-TOKEN:<token>-OK``` if the sell returns Ok
    /// ```SELL-TOKEN:<token>-ERROR``` if the sell returns Err
    fn sell_code(token: &Uuid, success: bool) -> String {
        match success {
            true => {
                format!("SELL-TOKEN:{}-OK", token.to_string())
            }
            false => {
                format!("SELL-TOKEN:{}-ERROR", token.to_string())
            }
        }
    }
}

impl Logger for TxtFileLogger {
    fn log_initialization(&mut self, eur: &Good, usd: &Good, jpy: &Good, cny: &Good) {
        let initialization_code = TxtFileLogger::initialization_code(eur, usd, jpy, cny);
        let initialization_code = TxtFileLogger::format_log_code(&self.market_name, Local::now(), &initialization_code);
        writeln!(self.file, "{initialization_code}").unwrap();
    }

    fn log_lock_buy(&mut self, locked: &Good, trader_name: &str, bid: &Good, token: Option<&Uuid>) {
        let buy_code = TxtFileLogger::lock_buy_code(locked, trader_name, bid, token);
        let buy_code = TxtFileLogger::format_log_code(&self.market_name, Local::now(), &buy_code);
        writeln!(self.file, "{buy_code}").unwrap();
    }

    fn log_lock_sell(&mut self, locked: &Good, trader_name: &str, offer: &Good, token: Option<&Uuid>) {
        let sell_code = TxtFileLogger::lock_sell_code(locked, trader_name, offer, token);
        let sell_code = TxtFileLogger::format_log_code(&self.market_name, Local::now(), &sell_code);
        writeln!(&self.file, "{sell_code}").unwrap();
    }

    fn log_buy(&mut self, token: &Uuid, success: bool) {
        let buy_code = TxtFileLogger::buy_code(token, success);
        let buy_code = TxtFileLogger::format_log_code(&self.market_name, Local::now(), &buy_code);
        writeln!(self.file, "{buy_code}").unwrap();
    }

    fn log_sell(&mut self, token: &Uuid, success: bool) {
        let sell_code = TxtFileLogger::sell_code(token, success);
        let sell_code = TxtFileLogger::format_log_code(&self.market_name, Local::now(), &sell_code);
        writeln!(&self.file, "{sell_code}").unwrap();
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use unitn_market_2022::good::good_kind::GoodKind::{EUR, USD, YEN, YUAN};

    use super::*;

    #[test]
    fn test_datetime_format() {
        let datetime: DateTime<Local> = DateTime::from_str("2022-12-01 15:39:19.315644312 +01:00").unwrap();
        let formatted = TxtFileLogger::format_log_code("DogeMarket", datetime, "Testing testing...");
        // `<market_name>|YY:MM:DD:HH:SEC:MSES|<log_code>`
        assert_eq!("DogeMarket|2022:12:01:15:39:19:315|Testing testing...", formatted);
    }

    #[test]
    fn test_market_initialization_log_code() {
        let initialization_code = TxtFileLogger::initialization_code(&Good::new(EUR, 14.2),
                                                                     &Good::new(USD, 8.2),
                                                                     &Good::new(YEN, 5.7),
                                                                     &Good::new(YUAN, 9.1));
        // MARKET_INITIALIZATION
        // EUR: <eur_good_quantity>
        // USD: <usd_good_quantity>
        // YEN: <yen_good_quantity>
        // YUAN: <yuan_good_quantity>
        // END_MARKET_INITIALIZATION
        assert_eq!("MARKET_INITIALIZATION\n\
        EUR: 14.2\n\
        USD: 8.2\n\
        YEN: 5.7\n\
        YUAN: 9.1\n\
        END_MARKET_INITIALIZATION", initialization_code);
    }

    #[test]
    fn test_lock_buy_log_code() {
        let output = TxtFileLogger::lock_buy_code(&Good::new(USD, 1500.), "DogeMarket", &Good::new(EUR, 500.), Some(&Uuid::from_str("3ea6179f-f05a-4cc0-a66f-ed55cd1b0aa3").unwrap()));
        assert_eq!("LOCK_BUY-DogeMarket-KIND_TO_BUY:USD-QUANTITY_TO_BUY:1500-BID:500-TOKEN:3ea6179f-f05a-4cc0-a66f-ed55cd1b0aa3", output);
        let output = TxtFileLogger::lock_buy_code(&Good::new(YEN, 33454.), "DogeMarket", &Good::new(EUR, 200.), None);
        assert_eq!("LOCK_BUY-DogeMarket-KIND_TO_BUY:YEN-QUANTITY_TO_BUY:33454-BID:200-ERROR", output);
    }

    #[test]
    fn test_lock_sell_log_code() {
        let output = TxtFileLogger::lock_sell_code(&Good::new(YUAN, 280.), "ShibaMarket", &Good::new(EUR, 35.), Some(&Uuid::from_str("3b177215-6b9d-414f-befd-7703a80e5829").unwrap()));
        assert_eq!("LOCK_SELL-ShibaMarket-KIND_TO_SELL:YUAN-QUANTITY_TO_SELL:280-OFFER:35-TOKEN:3b177215-6b9d-414f-befd-7703a80e5829", output);
        let output = TxtFileLogger::lock_sell_code(&Good::new(USD, 15000.), "ShibaMarket", &Good::new(EUR, 12000.), None);
        assert_eq!("LOCK_SELL-ShibaMarket-KIND_TO_SELL:USD-QUANTITY_TO_SELL:15000-OFFER:12000-ERROR", output);
    }


    #[test]
    fn test_buy_log_code() {
        let output = TxtFileLogger::buy_code(&Uuid::from_str("465823ac-cccf-407a-971b-49679f32d874").unwrap(), true);
        assert_eq!("BUY-TOKEN:465823ac-cccf-407a-971b-49679f32d874-OK", output);
        let output = TxtFileLogger::buy_code(&Uuid::from_str("3485e814-79c2-4afc-9c8d-f2bdbe4d78d9").unwrap(), false);
        assert_eq!("BUY-TOKEN:3485e814-79c2-4afc-9c8d-f2bdbe4d78d9-ERROR", output);
    }

    #[test]
    fn test_sell_log_code() {
        let output = TxtFileLogger::sell_code(&Uuid::from_str("938d4b2f-40e7-4ede-915f-f050fd5c5237").unwrap(), true);
        assert_eq!("SELL-TOKEN:938d4b2f-40e7-4ede-915f-f050fd5c5237-OK", output);
        let output = TxtFileLogger::sell_code(&Uuid::from_str("7ecaab48-62e4-4a31-8e9a-d13f3f2cbc1d").unwrap(), false);
        assert_eq!("SELL-TOKEN:7ecaab48-62e4-4a31-8e9a-d13f3f2cbc1d-ERROR", output);
    }
}
