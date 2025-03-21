use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use chrono::NaiveDateTime;

use crate::error::AppError;
use crate::schema::transactions;
use crate::schema::transactions::dsl::*;
use crate::models::util::{
    serialize_naive_datetime, 
    deserialize_naive_datetime
};

#[derive(Debug, Queryable, Serialize, Deserialize)]
#[diesel(table_name = transactions)]
pub struct Transaction {
    pub id: i32,
    pub order_id: i32,
   #[serde(serialize_with = "serialize_naive_datetime", 
       deserialize_with = "deserialize_naive_datetime")]
    pub created_at: NaiveDateTime,
    pub rate: f64,
    pub amount: f64,
    pub order_type: String,
    pub pair: String,
    pub price: f64,
    pub fee_currency: String,
    pub fee: f64,
}

impl Transaction {
    #[allow(dead_code)]
    pub fn create(conn: &mut PgConnection, new_order: NewTransaction) -> Result<(), AppError> {
        diesel::insert_into(transactions)
            .values(&new_order)
            .execute(conn)?;

        Ok(())
    }
}

#[derive(Debug, Insertable, Serialize, Deserialize)]
#[diesel(table_name = transactions)]
pub struct NewTransaction {
    pub order_id: i32,
   #[serde(serialize_with = "serialize_naive_datetime", 
       deserialize_with = "deserialize_naive_datetime")]
    pub created_at: NaiveDateTime,
    pub rate: f64,
    pub amount: f64,
    pub order_type: String,
    pub pair: String,
    pub price: f64,
    pub fee_currency: String,
    pub fee: f64,
}
