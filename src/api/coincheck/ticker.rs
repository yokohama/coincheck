use reqwest::Client;

use crate::error::AppError;
use crate::api::coincheck::client::CoincheckClient;
use crate::models::ticker::NewTicker;

pub async fn find(
    coincheck_client: &CoincheckClient,
    currency: &str,
) -> Result<NewTicker, AppError> {

    let path = format!("/api/ticker?pair={}_jpy", currency);
    let endpoint = format!("{}{}", coincheck_client.base_url, path);

    let ticker = Client::new()
        .get(&endpoint)
        .send()
        .await?
        .json::<NewTicker>()
        .await?;

    Ok(ticker)
}
