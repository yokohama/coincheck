use chrono::Utc;

use reqwest::Client;
use serde::Serialize;
use serde_json::Value;
use log::{info, error};

use crate::error::AppError;
use crate::api::coincheck::{
    client,
    private,
};
use crate::models::order::NewOrder;

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
    new_order: &mut NewOrder,
    amount: f64,
) -> Result<NewOrder, AppError> {
    let pair = format!("{}_jpy", new_order.pair);

    let order = match new_order.order_type.as_str() {
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

    let comment = format!("{}, [{}]: {}", new_order.comment.take().unwrap(), status, body);
    new_order.comment = Some(comment);

    if status.is_success() {
        info!("Status {}: {}", status, body);
        new_order.api_call_success_at = Some(Utc::now().naive_utc());
    } else {
        error!("Status {}: {}", status, body);
    }

    client::sleep()?;
    Ok(new_order.clone())
}
