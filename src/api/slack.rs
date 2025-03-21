use std::env;
use dotenvy::dotenv;

use reqwest::Client;
use serde_json::json;

use crate::error::AppError;
use crate::models::summary::NewSummary;
use crate::models::order::NewOrder;

pub async fn send_orderd_information(new_orders: Vec<NewOrder>) -> Result<(), AppError> {
    dotenv().ok();

    let url = env::var("SLACK_INCOMMING_WEBHOOK_URL")?;
    let client = Client::new();

    let mut fields: Vec<serde_json::Value> = Vec::new();
    for new_order in new_orders {
        fields.push(make_orderd_information_field(new_order));
    }

    let mut blocks = vec![
        json!({
	   		"type": "section",
	   		"text": {
	   			"type": "mrkdwn",
	   			"text": "*注文を実行しました！*"
	   		}
        })
    ];

    client.post(url)
        .json(&json!({ "blocks": blocks.extend(fields) }))
        .send()
        .await?;

    Ok(())
}

fn make_orderd_information_field(new_order: NewOrder) -> serde_json::Value {
    json!({
   		"type": "section",
   		"fields": [
   			{
   				"type": "mrkdwn",
   				"text": format!("*通貨:* {}", new_order.currency)
   			},
   			{
   				"type": "mrkdwn",
   				"text": format!("*オペレーション:* {}", new_order.ops)
   			},
   			{
   				"type": "mrkdwn",
   				"text": format!("*Amount:* {}", new_order.amount)
  			},
        ]
    })
}

pub async fn send_summary(new_summary: NewSummary) -> Result<(), AppError> {
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
                        "*Summary*\n *Total invested:* {}円\n *Total JPY value:* {}円\n *P/L:* {}円",
                        new_summary.total_invested,
                        total_jpy_value,
                        pl,
                    )
                }
	    	}
	    ]
    });

    client.post(&url).json(&payload).send().await?.error_for_status()?;

    Ok(())
}
