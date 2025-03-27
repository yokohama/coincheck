use diesel::prelude::*;

use crate::{
    models,
    error::AppError,
};

#[allow(dead_code)]
pub async fn calc_crossover(
    conn: &mut PgConnection,
) -> Result<(), AppError> {

    let pair_str = "btc";
    let offset = 15;
    models::optimized_ma::OptimizedMa::create(conn, pair_str, offset)?;

    Ok(())
}
