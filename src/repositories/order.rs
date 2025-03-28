use std::env;
use log::{info, error};

use diesel::prelude::*;

use crate::{
    api::{coincheck, slack}, error::AppError, models::{self, order::NewOrder}, repositories
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
    let my_managed_balanies = repositories::balance::my_managed_balancies(&balancies);
    let my_trading_currency = repositories::balance::my_trading_currencies(&client).await?;
    let jpy_balance = repositories::balance::get_jpy_balance(&balancies)?;

    info!("#");
    info!("# オーダー情報");
    info!("#");
    info!("balance: {:#?}", my_managed_balanies);
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

        let mut new_order = models::order::NewOrder::new(currency.clone());

        let strategy = MaOptimizerStrategy;
        match strategy.determine_trade_signal(
            conn,
            currency,
            ticker.bid,
            ticker.ask,
            crypto_balance,
        ).await {
            Ok(signal) => {
                let (order_type, jpy_amount, crypto_amount, comment) = match signal {
                    TradeSignal::MarcketBuy { amount, reason } => ("market_buy", amount, 0.0, reason.clone()),
                    TradeSignal::MarcketSell { amount, reason } => ("market_sell", 0.0, amount, reason.clone()),
                    TradeSignal::Hold { reason } => ("hold", 0.0, 0.0, reason.clone()),
                    TradeSignal::InsufficientData { reason } => ("insufficient_data", 0.0, 0.0, reason.clone()),
                };

                new_order.order_type = order_type.to_string();
                new_order.jpy_amount = jpy_amount;
                new_order.crypto_amount = crypto_amount;
                new_order.comment = comment;

                new_orders.push(new_order);
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
    for mut new_order in new_orders.iter_mut() {
        let amount;
        if new_order.order_type == "market_buy" {
            new_order.jpy_amount = jpy_amount_per_currency;
            amount = jpy_amount_per_currency;
        } else if new_order.order_type == "market_sell" {
            amount = new_order.crypto_amount;
        } else {
            print_log(&new_order);
            models::order::Order::create(conn, &new_order)?;
            continue;
        };

        let mut orderd = coincheck::order::post_market_order(client, &mut new_order, amount).await?;

        if orderd.api_call_success_at.is_some() {
            slack::send_orderd_information(&orderd).await?;

            let orderd_rate = coincheck::rate::find(client, orderd.pair.as_str()).await?;
            orderd.buy_rate = Some(orderd_rate.buy_rate);
            orderd.sell_rate = Some(orderd_rate.sell_rate);
            orderd.spread_ratio = Some(orderd_rate.spread_ratio);

            success_order_count += 1;
        }

        print_log(&orderd);
        models::order::Order::create(conn, &orderd)?;
    }

    make_summary(success_order_count, conn, client).await?;
    Ok(())
}

fn print_log(new_order: &NewOrder) {
    info!("#-- [ {} ]", new_order.order_type);
    info!("# pair: {}", new_order.pair);
    info!("# crypt_amount: {}", new_order.crypto_amount);
    info!("# jpy_amount: {}", new_order.jpy_amount);
    info!("# comment: {:?}", new_order.comment);
    println!("");
}

async fn make_summary(
    success_order_count: u32,
    conn: &mut PgConnection, 
    client: &coincheck::client::CoincheckClient
) -> Result<(), AppError> {
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
