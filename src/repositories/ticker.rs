use diesel::prelude::*;
use diesel::dsl::avg;
use crate::schema::tickers::dsl::*;
use std::error::Error;

#[derive(Debug)]
pub enum TradeSignal {
    Buy,
    Sell,
    Hold,
    InsufficientData,
}

pub fn determine_trade_signal(
    conn: &mut PgConnection,
    currency: &str
) -> Result<TradeSignal, Box<dyn Error>> {

    let ma5 = tickers
        .filter(pair.eq(currency))
        .order(timestamp.desc())
        .limit(5)
        .select(avg(last))
        .first::<Option<f64>>(conn)
        .ok();

    let ma50 = tickers
        .filter(pair.eq(currency))
        .order(timestamp.desc())
        .limit(50)
        .select(avg(last))
        .first::<Option<f64>>(conn)
        .ok();

    match (ma5, ma50) {
        (Some(ma5), Some(ma50)) => {
            if ma5 > ma50 {
                Ok(TradeSignal::Buy) // ゴールデンクロス
            } else if ma5 < ma50 {
                Ok(TradeSignal::Sell) // デッドクロス
            } else {
                Ok(TradeSignal::Hold)
            }
        }
        _ => Ok(TradeSignal::InsufficientData) // データ不足
    }
}
