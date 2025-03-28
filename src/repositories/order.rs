use std::env;
use log::{info, error};

use diesel::prelude::*;
use serde_json::Value;

use crate::{
    api::{coincheck, slack}, 
    error::AppError, 
    models::{self, order::NewOrder}, 
    repositories
};

use crate::strategies::strategy_trait::Strategy;
use crate::strategies::ma_optimizer::MaOptimizerStrategy;

#[allow(dead_code)]
pub async fn post_market_order(
    conn: &mut PgConnection,
    client: &coincheck::client::CoincheckClient,
) -> Result<(), AppError> {

    // ストラテジーの切替え
    let strategy = MaOptimizerStrategy;

    // 全体の資産情報の取得
    let Some((balances, my_managed_balances, my_trading_currency, jpy_balance)) = 
        fetch_balances(client).await? else {
        return Err(AppError::InvalidData("事前の資産情報が見つかりませんでした。".to_string()));
    };

    print_log_header(my_managed_balances);

    // new_ordersに、通過毎のオーダーの内容をプッシュしてまとめていく
    let mut new_orders: Vec<models::order::NewOrder> = Vec::new();

    // 通貨毎のオーダーの作成と、new_ordersにプッシュ
    for currency in my_trading_currency.iter() {

        // 通貨情報の取得
        let Some((ticker, crypto_balance)) = 
            fetch_ticker_and_crypto_balance(client, currency, &balances).await? else {
            continue;
        };

        let mut new_order = models::order::NewOrder::new(currency.clone());

        // 戦略に合わせて、通貨毎に注文内容を決定して、new_owdersにプッシュ
        match strategy.determine_trade_signal(
            conn,
            currency,
            ticker.bid,
            ticker.ask,
            crypto_balance,
        ).await {
            Ok(signal) => {
                signal.apply_to(&mut new_order);
                new_orders.push(new_order);
            },
            Err(e) => {
                error!("#- [{}] signal取得失敗: {}", currency, e);
                continue;
            }
        };
    };

    // 購入と判断した注文数で、使えるJPYを等分
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

        info!("{}", new_order.order_type);
        info!("{}", new_order.order_type);
        info!("{}", new_order.order_type);
        info!("{}", new_order.order_type);

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

    if success_order_count > 0 { make_summary(conn, client).await?; }

    Ok(())
}

fn print_log_header(my_managed_balances: Value) {
    info!("#");
    info!("# オーダー情報");
    info!("#");
    info!("balance: {:#?}", my_managed_balances);
    println!("");
}

async fn fetch_balances(
    client: &coincheck::client::CoincheckClient
) -> Result<Option<(Value, Value, Vec<String>, f64)>, AppError> {
    let balances = repositories::balance::my_balancies(&client).await?;
    let my_managed_balances = repositories::balance::my_managed_balancies(&balances)?;
    let my_trading_currency = repositories::balance::my_trading_currencies(&client).await?;
    let jpy_balance = repositories::balance::get_jpy_balance(&balances)?;

    Ok(Some((balances, my_managed_balances, my_trading_currency, jpy_balance)))
}

async fn fetch_ticker_and_crypto_balance(
    client: &coincheck::client::CoincheckClient,
    currency: &str,
    balances: &Value,
) -> Result<Option<(models::ticker::NewTicker, f64)>, AppError> {
    let ticker = match coincheck::ticker::find(client, currency).await {
        Ok(t) => t,
        Err(e) => {
            error!("#- [{}] ticker取得失敗: api非対応通貨の可能性: {}", currency, e);
            return Ok(None);
        }
    };

    let crypto_balance = match repositories::balance::get_crypto_balance(balances, currency) {
        Ok(b) => b,
        Err(_) => {
            error!("");
            return Ok(None);
        }
    };

    Ok(Some((ticker, crypto_balance)))
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
    conn: &mut PgConnection, 
    client: &coincheck::client::CoincheckClient
) -> Result<(), AppError> {
    let mut report = repositories::summary::make_report(conn, &client).await?;
    models::summary::Summary::create(conn, &report.summary, &mut report.summary_records)?;
    slack::send_summary("直近レポート", &report.summary, report.summary_records).await?;

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
