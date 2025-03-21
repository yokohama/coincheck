use std::thread;
use std::time::Duration;
use dotenvy::dotenv;
use log::{info, error};
use simplelog::{Config, LevelFilter, SimpleLogger};

use tokio;

use coincheck::{
    api, 
    repositories,
    models,
    db::establish_connection,
    error::AppError,
};

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
    let mut conn = pool.get()?;
    let client = api::coincheck::client::CoincheckClient::new()?;

    let my_balancies = repositories::balance::my_balancies(&client).await?;
    let my_trading_currencies = repositories::balance::my_trading_currencies(&client).await?;

    let mut new_summary_records: Vec<models::summary_record::NewSummaryRecord> = Vec::new();
    let mut total_jpy_value: f64 = 0.0;

    if let Some(balances) = my_balancies.as_object() {
        for currency in my_trading_currencies.iter() {
            if let Some(amount) = balances.get(currency).and_then(|v| v.as_f64()) {

                thread::sleep(Duration::from_millis(500));

                let rate = api::coincheck::rate::find(&client, &currency).await?;
                let jpy_value = amount * rate.sell_rate;

                new_summary_records.push(models::summary_record::NewSummaryRecord {
                    summary_id: None,
                    currency: currency.to_string(),
                    amount,
                    rate: rate.sell_rate,
                    jpy_value,
                });

                total_jpy_value += jpy_value;

            } else {
                println!("hoge");
            }
        }
    } else {
        println!("JSON is not an object");
    }

    let total_invested = repositories::transaction::total_invested(&mut conn)?;
    let pl = total_jpy_value - total_invested;

    let new_summary = models::summary::NewSummary {
        total_invested,
        total_jpy_value,
        pl,
    };

    repositories::summary::create(&mut conn, &new_summary, new_summary_records)?;

    api::slack::send_summary(new_summary).await?;
    info!("Execute summary successful.");

    Ok(())
}
