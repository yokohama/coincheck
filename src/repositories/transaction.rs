use std::error::Error;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::dsl::sum;

use crate::schema::transactions::dsl::*;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Summary {
    pub currency: String,
    pub total_invested: f64, // 投資額
    pub total_amount: f64, // 所持数
}

pub fn total_invested(conn: &mut PgConnection) -> Result<f64, Box<dyn Error>> {
    let invested: Option<f64> = transactions
        .filter(order_type.eq("buy"))
        .select(sum(price))
        .first(conn)?;

    Ok(invested.unwrap_or(0.0))
}
