use std::error::Error;
use dotenvy::dotenv;

use tokio;

use coincheck::db::establish_connection;
use coincheck::api;
use coincheck::repositories;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().expect(".env file not found");

    let pool = establish_connection();
    let mut conn = pool.get().expect("Failed to get DB connection");
    let client = api::coincheck::client::CoincheckClient::new()?;

    let my_trading_currencies = repositories::balance::my_trading_currencies(&client).await?;

    for currency in my_trading_currencies.iter() {
        let mut new_ticker = api::coincheck::ticker::find(&client, &currency).await?;
        new_ticker.pair = Some(currency.to_string());
        repositories::ticker::create(&mut conn, new_ticker)?;
    };

    Ok(())
}
