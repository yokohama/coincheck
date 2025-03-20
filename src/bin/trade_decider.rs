use std::error::Error;
use coincheck::repositories::ticker::TradeSignal;

use tokio;

use coincheck::db::establish_connection;
use coincheck::repositories;
use coincheck::api;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

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
            TradeSignal::Hold => {},
            TradeSignal::InsufficientData => {},
        }
    };

    Ok(())
}
