use std::env;
use dotenvy::dotenv;

use diesel::prelude::*;

use crate::error::AppError;
use crate::api::{coincheck, slack};
use crate::models::order::{NewOrder, Order};
use diesel::sql_types::{Nullable, Double, Text, Integer};

#[allow(dead_code)]
pub enum TradeSignal {
    MarcketBuy(f64),  //f64: buy amount
    MarcketSell(f64), //f64: sell amount
    Hold,
    InsufficientData,
}

#[derive(QueryableByName)]
#[allow(dead_code)]
pub struct AvgResult {
    #[diesel(sql_type = Nullable<Double>)]
    avg: Option<f64>,
}

#[allow(dead_code)]
pub async fn post_market_order(
    conn: &mut PgConnection, 
    client: &coincheck::client::CoincheckClient,
    new_order: NewOrder
) -> Result<(), AppError> {

    let status = coincheck::order::post_market_order(
        client, 
        new_order.pair.as_str(), 
        new_order.order_type.as_str(),
        new_order.amount
    ).await?;

    if status.is_success() {
        Order::create(conn, &new_order)?;
        slack::send_orderd_information(&new_order).await?;
    }

    Ok(())
}

#[allow(dead_code)]
pub fn determine_trade_signal(
    conn: &mut PgConnection,
    currency: &str,
    current_bid: f64,
    current_ask: f64,
    crypto_balance: f64,
) -> Result<TradeSignal, AppError> {
    dotenv().ok();

    let sma_short = env::var("MA_SHORT")?.parse::<i32>()
        .map_err(|e| AppError::InvalidData(format!("Parse error: {}", e)))?;

    let sma_long = env::var("MA_LONG")?.parse::<i32>()
        .map_err(|e| AppError::InvalidData(format!("Parse error: {}", e)))?;

    let spread_threshold = env::var("SPREAD_THRESHOLD")?.parse::<f64>()
        .unwrap_or(1.0);

    let sell_ratio = env::var("SELL_RATIO")?.parse::<f64>()
        .unwrap_or(0.5);

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

    let spread_ratio = ((current_ask - current_bid) / current_bid) * 100.0;
    if spread_ratio > spread_threshold {
        // スプレッド負けするので見送り。
        return Ok(TradeSignal::Hold);
    }

    match (ma_short_avg, ma_long_avg) {
        (Some(short_avg), Some(long_avg)) => {
            if short_avg > long_avg {
                // ゴールデンクロス
                // 0.0の仮値をセット。
                // すべてjpyで購入なので、呼び出し元で他購入通貨とのバランスを計算して再セットする。
                Ok(TradeSignal::MarcketBuy(0.0))

            } else if short_avg < long_avg {
                // デッドクロス
                // こちらは仮想通貨毎に売る量を決定できるので、ここでセット
                let amount = crypto_balance * sell_ratio;
                Ok(TradeSignal::MarcketSell(amount))

            } else {
                Ok(TradeSignal::Hold)
            }
        }
        _ => Ok(TradeSignal::InsufficientData) // データ不足
    }
}
