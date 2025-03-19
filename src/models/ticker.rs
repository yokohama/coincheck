use std::error::Error;

use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use chrono::NaiveDateTime;

use crate::schema::tickers;
use crate::schema::tickers::dsl::*;
use crate::models::util::{
    serialize_unix_timestamp,
    deserialize_unix_timestamp,
};

#[derive(Debug, Queryable, Serialize, Deserialize)]
#[diesel(table_name = tickers)]
pub struct Ticker {
    pub id: i32,
    pub pair: String,
    pub last: f64,
    pub bid: f64,
    pub ask: f64,
    pub high: f64,
    pub low: f64,
    pub volume: f64,
    pub timestamp: NaiveDateTime,
}

impl Ticker {
    #[allow(dead_code)]
    pub fn create(conn: &mut PgConnection, new_ticker: NewTicker) -> Result<(), Box<dyn Error>> {
        diesel::insert_into(tickers)
            .values(&new_ticker)
            .execute(conn)?;

        Ok(())
    }
}

#[derive(Debug, Insertable, Serialize, Deserialize)]
#[diesel(table_name = tickers)]
pub struct NewTicker {
    pub pair: Option<String>,
    pub last: f64,
    pub bid: f64,
    pub ask: f64,
    pub high: f64,
    pub low: f64,
    pub volume: f64,

    #[serde(serialize_with = "serialize_unix_timestamp", 
       deserialize_with = "deserialize_unix_timestamp")]
    pub timestamp: NaiveDateTime,
}
