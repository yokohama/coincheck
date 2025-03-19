use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

use hmac::{Hmac, Mac};
use sha2::Sha256;
use hex;
use reqwest::header::{HeaderMap, HeaderValue};

use crate::api::coincheck::client::CoincheckClient;

type HmacSha256 = Hmac<Sha256>;

pub fn headers(
    url: &str,
    client: &CoincheckClient,
) -> Result<HeaderMap, Box<dyn Error>> {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs()
        .to_string();

    let message = format!("{}{}", nonce, url);

    let mut mac = HmacSha256::new_from_slice(client.secret_key.as_bytes())?;
    mac.update(message.as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());

    // ヘッダーの作成
    let mut headers = HeaderMap::new();
    headers.insert("ACCESS-KEY", HeaderValue::from_str(&client.access_key)?);
    headers.insert("ACCESS-NONCE", HeaderValue::from_str(&nonce)?);
    headers.insert("ACCESS-SIGNATURE", HeaderValue::from_str(&signature)?);

    Ok(headers)
}
