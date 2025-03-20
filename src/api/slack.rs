use std::error::Error;
use std::env;
use dotenvy::dotenv;

use reqwest::Client;
use serde_json::json;

pub async fn send_orderd_information(
    currency: &str, 
    ops: &str,  // Buy or Sell
    amount: f64
) -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let url = env::var("SLACK_INCOMMING_WEBHOOK_URL")?;
    let client = Client::new();

    let payload = json!({
	    "blocks": [
	    	{
	    		"type": "section",
	    		"text": {
	    			"type": "mrkdwn",
	    			"text": "*注文を実行しました！*"
	    		}
	    	},
	    	{
	    		"type": "section",
	    		"fields": [
	    			{
	    				"type": "mrkdwn",
	    				"text": format!("*通貨:* {}", currency)
	    			},
	    			{
	    				"type": "mrkdwn",
	    				"text": format!("*オペレーション:* {}", ops)
	    			},
	    			{
	    				"type": "mrkdwn",
	    				"text": format!("*Amount:* {}", amount)
	    			},
	    		]
	    	}
	    ]
    });

    client.post(url).json(&payload).send().await?;

    Ok(())
}
