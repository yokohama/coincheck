use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{Text, Integer, BigInt, Numeric};
use bigdecimal::{ToPrimitive, BigDecimal};
use serde::{Serialize, Deserialize};

use crate::error::AppError;
use crate::schema::optimized_mas;

#[derive(Debug, QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct CrossoverStats {
    #[diesel(sql_type = BigInt)]
    total_crosses: i64,
    #[diesel(sql_type = BigInt)]
    wins_count: i64,
    #[diesel(sql_type = Numeric)]
    win_rate_pct: BigDecimal,
}

#[derive(Debug, Queryable, Serialize, Deserialize)]
#[diesel(table_name = optimized_mas)]
pub struct OptimizedMa {
    pub id: i32,
    pub pair: String,
    pub short_ma: i32,
    pub long_ma: i32,
    pub offset_minutes: i32,
    pub win_rate_pct: Option<f64>,
    pub total: Option<i32>,
    pub wins: Option<i32>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable, Serialize, Deserialize, Clone)]
#[diesel(table_name = optimized_mas)]
pub struct NewOptimizedMa {
    pub pair: String,
    pub short_ma: i32,
    pub long_ma: i32,
    pub offset_minutes: i32,
    pub win_rate_pct: f64,
    pub total: i32,
    pub wins: i32,
}

impl OptimizedMa {
    pub fn find_best_for_ma(
        conn: &mut PgConnection,
        pair_str: &str,
    ) -> Result<Option<(i32, i32)>, AppError> {
        use crate::schema::optimized_mas::dsl::{
            optimized_mas, 
            pair, 
            win_rate_pct,
            short_ma, long_ma
        };

        let result = optimized_mas
            .filter(pair.eq(pair_str))
            .filter(win_rate_pct.ge(50.0))
            .order(win_rate_pct.desc())
            .first::<OptimizedMa>(conn)
            .optional()?;

        if let Some(record) = result {
            Ok(Some((record.short_ma, record.long_ma)))
        } else {
            Ok(None)
        }
    }

    #[allow(dead_code)]
    pub fn create(
        conn: &mut PgConnection, 
        pair_str: &str,
        offset: i32,
    ) -> Result<(), AppError> {
        for short in 5..=10 {
            for long in (short + 5)..=30 {
                let result = sql_query(
                    r#"
                    WITH base AS (
                      SELECT
                        id,
                        pair,
                        timestamp,
                        last,
                        ROW_NUMBER() OVER (ORDER BY timestamp) AS rn
                      FROM tickers
                      WHERE pair = $1
                    ),
                    sma_calc AS (
                      SELECT
                        b1.timestamp,
                        b1.last,
                        b1.rn,
                        (SELECT AVG(b2.last)
                         FROM base b2
                         WHERE b2.rn BETWEEN b1.rn - ($2 - 1) AND b1.rn) AS sma_short,
                        (SELECT AVG(b3.last)
                         FROM base b3
                         WHERE b3.rn BETWEEN b1.rn - ($3 - 1) AND b1.rn) AS sma_long
                      FROM base b1
                      WHERE b1.rn >= $3
                    ),
                    sma_with_diff AS (
                      SELECT *,
                        sma_short - sma_long AS diff,
                        LAG(sma_short - sma_long) OVER (ORDER BY timestamp) AS prev_diff
                      FROM sma_calc
                    ),
                    cross_events AS (
                      SELECT timestamp AS cross_time, rn AS cross_rn, last AS cross_last, 'GC' AS cross_type
                      FROM sma_with_diff
                      WHERE prev_diff < 0 AND diff >= 0
                      UNION ALL
                      SELECT timestamp, rn, last, 'DC'
                      FROM sma_with_diff
                      WHERE prev_diff > 0 AND diff <= 0
                    ),
                    cross_with_result AS (
                      SELECT
                        g.cross_time,
                        g.cross_type,
                        g.cross_last,
                        b2.timestamp AS after_time,
                        b2.last AS after_last,
                        CASE
                          WHEN g.cross_type = 'GC' AND b2.last > g.cross_last THEN true
                          WHEN g.cross_type = 'DC' AND b2.last < g.cross_last THEN true
                          ELSE false
                        END AS is_win
                      FROM cross_events g
                      JOIN base b2 
                        ON b2.timestamp = (
                          SELECT MIN(b3.timestamp)
                          FROM base b3
                          WHERE b3.timestamp >= g.cross_time + ($4 || ' minutes')::interval
                        )
                    )
                    SELECT
                      COUNT(*) AS total_crosses,
                      COUNT(*) FILTER (WHERE is_win) AS wins_count,
                      ROUND(COUNT(*) FILTER (WHERE is_win) * 100.0 / COUNT(*), 2) AS win_rate_pct
                    FROM cross_with_result
                    "#
                )
                .bind::<Text, _>(pair_str)
                .bind::<Integer, _>(short)
                .bind::<Integer, _>(long)
                .bind::<Integer, _>(offset)
                .get_result::<CrossoverStats>(conn)
                .optional()?;

                if let Some(r) = result {
                    if r.total_crosses == 0 {
                        continue;
                    }

                    let new_optimized_ma = NewOptimizedMa {
                        pair: pair_str.to_string(),
                        short_ma: short,
                        long_ma: long,
                        offset_minutes: offset,
                        win_rate_pct: r.win_rate_pct.to_f64().unwrap_or(0.0),
                        total: r.total_crosses as i32,
                        wins: r.wins_count as i32,
                    };

                    diesel::insert_into(optimized_mas::table)
                        .values(new_optimized_ma)
                        .execute(conn)?;
                }
            }
        }

        Ok(())
    }
}
