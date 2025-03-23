use diesel::prelude::*;

use crate::error::AppError;
use crate::models;

#[allow(dead_code)]
pub fn create(
    conn: &mut PgConnection, 
    new_ticker: models::ticker::NewTicker
) -> Result<(), AppError> {
    models::ticker::Ticker::create(conn, new_ticker)?;

    Ok(())
}
