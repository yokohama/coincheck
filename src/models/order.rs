use chrono::NaiveDateTime;

use diesel::prelude::*;
use serde::{Serialize, Deserialize};

use crate::error::AppError;
use crate::schema::orders;
use crate::schema::orders::dsl::*;

#[derive(Debug, Queryable, Serialize, Deserialize)]
#[diesel(table_name = orders)]
pub struct Order {
    pub id: i32,
    pub rate: f64,
    pub crypto_amount: f64,
    pub jpy_amount: f64,
    pub order_type: String,
    pub pari: f64,
    pub buy_rate: Option<f64>,
    pub sell_rate: Option<f64>,
    pub spread_ratio: Option<f64>,
    pub spread_threshold: Option<f64>,
    pub comment: Option<String>,
    pub api_call_success_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

impl Order {
    #[allow(dead_code)]
    pub fn create(
        conn: &mut PgConnection, 
        new_order: &NewOrder
     ) -> Result<(), AppError> {

        diesel::insert_into(orders)
            .values(new_order)
            .execute(conn)?;

        Ok(())
    }
}

#[derive(Debug, Insertable, Serialize, Deserialize, Clone)]
#[diesel(table_name = orders)]
pub struct NewOrder {
    pub rate: Option<f64>,
    pub pair: String,
    pub order_type: String,
    pub crypto_amount: f64,
    pub jpy_amount: f64,
    pub buy_rate: Option<f64>,
    pub sell_rate: Option<f64>,
    pub spread_ratio: Option<f64>,
    pub spread_threshold: Option<f64>,
    pub comment: Option<String>,
    pub api_call_success_at: Option<NaiveDateTime>,
}

impl NewOrder {
    pub fn new(pair_str: String) -> Self {
        Self {
            rate: Some(0.0),
            buy_rate: Some(0.0),
            sell_rate: Some(0.0),
            pair: pair_str,
            order_type: "".to_string(),
            jpy_amount: 0.0,
            crypto_amount: 0.0,
            spread_ratio: Some(0.0),
            spread_threshold: Some(0.0),
            comment: None,
            api_call_success_at: None,
        }
    }
}
