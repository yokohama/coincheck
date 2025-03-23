use std::env;
use dotenvy::dotenv;
use std::thread;
use std::time::Duration;

use log::{info, error};
use simplelog::{Config, LevelFilter, SimpleLogger};
use tokio;

use coincheck::error::AppError;
use coincheck::db::establish_connection;
use coincheck::api;
use coincheck::repositories::{self, order::TradeSignal};
use coincheck::models::order::NewOrder;

#[tokio::main]
async fn main() {
    dotenv().ok();
    SimpleLogger::init(LevelFilter::Info, Config::default()).unwrap();

    if let Err(e) = run().await {
        error!("Error occurred: {}", e);
    }
}

async fn run() -> Result<(), AppError> {
    let pool = establish_connection();
    let mut conn = pool.get().expect("Failed to get DB connection");
    let client = api::coincheck::client::CoincheckClient::new()?;

    let balancies = repositories::balance::my_balancies(&client).await?;
    let my_trading_currency = repositories::balance::my_trading_currencies(&client).await?;
    let jpy_balance = repositories::balance::get_jpy_balance(&balancies)?;

    let mut new_orders: Vec<NewOrder> = Vec::new();
    for currency in my_trading_currency.iter() {
        let ticker = api::coincheck::ticker::find(&client, &currency).await?;
        thread::sleep(Duration::from_millis(500));

        let signal = repositories::order::determine_trade_signal(
            &mut conn, 
            currency,
            ticker.bid,
            ticker.ask,
            jpy_balance,
            repositories::balance::get_crypto_balance(&balancies, &currency)?,
        ).map_err(|e| AppError::InvalidData(format!("{}", e)))?;

        match signal {
            TradeSignal::MarcketBuy(amount) => {
                let new_order = NewOrder {
                    rate: Some(0.0),
                    pair: currency.clone(),
                    order_type: "buy".to_string(),
                    amount,
                };
                new_orders.push(new_order);
            },
            TradeSignal::MarcketSell(amount) => {
                let new_order = NewOrder {
                    rate: Some(0.0),
                    pair: currency.clone(),
                    order_type: "sell".to_string(),
                    amount,
                };
                new_orders.push(new_order);
            },
            TradeSignal::Hold => {},
            TradeSignal::InsufficientData => {},
        }
    };

    let buy_ratio = env::var("BUY_RATIO")?
        .parse::<f64>()
        .map_err(|e| AppError::InvalidData(format!("Parse error: {}", e)))?;

    let jpy_amount = jpy_balance * buy_ratio;
    info!("# jpy_amount: {}", jpy_amount);
    let jpy_amount_per_currency = jpy_amount / new_orders.len() as f64;
    info!("# jpy_amount_per_currency: {}", jpy_amount_per_currency);

    let new_orders_count = new_orders.len();

    for new_order in new_orders.iter_mut() {
        if new_order.order_type == "buy" {
            info!("# buy: {}JPY", jpy_amount_per_currency);
            new_order.amount = jpy_amount_per_currency;
        };

        repositories::order::post_market_order(
            &mut conn,
            &client,
            new_order.clone()
        ).await?;
    }

    if new_orders_count > 0 {
        let report = repositories::summary::make_report(&mut conn, &client).await?;
        api::slack::send_summary("直近レポート", &report.summary, report.summary_records).await?;
    }

    Ok(())
}
