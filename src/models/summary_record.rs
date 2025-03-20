use std::error::Error;

use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use chrono::NaiveDateTime;

use crate::schema::summary_records;
use crate::schema::summary_records::dsl::*;

#[derive(Debug, Queryable, Serialize, Deserialize)]
#[diesel(table_name = summary_records)]
pub struct SummaryRecord {
    pub id: i32,
    pub summary_id: i32,
    pub currency: String,
    pub amount: f64,
    pub rate: f64,
    pub jpy_value: f64,
    pub created_at: NaiveDateTime,
}

impl SummaryRecord {

    #[allow(dead_code)]
    pub fn create(
        conn: &mut PgConnection, 
        new_summary_record: NewSummaryRecord
    ) -> Result<(), Box<dyn Error>> {
        diesel::insert_into(summary_records)
            .values(&new_summary_record)
            .execute(conn)?;

        Ok(())
    }
}

#[derive(Debug, Insertable, Serialize, Deserialize)]
#[diesel(table_name = summary_records)]
pub struct NewSummaryRecord {
    pub summary_id: Option<i32>,
    pub currency: String,
    pub amount: f64,
    pub rate: f64,
    pub jpy_value: f64,
}
