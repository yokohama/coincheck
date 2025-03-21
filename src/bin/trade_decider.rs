use dotenvy::dotenv;

use log::{info, error};
use simplelog::{Config, LevelFilter, SimpleLogger};
use tokio;

use coincheck::error::AppError;
use coincheck::db::establish_connection;
use coincheck::repositories;
use coincheck::api;
use coincheck::repositories::ticker::TradeSignal;

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

    for currency in my_trading_currency.iter() {
        let signal = repositories::ticker::determine_trade_signal(
            &mut conn, 
            currency
        ).unwrap();

        match signal {
            TradeSignal::Buy => {
                api::slack::send_orderd_information(&currency, "buy", 1.1).await?;
            },
            TradeSignal::Sell => {
                api::slack::send_orderd_information(&currency, "sell", 1.1).await?;
            },
            TradeSignal::Hold => {
                api::slack::send_orderd_information(&currency, "hold", 0.0).await?;
            },
            TradeSignal::InsufficientData => {
                api::slack::send_orderd_information(&currency, "insufficient data", 0.0).await?;
            },
        }
    };

    info!("Execute trade_decider successful.");

    Ok(())
}
