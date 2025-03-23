use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use chrono::NaiveDateTime;

use crate::error::AppError;
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
    pub fn create(conn: &mut PgConnection, new_ticker: NewTicker) -> Result<(), AppError> {
        diesel::insert_into(tickers)
            .values(&new_ticker)
            .execute(conn)?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn delete_oldest(conn: &mut PgConnection) -> Result<usize, AppError> {
        let retention_threshold = 1000;
        let purge_ratio = 0.10;

        let total: i64 = tickers
            .select(diesel::dsl::count_star())
            .first(conn)?;

        if total >= retention_threshold {
            let to_delete = (total as f64 * purge_ratio).ceil() as i64;

            let ids_to_delete = tickers
                .select(id)
                .order(timestamp.asc())
                .limit(to_delete)
                .load::<i32>(conn)?;

            if !ids_to_delete.is_empty() {
                let deleted = diesel::delete(tickers.filter(id.eq_any(ids_to_delete)))
                    .execute(conn)?;
                Ok(deleted)
            } else {
                Ok(0)
            }
        } else {
            Ok(0)
        }
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
