use std::env;
use dotenvy::dotenv;
use log::{info, error};

use diesel::prelude::*;

use crate::{
    api::{coincheck, slack},
    repositories,
    models,
    error::AppError,
};
use diesel::sql_types::{Nullable, Double, Text, Integer};

/*
 * [cron]
 * 2分毎に、cargo run --bin ticker_fetcherを実行して、tickersに情報を蓄積
 * 30毎に、cargo run --bin orderを実行して、注文
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
) -> Result<(), AppError> {
    let balancies = repositories::balance::my_balancies(&client).await?;
    let my_trading_currency = repositories::balance::my_trading_currencies(&client).await?;
    let jpy_balance = repositories::balance::get_jpy_balance(&balancies)?;

    info!("#");
    info!("# オーダー情報");
    info!("#");
    info!("balance: {:#?}", balancies);
    println!("");

    let mut new_orders: Vec<models::order::NewOrder> = Vec::new();
    for currency in my_trading_currency.iter() {
        let ticker = match coincheck::ticker::find(&client, &currency).await {
            Ok(ticker) => ticker,
            Err(e) => {
                error!("#- [{}] ticker取得失敗: api非対応通貨の可能性: {}", currency, e);
                continue;
            }
        };

        let crypto_balance = match repositories::balance::get_crypto_balance(&balancies, currency) {
            Ok(b) => b,
            Err(_) => {
                error!("#- [{}] balance取得失敗: データ不足", currency);
                continue;
            }
        };

        match determine_trade_signal(
            conn, 
            currency,
            ticker.bid,
            ticker.ask,
            crypto_balance,
        ).await {
            Ok(s) => {
              match s {
                TradeSignal::MarcketBuy(amount) => {
                    let new_order = models::order::NewOrder {
                        rate: Some(0.0),
                        buy_rate: Some(0.0),
                        sell_rate: Some(0.0),
                        pair: currency.clone(),
                        order_type: "market_buy".to_string(),
                        jpy_amount: amount,
                        crypto_amount: 0.0,
                        spread_ratio: Some(0.0),
                    };
                    new_orders.push(new_order);
                },
                TradeSignal::MarcketSell(amount) => {
                    let new_order = models::order::NewOrder {
                        rate: Some(0.0),
                        buy_rate: Some(0.0),
                        sell_rate: Some(0.0),
                        pair: currency.clone(),
                        order_type: "market_sell".to_string(),
                        jpy_amount: 0.0,
                        crypto_amount: amount,
                        spread_ratio: Some(0.0),
                    };
                    new_orders.push(new_order);
                },
                TradeSignal::Hold => {},
                TradeSignal::InsufficientData => {},
              }
            },
            Err(e) => {
                error!("#- [{}] signal取得失敗: {}", currency, e);
                continue;
            }
        };
    };

    // 購入と判断した通貨毎に使えるJPYを等分
    let jpy_amount_per_currency = get_buy_ratio(jpy_balance, new_orders.len() as i32)?;

    // 売りが先にくるようにソート
    new_orders.sort_by_key(|order| {
        if order.order_type == "sell" { 0 } else { 1 }
    });

    let mut success_order_count = 0;
    for new_order in new_orders.iter_mut() {

        let mut amount = 0.0;
        if new_order.order_type == "market_buy" {
            new_order.jpy_amount = jpy_amount_per_currency;
            amount = jpy_amount_per_currency;
        } else {
            amount = new_order.crypto_amount;
        };

        let (status, _) = coincheck::order::post_market_order(
            client, 
            new_order.pair.as_str(), 
            new_order.order_type.as_str(),
            amount
        ).await?;

        if status.is_success() {
            let orderd_rate = coincheck::rate::find(client, new_order.pair.as_str()).await?;
            new_order.buy_rate = Some(orderd_rate.buy_rate);
            new_order.sell_rate = Some(orderd_rate.sell_rate);
            new_order.spread_ratio = Some(orderd_rate.spread_ratio);

            models::order::Order::create(conn, &new_order)?;
            slack::send_orderd_information(&new_order).await?;
            success_order_count += 1;
        }

        println!("");
    }

    if success_order_count > 0 {
        let mut report = repositories::summary::make_report(conn, &client).await?;
        models::summary::Summary::create(conn, &report.summary, &mut report.summary_records)?;
        slack::send_summary("直近レポート", &report.summary, report.summary_records).await?;
    } else {
        info!("summary: オーダーなし");
    }

    println!("");
    Ok(())
}

#[allow(dead_code)]
pub async fn determine_trade_signal(
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
        return Ok(TradeSignal::Hold);
    }

    // TODO: ちゃんとログ出てない
    info!("short_avg={:#?}, long_avg={:#?}", ma_short_avg, ma_long_avg);
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

/*
 * 通貨毎に購入するJPYを算出(今は単純に等分している）。
 */
fn get_buy_ratio(jpy_balance: f64, new_orders_length: i32) -> Result<f64, AppError> {
    let threshold_1 = env::var("BUY_THRESHOLD_1")?.parse::<f64>().unwrap();
    let threshold_2 = env::var("BUY_THRESHOLD_2")?.parse::<f64>().unwrap();
    let threshold_3 = env::var("BUY_THRESHOLD_3")?.parse::<f64>().unwrap();
    
    let ratio_1 = env::var("BUY_RATIO_1")?.parse::<f64>().unwrap();
    let ratio_2 = env::var("BUY_RATIO_2")?.parse::<f64>().unwrap();
    let ratio_3 = env::var("BUY_RATIO_3")?.parse::<f64>().unwrap();
    let ratio_default = env::var("BUY_RATIO_DEFAULT")?.parse::<f64>().unwrap();
    
    let buy_ratio = if jpy_balance < threshold_1 {
        ratio_1
    } else if jpy_balance < threshold_2 {
        ratio_2
    } else if jpy_balance < threshold_3 {
        ratio_3
    } else {
        ratio_default
    };

    let jpy_amount = jpy_balance * buy_ratio;
    let jpy_amount_per_currency = jpy_amount / new_orders_length as f64;

    Ok(jpy_amount_per_currency)
}
