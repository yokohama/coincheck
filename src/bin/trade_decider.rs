use dotenvy::dotenv;
use std::thread;
use std::time::Duration;

use log::{info, error};
use simplelog::{Config, LevelFilter, SimpleLogger};
use tokio;

use coincheck::error::AppError;
use coincheck::db::establish_connection;
use coincheck::repositories;
use coincheck::api;
use coincheck::repositories::ticker::TradeSignal;
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

    let my_trading_currency = repositories::balance::my_trading_currencies(&client).await?;

    let mut new_orders: Vec<NewOrder> = Vec::new();
    for currency in my_trading_currency.iter() {
        let signal = repositories::ticker::determine_trade_signal(
            &mut conn, 
            currency
        ).unwrap();

        match signal {
            TradeSignal::Buy => {
                let new_order = NewOrder {
                    rate: Some(0.0),
                    pair: currency.clone(),
                    order_type: "buy".to_string(),
                    amount: 1.1,
                };
                new_orders.push(new_order);
            },
            TradeSignal::Sell => {
                let new_order = NewOrder {
                    rate: Some(0.0),
                    pair: currency.clone(),
                    order_type: "sell".to_string(),
                    amount: 1.1,
                };
                new_orders.push(new_order);
            },
            TradeSignal::Hold => {},
            TradeSignal::InsufficientData => {},
        }
    };

    for new_order in new_orders.iter() {
        repositories::order::create(&mut conn, new_order.clone())?;
        thread::sleep(Duration::from_millis(500));
    }

    api::slack::send_orderd_information(new_orders).await?;

    info!("Execute trade_decider successful.");

    Ok(())
}
