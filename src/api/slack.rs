use std::env;
use dotenvy::dotenv;

use log::info;

use reqwest::Client;
use serde_json::json;

use crate::error::AppError;
use crate::models::summary::NewSummary;
use crate::models::order::NewOrder;
use crate::models::summary_record::NewSummaryRecord;

pub async fn send_orderd_information(new_order: &NewOrder) -> Result<(), AppError> {
    dotenv().ok();

    let mut operation_type = "";
    if new_order.order_type == "buy" {
        operation_type = "購入";
    } else {
        operation_type = "売却";
    }

    let currency = new_order.pair.to_uppercase();

    let text = format!(
        ":coin: *[{}]* を *{}* 、{}しました！", 
        currency,
        new_order.amount,
        operation_type
    );

    let url = env::var("SLACK_INCOMMING_WEBHOOK_URL")?;
    let client = Client::new();

    let payload = json!({
        "blocks": [
            {
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": text
                }
            }
        ]
    });

    let res = client.post(url).json(&payload).send().await?;
    info!("Slack: send_order_information: status={}, body={}", res.status(), res.text().await?);

    Ok(())
}

pub async fn send_summary(
    title: &str, 
    new_summary: &NewSummary,
    new_summary_records: Vec<NewSummaryRecord>
) -> Result<(), AppError> {
    dotenv().ok();

    let url = env::var("SLACK_INCOMMING_WEBHOOK_URL")?;
    let client = Client::new();

    let total_jpy_value = new_summary.total_jpy_value.round() as i32;
    let pl = new_summary.pl.round() as i32;

    let fields = make_currency_fields(new_summary_records);
    let payload = json!({
	    "blocks": [
	    	{
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": format!(
                        ":moneybag: *{}*\n *Total invested:* {}円\n *Total JPY value:* {}円\n *P/L:* {}円",
                        title,
                        new_summary.total_invested,
                        total_jpy_value,
                        pl,
                    )
                }
	    	},
            {
                "type": "section",
                "fields": fields
            }
	    ]
    });

    println!("{:#?}", payload);

    let res = client.post(&url).json(&payload).send().await?.error_for_status()?;
    info!("Slack send_summary: status={}, body={}", res.status(), res.text().await?);

    Ok(())
}

pub fn make_currency_fields(new_summary_records: Vec<NewSummaryRecord>) -> serde_json::Value {
    let mut fields: Vec<serde_json::Value> = Vec::new();
    for record in new_summary_records {
        fields.push(json!({
            "type": "mrkdwn",
            "text": format!("*{}*\n{}", record.currency.to_uppercase(), record.amount)
        }));
    };

    json!(fields)
}
