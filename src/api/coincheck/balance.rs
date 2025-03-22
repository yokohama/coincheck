use serde_json::Value;

use::reqwest::Client;

use crate::api::coincheck::{private, client::CoincheckClient};
use crate::error::AppError;

pub fn curl(coincheck_client: &CoincheckClient) -> Result<String, AppError> {
    let endpoint = format!("{}{}", coincheck_client.base_url, "/api/accounts/balance");
    let headers = private::headers(&endpoint, &coincheck_client, None)?;

    let access_key = headers["ACCESS-KEY"]
        .to_str().map_err(|e| AppError::InvalidData(format!("{}", e).to_string()))?;
    let access_nonce = headers["ACCESS-NONCE"]
        .to_str().map_err(|e| AppError::InvalidData(format!("{}", e).to_string()))?;
    let access_signature = headers["ACCESS-SIGNATURE"]
        .to_str().map_err(|e| AppError::InvalidData(format!("{}", e).to_string()))?;

    Ok(format!(
        "curl -X GET {} -H \"ACCESS-KEY: {}\" -H \"ACCESS-NONCE: {}\" -H \"ACCESS-SIGNATURE: {}\" -H \"Content-Type: application/json\"",
        endpoint,
        access_key,
        access_nonce,
        access_signature
    ))
}

pub async fn find(coincheck_client: &CoincheckClient) -> Result<Value, AppError> {
    let endpoint = format!("{}{}", coincheck_client.base_url, "/api/accounts/balance");
    let headers = private::headers(&endpoint, &coincheck_client, None)?;

    let client = Client::new();
    let response = client
        .get(endpoint)
        .headers(headers)
        .send()
        .await?;

    Ok(response.json().await?)
}
