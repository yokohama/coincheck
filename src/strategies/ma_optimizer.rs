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
 * optimized_masテーブルから勝率の高い、ma_shortとma_longを読み込んでcrossoverを計算。
 *
 * [cron]
 * 2分毎に、cargo run --bin ticker_fetcherを実行して、tickersに情報を蓄積
 * 15毎に、cargo run --bin orderを実行して、注文
 * 
 * [envの設定]
 * BUY_THRESHOLD_1=20000
 * BUY_RATIO_1=0.9
 * BUY_THRESHOLD_2=50000
 * BUY_RATIO_2=0.7
 * BUY_THRESHOLD_3=150000
 * BUY_RATIO_3=0.5
 * SELL_RATIO=0.4
 */

pub struct MaOptimizerStrategy;

#[derive(QueryableByName)]
#[allow(dead_code)]
pub struct AvgResult {
    #[diesel(sql_type = Nullable<Double>)]
    avg: Option<f64>,
}

#[async_trait]
#[allow(dead_code)]
impl Strategy for MaOptimizerStrategy {
    async fn determine_trade_signal(
        &self,
        conn: &mut PgConnection,
        currency: &str,
        current_bid: f64,
        current_ask: f64,
        crypto_balance: f64,
    ) -> Result<TradeSignal, AppError> {
        dotenv().ok();

        let ma_border_threshold_ratio = env::var("MA_BORDER_THRESHOLD_RATIO")
            .unwrap_or("60.0".to_string())
            .parse::<f64>()
            .map_err(|e| AppError::InvalidData(format!("Parse error: {}", e)))?;

        let (sma_short, sma_long, win_rate_pct) = match models::optimized_ma::OptimizedMa::find_best_for_ma(conn, currency)? {
            Some((short, long, win_rate)) if win_rate >= ma_border_threshold_ratio => (short, long, win_rate),
            Some((short, long, win_rate)) => {
                let reason = format!("クロスの勝率:[{}% < {}%]、見送り", win_rate, ma_border_threshold_ratio);
                return Ok(
                    TradeSignal::Hold { 
                        spread_threshold: None,
                        spread_ratio: None,
                        ma_short: Some(short),
                        ma_long: Some(long),
                        ma_win_rate: Some(win_rate),
                        reason: Some(reason) 
                    });
            },
            None => return Ok(TradeSignal::Hold { 
                spread_threshold: None,
                spread_ratio: None,
                ma_short: None,
                ma_long: None,
                ma_win_rate: None,
                reason: Some("ベストなmaなし".to_string()) 
            }),
        };

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
    
        let a_spread_ratio = ((current_ask - current_bid) / current_bid) * 100.0;
        let a_spread_threshold = models::ticker::get_dynamic_spread_threshold(conn, currency).await?;
        if a_spread_ratio > a_spread_threshold {
            return Ok(TradeSignal::Hold { 
                spread_threshold: Some(a_spread_threshold),
                spread_ratio: Some(a_spread_ratio),
                ma_short: Some(sma_short),
                ma_long: Some(sma_long),
                ma_win_rate: Some(win_rate_pct),
                reason: Some("スプレッド負け".to_string()) 
            });
        }
    
        match (ma_short_avg, ma_long_avg) {
            (Some(short_avg), Some(long_avg)) => {
                if short_avg > long_avg {
                    // ゴールデンクロス(0.0の仮値をセット)
                    // すべてjpyで購入なので、呼び出し元で他購入通貨とのバランスを計算して再セットする。
                    let reason = format!("jpy_amount分{}を購入", currency);
                    Ok(TradeSignal::MarcketBuy { 
                        spread_threshold: Some(a_spread_threshold),
                        spread_ratio: Some(a_spread_ratio),
                        ma_short: Some(sma_short),
                        ma_long: Some(sma_long),
                        ma_win_rate: Some(win_rate_pct),
                        amount: 0.0, 
                        reason: Some(reason) 
                    })
    
                } else if short_avg < long_avg {
                    // デッドクロス
                    // こちらは仮想通貨毎に売る量を決定できるので、ここでセット
                    // TODO: マジックナンバー。0.001はbtc最低売却量のthreshold。マップでもたせる。
                    let amount = crypto_balance * sell_ratio;
                    if amount < 0.001 {
                        let reason = format!("最低売却量未満: {}", amount);
                        return Ok(TradeSignal::Hold { 
                            spread_threshold: Some(a_spread_threshold),
                            spread_ratio: Some(a_spread_ratio),
                            ma_short: Some(sma_short),
                            ma_long: Some(sma_long),
                            ma_win_rate: Some(win_rate_pct),
                            reason: Some(reason) 
                        })
                    }
                    let reason = format!("{}{}を売却", amount, currency);
                    Ok(TradeSignal::MarcketSell { 
                        spread_threshold: Some(a_spread_threshold),
                        spread_ratio: Some(a_spread_ratio),
                        ma_short: Some(sma_short),
                        ma_long: Some(sma_long),
                        ma_win_rate: Some(win_rate_pct),
                        amount, reason: Some(reason) 
                    })
    
                } else {
                    let reason = format!(
                        "こんなことあるのか？: short_avg={:#?} long_avg={:#?}", 
                        ma_short_avg, 
                        ma_long_avg);
                    Ok(TradeSignal::Hold { 
                        spread_threshold: Some(a_spread_threshold),
                        spread_ratio: Some(a_spread_ratio),
                        ma_short: Some(sma_short),
                        ma_long: Some(sma_long),
                        ma_win_rate: Some(win_rate_pct),
                        reason: Some(reason) 
                    })
                }
            },
            _ => Ok(TradeSignal::InsufficientData { 
                spread_threshold: None,
                spread_ratio: None,
                ma_short: None,
                ma_long: None,
                ma_win_rate: None,
                reason: Some("データ不足".to_string()) 
            })
        }
    }
}
