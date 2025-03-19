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

pub fn summary(conn: &mut PgConnection, currency: &str) -> Result<Summary, Box<dyn Error>> {
    let pair_str = format!("{}_jpy", currency);

    let buy = transactions
        .filter(order_type.eq("buy"))
        .filter(pair.eq(&pair_str))
        .select((sum(price), sum(amount)))
        .first::<(Option<f64>, Option<f64>)>(conn)?;

    let sell = transactions
        .filter(order_type.eq("sell"))
        .filter(pair.eq(&pair_str))
        .select((sum(price), sum(amount)))
        .first::<(Option<f64>, Option<f64>)>(conn)?;

    let buy_amount = buy.1.unwrap_or(0.0);
    let sell_amount = sell.1.unwrap_or(0.0);

    Ok(Summary {
        currency: currency.to_string(),
        total_invested: buy.0.unwrap_or(0.0),
        total_amount: buy_amount - sell_amount,
    })
}
