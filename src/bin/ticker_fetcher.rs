use dotenvy::dotenv;

use log::{info, error};
use simplelog::{Config, LevelFilter, SimpleLogger};
use tokio;

use coincheck::error::AppError;
use coincheck::db::establish_connection;
use coincheck::api;
use coincheck::repositories;

#[tokio::main]
async fn main() {
    dotenv().ok();
    SimpleLogger::init(LevelFilter::Info, Config::default()).unwrap();

    if let Err(e) = run().await {
        error!("Error occurred: {}", e);
    }
}

async fn run() -> Result<(), AppError> {
    dotenv().expect(".env file not found");

    let pool = establish_connection();
    let mut conn = pool.get()?;
    let client = api::coincheck::client::CoincheckClient::new()?;

    let my_trading_currencies = repositories::balance::my_trading_currencies(&client).await?;

    for currency in my_trading_currencies.iter() {
        let mut new_ticker = api::coincheck::ticker::find(&client, &currency).await?;
        new_ticker.pair = Some(currency.to_string());
        repositories::ticker::create(&mut conn, new_ticker)?;
    };

    info!("Execute ticker_fetcher successful.");

    Ok(())
}
