use reqwest::Client;
use serde::Serialize;
use serde_json::Value;
use log::{info, error};

use crate::error::AppError;
use crate::api::coincheck::{
    client,
    private,
};

#[derive(Debug, Serialize)]
#[serde(tag = "order_type")]
#[allow(dead_code)]
pub enum MarketOrderRequest {
    #[serde(rename = "market_buy")]
    Buy {  
        pair: String,
        market_buy_amount: f64,
    },

    #[serde(rename = "market_sell")]
    Sell {
        pair: String,
        amount: f64,
    }
}

pub async fn post_market_order(
    coincheck_client: &client::CoincheckClient,
    currency: &str,
    order_type: &str,
    amount: f64,
) -> Result<(reqwest::StatusCode, Value), AppError> {
    let pair = format!("{}_jpy", currency);

    let order = match order_type {
        "market_buy" => MarketOrderRequest::Buy {
            pair: pair.clone(),
            market_buy_amount: amount,
        },
        "market_sell" => MarketOrderRequest::Sell {
            pair: pair.clone(),
            amount,
        },
        _ => return Err(AppError::InvalidData("Invalid order_type".to_string())),
    };

    let json_string = serde_json::to_string(&order)?;

    let endpoint = format!("{}/api/exchange/orders", coincheck_client.base_url);
    let headers = private::headers(&endpoint, &coincheck_client, Some(&json_string))?;

    let res = Client::new()
        .post(&endpoint)
        .headers(headers)
        .header("Content-Type", "application/json")
        .body(json_string)
        .send()
        .await?;

    let status = res.status();
    let body: Value = res.json().await?;
    if status.is_success() {
        info!("Status {}: {}", status, body);
        //info!("{:#?}", order);
    } else {
        error!("Status {}: {}", status, body);
        //error!("{:#?}", order);
    }

    client::sleep()?;
    Ok((status, body))
}
