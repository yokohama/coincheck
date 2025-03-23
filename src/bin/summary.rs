use dotenvy::dotenv;
use log::{info, error};
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

    let mut report = repositories::summary::make_report(&mut conn, &client).await?;
    repositories::summary::create(&mut conn, &report.summary, &mut report.summary_records)?;
    api::slack::send_summary("本日のレポート", &report.summary, report.summary_records).await?;
    info!("Execute summary successful.");

    Ok(())
}
