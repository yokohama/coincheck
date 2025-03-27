use std::env;
use dotenvy::dotenv;

use async_trait::async_trait;

use diesel::prelude::*;
use diesel::sql_types::{Nullable, Double, Text, Integer};

use crate::{
    models,
    error::AppError,
    strategies::{
        strategy_trait::Strategy,
        trade_signal::TradeSignal,
    },
};

/*
 * [strategy]
 * envから、ma_shortとma_longを読み込んでcrossoverを計算。
 *
 * [cron]
 * 2分毎に、cargo run --bin ticker_fetcherを実行して、tickersに情報を蓄積
 * 15毎に、cargo run --bin orderを実行して、注文
 * 
 * [envの設定]
 * MA_SHORT=10
 * MA_LONG=30
 * BUY_THRESHOLD_1=20000
 * BUY_RATIO_1=0.9
 * BUY_THRESHOLD_2=50000
 * BUY_RATIO_2=0.7
 * BUY_THRESHOLD_3=150000
 * BUY_RATIO_3=0.5
 * SELL_RATIO=0.4
 */

pub struct BasicStrategy;

#[derive(QueryableByName)]
#[allow(dead_code)]
pub struct AvgResult {
    #[diesel(sql_type = Nullable<Double>)]
    avg: Option<f64>,
}

#[async_trait]
#[allow(dead_code)]
impl Strategy for BasicStrategy {
    async fn determine_trade_signal(
        &self,
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
    
        let sell_ratio = env::var("SELL_RATIO")?.parse::<f64>().unwrap();
    
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
        let spread_threshold = models::ticker::get_dynamic_spread_threshold(conn, currency).await?;
        if spread_ratio > spread_threshold {
            let reason = format!(
                "スプレッド負け: spread_ratio={} spread_threshold={}", 
                spread_ratio, 
                spread_threshold);
            return Ok(TradeSignal::Hold { reason: Some(reason) });
        }
    
        match (ma_short_avg, ma_long_avg) {
            (Some(short_avg), Some(long_avg)) => {
                if short_avg > long_avg {
                    // ゴールデンクロス
                    // 0.0の仮値をセット。
                    // すべてjpyで購入なので、呼び出し元で他購入通貨とのバランスを計算して再セットする。
                    Ok(TradeSignal::MarcketBuy { amount: 0.0, reason: None })
    
                } else if short_avg < long_avg {
                    // デッドクロス
                    // こちらは仮想通貨毎に売る量を決定できるので、ここでセット
                    let amount = crypto_balance * sell_ratio;
                    Ok(TradeSignal::MarcketSell { amount, reason: None })
    
                } else {
                    let reason = format!(
                        "こんなことあるのか？: short_avg={:#?} long_avg={:#?}", 
                        ma_short_avg, 
                        ma_long_avg);
                    Ok(TradeSignal::Hold { reason: Some(reason) })
                }
            }
            _ => Ok(TradeSignal::InsufficientData { reason: Some("データ不足".to_string()) })
        }
    }
}
