use std::error::Error;
use dotenvy::dotenv;

use tokio;

use coincheck::db::establish_connection;
use coincheck::api;
use coincheck::models;

const CURRENCIES: &[&str] = &["btc", "eth", "shib", "avax"];

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().expect(".env file not found");

    let pool = establish_connection();
    let mut conn = pool.get().expect("Failed to get DB connection");
    let client = api::coincheck::client::CoincheckClient::new()?;

    // tickerを更新
    for currency in CURRENCIES.iter() {
        let mut new_ticker = api::coincheck::ticker::find(&client, &currency).await?;
        new_ticker.pair = Some(currency.to_string());
        models::ticker::Ticker::create(&mut conn, new_ticker)?;
    };

    Ok(())
}
