use std::error::Error;
use dotenvy::dotenv;

use tokio;

use coincheck::db::establish_connection;
use coincheck::api;
use coincheck::models;
use coincheck::repositories;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().expect(".env file not found");

    let pool = establish_connection();
    let mut conn = pool.get().expect("Failed to get DB connection");
    let client = api::coincheck::client::CoincheckClient::new()?;

    let my_currencies = repositories::balance::my_currencies(&client).await?;
    let my_pairs = repositories::balance::my_pairs(&my_currencies);

    // tickerを更新
    for currency in my_pairs.iter() {
        let mut new_ticker = api::coincheck::ticker::find(&client, &currency).await?;
        new_ticker.pair = Some(currency.to_string());
        models::ticker::Ticker::create(&mut conn, new_ticker)?;
    };

    Ok(())
}
