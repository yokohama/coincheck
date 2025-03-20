use std::error::Error;

use tokio;

use coincheck::db::establish_connection;
use coincheck::repositories;
use coincheck::api;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let pool = establish_connection();
    let mut conn = pool.get().expect("Failed to get DB connection");
    let client = api::coincheck::client::CoincheckClient::new()?;

    let my_currencies = repositories::balance::my_currencies(&client).await?;
    let my_pairs = repositories::balance::my_pairs(&my_currencies);

    for currency in my_pairs.iter() {
        let signal = repositories::ticker::determine_trade_signal(&mut conn, currency).unwrap();
        println!("{}: {:?}", currency, signal);
    };
    println!("");

    Ok(())
}
