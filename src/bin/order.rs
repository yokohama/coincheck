use dotenvy::dotenv;
use std::thread;
use std::time::Duration;

use log::error;
use simplelog::{Config, LevelFilter, SimpleLogger};
use tokio;

use coincheck::error::AppError;
use coincheck::db::establish_connection;
use coincheck::api;
use coincheck::repositories::{self, ticker::TradeSignal};
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

        let signal = repositories::ticker::determine_trade_signal(
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

    for new_order in new_orders.iter() {
        repositories::order::post_market_order(
            &mut conn,
            &client,
            new_order.clone()
        ).await?;
    }

    Ok(())
}
