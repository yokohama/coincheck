use thiserror::Error;

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

    #[error("Invalid data: {0}")]
    InvalidData(String),
}
