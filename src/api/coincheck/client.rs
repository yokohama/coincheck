use std::env;
use reqwest::Client;

use crate::error::AppError;

#[derive(Debug)]
#[allow(dead_code)]
pub struct CoincheckClient {
    pub base_url: String,
    pub access_key: String,
    pub secret_key: String,
    pub client: Client,
}

impl CoincheckClient {
    pub fn new() -> Result<Self, AppError> {
        let base_url = env::var("COINCHECK_BASE_URL")?;
        let access_key = env::var("COINCHECK_ACCESS_KEY")?;
        let secret_key = env::var("COINCHECK_SECRET_ACCESS_KEY")?;

        Ok(Self {
            base_url,
            access_key,
            secret_key,
            client: Client::new(),
        })
    }
}
