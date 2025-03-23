use std::env;
use dotenvy::dotenv;

use log::info;

use reqwest::Client;
use serde_json::json;

use crate::error::AppError;
use crate::models::summary::NewSummary;
use crate::models::order::NewOrder;

pub async fn send_orderd_information(new_order: &NewOrder) -> Result<(), AppError> {
    dotenv().ok();

    let url = env::var("SLACK_INCOMMING_WEBHOOK_URL")?;
    let client = Client::new();

    let payload = json!({
        "blocks": [
            {
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": ":coin: *注文を実行しました！*"
                }
            },
            {
                "type": "section",
                "fields": [
                     {
                       "type": "mrkdwn",
                       "text": format!("*通貨:* {}", new_order.pair)
                     },
                     {
                         "type": "mrkdwn",
                         "text": format!("*オペレーション:* {}", new_order.order_type)
                     },
                     {
                         "type": "mrkdwn",
                         "text": format!("*Amount:* {}", new_order.amount)
                     },
                ]
            }
        ]
    });

    let res = client.post(url).json(&payload).send().await?;
    info!("Slack: send_order_information: status={}, body={}", res.status(), res.text().await?);

    Ok(())
}

pub async fn send_summary(title: &str, new_summary: &NewSummary) -> Result<(), AppError> {
    dotenv().ok();

    let url = env::var("SLACK_INCOMMING_WEBHOOK_URL")?;
    let client = Client::new();

    let total_jpy_value = new_summary.total_jpy_value.round() as i32;
    let pl = new_summary.pl.round() as i32;

    let payload = json!({
	    "blocks": [
	    	{
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": format!(
                        ":moneybag: *{}*\n\n *Total invested:* {}円\n *Total JPY value:* {}円\n *P/L:* {}円",
                        title,
                        new_summary.total_invested,
                        total_jpy_value,
                        pl,
                    )
                }
	    	}
	    ]
    });

    let res = client.post(&url).json(&payload).send().await?.error_for_status()?;
    info!("Slack send_summary: status={}, body={}", res.status(), res.text().await?);

    Ok(())
}
