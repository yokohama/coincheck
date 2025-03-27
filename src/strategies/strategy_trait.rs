use async_trait::async_trait;
use diesel::prelude::*;

use crate::error::AppError;
use crate::strategies::trade_signal::TradeSignal;

#[async_trait]
pub trait Strategy {
    async fn determine_trade_signal(
        &self,
        conn: &mut PgConnection,
        currency: &str,
        current_bid: f64,
        current_ask: f64,
        crypto_balance: f64,
    ) -> Result<TradeSignal, AppError>;
}
