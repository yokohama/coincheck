use serde_json::Value;
use::reqwest::Client;

use crate::api::coincheck::{private, client};
use crate::error::AppError;

pub async fn find(coincheck_client: &client::CoincheckClient) -> Result<Value, AppError> {
    let endpoint = format!("{}{}", coincheck_client.base_url, "/api/accounts/balance");
    let headers = private::headers(&endpoint, &coincheck_client, None)?;

    let client = Client::new();
    let response = client
        .get(endpoint)
        .headers(headers)
        .send()
        .await?;

    client::sleep()?;

    let mut json: serde_json::Value = response.json().await?;

    /* 
     * TODO: ウォレットからshibが消せないので、ここでハードコーディングで削除。
     * 将来的に、ignore_currenciesのように変数化
     */
    if let Some(obj) = json.as_object_mut() {
        obj.remove("shib");
    }

    Ok(json)
}
