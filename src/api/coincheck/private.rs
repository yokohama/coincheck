use std::time::{SystemTime, UNIX_EPOCH};

use hmac::{Hmac, Mac};
use sha2::Sha256;
use hex;
use reqwest::header::{HeaderMap, HeaderValue};

use crate::api::coincheck::client::CoincheckClient;
use crate::error::AppError;

type HmacSha256 = Hmac<Sha256>;

pub fn headers(
    url: &str,
    client: &CoincheckClient,
    body: Option<&String>,
) -> Result<HeaderMap, AppError> {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| AppError::InvalidData(format!("Time error: {}", e)))?
        .as_nanos()
        .to_string();

    let message = match body {
        Some(json_body) => format!("{}{}{}", nonce, url, json_body),
        None => format!("{}{}", nonce, url),
    };

    let mut mac = HmacSha256::new_from_slice(client.secret_key.as_bytes())
        .map_err(|e| AppError::InvalidData(format!("Hmac error: {}", e)))?;
    mac.update(message.as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());

    // ヘッダーの作成
    let mut headers = HeaderMap::new();
    headers.insert("ACCESS-KEY", 
        HeaderValue::from_str(&client.access_key)
            .map_err(|e| AppError::InvalidData(format!("Header error: {}", e)))?
    );
    headers.insert("ACCESS-NONCE", 
        HeaderValue::from_str(&nonce)
            .map_err(|e| AppError::InvalidData(format!("Header error: {}", e)))?
    );
    headers.insert("ACCESS-SIGNATURE", 
        HeaderValue::from_str(&signature)
            .map_err(|e| AppError::InvalidData(format!("Header error: {}", e)))?
    );

    Ok(headers)
}
