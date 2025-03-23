use dotenvy::dotenv;

use reqwest::Client;

use crate::error::AppError;
use crate::api::coincheck::client;
use crate::models::ticker::NewTicker;

#[allow(dead_code)]
pub async fn find(
    coincheck_client: &client::CoincheckClient,
    currency: &str,
) -> Result<NewTicker, AppError> {
    dotenv().ok();

    let path = format!("/api/ticker?pair={}_jpy", currency);
    let endpoint = format!("{}{}", coincheck_client.base_url, path);

    let ticker = Client::new()
        .get(&endpoint)
        .send()
        .await?
        .json::<NewTicker>()
        .await?;

    client::sleep()?;
    Ok(ticker)
}
