use dotenvy::dotenv;
use log::error;
use simplelog::{Config, LevelFilter, SimpleLogger};

use tokio;

use coincheck::{
    api,
    db::establish_connection,
    error::AppError,
    repositories,
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

    repositories::summary::reporing(&mut conn, &client).await?;

    Ok(())
}
