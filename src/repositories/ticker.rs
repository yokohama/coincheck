use std::env;
use dotenvy::dotenv;

use diesel::prelude::*;
use diesel::sql_types::{Nullable, Double, Text, Integer};
use diesel::QueryableByName;

use crate::error::AppError;
use crate::models;

pub enum TradeSignal {
    Buy,
    Sell,
    Hold,
    InsufficientData,
}

#[derive(QueryableByName)]
pub struct AvgResult {
    #[diesel(sql_type = Nullable<Double>)]
    avg: Option<f64>,
}

pub fn create(
    conn: &mut PgConnection, 
    new_ticker: models::ticker::NewTicker
) -> Result<(), AppError> {

    models::ticker::Ticker::create(conn, new_ticker)
}

pub fn determine_trade_signal(
    conn: &mut PgConnection,
    currency: &str,
) -> Result<TradeSignal, AppError> {

    dotenv().ok();

    let sma_short = env::var("MA_SHORT")?
        .parse::<i32>()
        .map_err(|e| AppError::InvalidData(format!("Parse error: {}", e)))?;
    let sma_long = env::var("MA_LONG")?
        .parse::<i32>()
        .map_err(|e| AppError::InvalidData(format!("Parse error: {}", e)))?;
    let ma_crossover_buy_threshold = env::var("MA_CROSSOVER_BUY_THRESHOLD")?
        .parse::<f64>()
        .map_err(|e| AppError::InvalidData(format!("Parse error: {}", e)))?;
    let ma_crossover_sell_threshold = env::var("MA_CROSSOVER_SELL_THRESHOLD")?
        .parse::<f64>()
        .map_err(|e| AppError::InvalidData(format!("Parse error: {}", e)))?;

    let periods = vec![sma_short, sma_long];
    let mut results: Vec<Option<AvgResult>> = Vec::new();

    for period in periods.iter() {
        let ma: Option<AvgResult> = diesel::sql_query("
            SELECT AVG(subquery.last) AS avg
            FROM (
                SELECT last
                FROM tickers
                WHERE pair = $1
                ORDER BY timestamp DESC
                LIMIT $2
            ) AS subquery
        ")
        .bind::<Text, _>(currency)
        .bind::<Integer, _>(period)
        .get_result(conn)
        .optional()?;

        results.push(ma);
    }

    let ma_short_avg = results[0].as_ref().map(|r| r.avg).flatten();
    let ma_long_avg = results[1].as_ref().map(|r| r.avg).flatten();

    match (ma_short_avg, ma_long_avg) {
        (Some(short_avg), Some(long_avg)) => {
            if short_avg > long_avg * ma_crossover_buy_threshold {
                Ok(TradeSignal::Buy) // ゴールデンクロス
            } else if short_avg < long_avg * ma_crossover_sell_threshold {
                Ok(TradeSignal::Sell) // デッドクロス
            } else {
                Ok(TradeSignal::Hold)
            }
        }
        _ => Ok(TradeSignal::InsufficientData) // データ不足
    }
}
