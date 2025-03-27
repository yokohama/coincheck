use std::env;
use log::{info, error};

use diesel::prelude::*;

use crate::{
    api::{coincheck, slack},
    repositories,
    models,
    error::AppError,
};

use crate::strategies::trade_signal::TradeSignal;
use crate::strategies::strategy_trait::Strategy;
use crate::strategies::ma_optimizer::MaOptimizerStrategy;

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

       let mut new_order = models::order::NewOrder {
           rate: Some(0.0),
           buy_rate: Some(0.0),
           sell_rate: Some(0.0),
           pair: currency.clone(),
           order_type: "".to_string(),
           jpy_amount: 0.0,
           crypto_amount: 0.0,
           spread_ratio: Some(0.0),
           api_error_msg: None,
       };

       let strategy = MaOptimizerStrategy;

       match strategy.determine_trade_signal(
           conn,
           currency,
           ticker.bid,
           ticker.ask,
           crypto_balance,
       ).await {
           Ok(s) => {
             match s {
               TradeSignal::MarcketBuy { amount, .. } => {
                   new_order.order_type = "market_buy".to_string();
                   new_order.jpy_amount = amount;
                   new_orders.push(new_order);
               },
               TradeSignal::MarcketSell { amount, .. } => {
                   new_order.order_type = "market_sell".to_string();
                   new_order.crypto_amount = amount;
                   new_orders.push(new_order);
               },
               TradeSignal::Hold { reason } => {
                   new_order.order_type = "hold".to_string();
                   new_order.api_error_msg = reason.clone();
                   new_orders.push(new_order);
                   //info!("{}", reason.unwrap());
               },
               TradeSignal::InsufficientData { reason } => {
                   new_order.order_type = "insufficient_data".to_string();
                   new_order.api_error_msg = reason.clone();
                   new_orders.push(new_order);
                   //info!("{}", reason.unwrap());
               },
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
        let amount;
        if new_order.order_type == "market_buy" {
            new_order.jpy_amount = jpy_amount_per_currency;
            amount = jpy_amount_per_currency;
        } else if new_order.order_type == "market_sell" {
            amount = new_order.crypto_amount;
        } else {
            models::order::Order::create(conn, &new_order)?;
            continue;
        };

        let (status, body) = coincheck::order::post_market_order(
            client, 
            new_order.pair.as_str(), 
            new_order.order_type.as_str(),
            amount
        ).await?;

        let orderd_rate = coincheck::rate::find(client, new_order.pair.as_str()).await?;
        new_order.buy_rate = Some(orderd_rate.buy_rate);
        new_order.sell_rate = Some(orderd_rate.sell_rate);
        new_order.spread_ratio = Some(orderd_rate.spread_ratio);

        info!("#-- [ {} ]", new_order.order_type);
        info!("# pair: {}", new_order.pair);
        info!("# crypt_amount: {}", new_order.crypto_amount);
        info!("# jpy_amount: {}", new_order.jpy_amount);
        info!("# api_error_msg: {:?}", new_order.api_error_msg);
        println!("");

        if status.is_success() {
            slack::send_orderd_information(&new_order).await?;
            success_order_count += 1;
        } else {
            new_order.api_error_msg = Some(body.get("error").unwrap().to_string());
        }

        models::order::Order::create(conn, &new_order)?;
    }

    if success_order_count > 0 {
        let mut report = repositories::summary::make_report(conn, &client).await?;
        models::summary::Summary::create(conn, &report.summary, &mut report.summary_records)?;
        slack::send_summary("直近レポート", &report.summary, report.summary_records).await?;
    } else {
        info!("summary: オーダーなし");
    }

    Ok(())
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
