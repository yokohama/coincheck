use reqwest;
use serde::Deserialize;

use crate::api::coincheck::client;
use crate::error::AppError;

#[derive(Deserialize)]
pub struct FetchRate {
    pub rate: String,
}

impl FetchRate {
    pub fn to_f64(&self) -> Result<f64, AppError> {
        self.rate.parse::<f64>().map_err(|e| AppError::InvalidData(format!("Parse error: {}", e)))
    }
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct Rate {
    pub currency: String,
    pub buy_rate: f64,
    pub sell_rate: f64,
    pub spread_ratio: f64,
}

pub async fn find(client: &client::CoincheckClient, currency: &str) -> Result<Rate, AppError> {
    let buy_endpoint = format!(
        "{}/api/exchange/orders/rate?pair={}_jpy&order_type=buy&amount=1", 
        client.base_url, 
        &currency
    );
    let sell_endpoint = format!(
        "{}/api/exchange/orders/rate?pair={}_jpy&order_type=sell&amount=1", 
        client.base_url, 
        &currency
    );

    let buy_rate = reqwest::get(&buy_endpoint).await?.json::<FetchRate>().await?.to_f64()?;
    let sell_rate = reqwest::get(&sell_endpoint).await?.json::<FetchRate>().await?.to_f64()?;
    let spread_ratio = ((buy_rate - sell_rate) / sell_rate) * 100.0;

    let rate = Rate {
        currency: currency.to_string(),
        buy_rate,
        sell_rate,
        spread_ratio,
    };

    client::sleep()?;
    Ok(rate)
}
