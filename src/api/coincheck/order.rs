use reqwest::Client;
use serde::Serialize;

use log::info;

use crate::error::AppError;
use crate::api::coincheck::{
    client::CoincheckClient,
    private,
};

#[derive(Debug, Serialize)]
#[serde(tag = "order_type")]
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
    coincheck_client: &CoincheckClient,
    currency: &str,
    order_type: &str,
    amount: f64,
) -> Result<(), AppError> {
    let pair = format!("{}_jpy", currency);

    let order = match order_type {
        "buy" => MarketOrderRequest::Buy {
            pair: pair.clone(),
            market_buy_amount: amount,
        },
        "sell" => MarketOrderRequest::Sell {
            pair: pair.clone(),
            amount,
        },
        _ => return Err(AppError::InvalidData("Invalid order_type".to_string())),
    };

    let endpoint = format!("{}/api/exchange/orders", coincheck_client.base_url);
    let headers = private::headers(&endpoint, &coincheck_client)?;

    Client::new()
        .post(&endpoint)
        .headers(headers)
        .json(&order)
        .send()
        .await?
        .error_for_status()?;

    info!("{:#?}", order);

    Ok(())
}
