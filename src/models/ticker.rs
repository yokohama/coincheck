use diesel::{prelude::*, sql_query};
use diesel::sql_types::{Text, Double};
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

#[derive(QueryableByName, Debug)]
#[allow(dead_code)]
struct SpreadStats {
    #[diesel(sql_type = Double)]
    pub median: f64,

    #[diesel(sql_type = Double)]
    pub stddev: f64,

    #[diesel(sql_type = Double)]
    pub suggested_spread_threshold: f64,
}

pub async fn get_dynamic_spread_threshold(conn: &mut PgConnection, currency: &str) -> Result<f64, AppError> {
    let query = r#"
        WITH spread_data AS (
            SELECT 
                ((ask - bid) / bid) * 100.0 AS spread
            FROM tickers
            WHERE pair = $1
        ),
        ordered_spread AS (
            SELECT spread,
                   ROW_NUMBER() OVER (ORDER BY spread) AS rn,
                   COUNT(*) OVER () AS total
            FROM spread_data
        ),
        median_calc AS (
            SELECT AVG(spread) AS median
            FROM ordered_spread
            WHERE rn IN ((total + 1) / 2, (total + 2) / 2)
        ),
        stddev_calc AS (
            SELECT STDDEV(spread) AS stddev FROM spread_data
        )
        SELECT 
            (median_calc.median + stddev_calc.stddev) AS suggested_spread_threshold
        FROM median_calc, stddev_calc;
    "#;

    let result: SpreadStats = sql_query(query)
        .bind::<Text, _>(currency)
        .get_result(conn)?;

    Ok(result.suggested_spread_threshold)
}
