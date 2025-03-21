use serde_json::Value;

use::reqwest::Client;

use crate::api::coincheck::{private, client::CoincheckClient};
use crate::error::AppError;

pub async fn find(coincheck_client: &CoincheckClient) -> Result<Value, AppError> {
    let endpoint = format!("{}{}", coincheck_client.base_url, "/api/accounts/balance");

    let headers = private::headers(&endpoint, &coincheck_client)?;

    let client = Client::new();
    let response = client
        .get(endpoint)
        .headers(headers)
        .send()
        .await?;

    Ok(response.json().await?)
}
