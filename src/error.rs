use thiserror::Error;
use plotters::prelude::DrawingAreaErrorKind;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] diesel::result::Error),

    #[error("API request failed: {0}")]
    ApiError(#[from] reqwest::Error),

    #[error("Environment variable missing: {0}")]
    EnvVarError(#[from] std::env::VarError),

    #[error("Database connection pool error: {0}")]
    PoolError(#[from] diesel::r2d2::PoolError),

    #[error("serde_json error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),

    #[error("reqwet header to str error: {0}")]
    ToStrError(#[from] reqwest::header::ToStrError),

    #[error("Invalid data: {0}")]
    InvalidData(String),
}
