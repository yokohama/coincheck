use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use chrono::NaiveDateTime;

use crate::error::AppError;
use crate::schema::summaries;
use crate::schema::summaries::dsl::*;
use crate::schema::summary_records;
use super::summary_record::NewSummaryRecord;

#[derive(Debug, Queryable, Serialize, Deserialize)]
#[diesel(table_name = summaries)]
pub struct Summary {
    pub id: i32,
    pub total_invested: f64,
    pub total_jpy_value: f64,
    pub pl: f64,
    pub created_at: NaiveDateTime,
}

impl Summary {
    #[allow(dead_code)]
    pub fn create(
        conn: &mut PgConnection, 
        new_summary: NewSummary,
        mut new_summary_records: Vec<NewSummaryRecord>,
    ) -> Result<(), AppError> {

        let inserted: Summary = diesel::insert_into(summaries)
            .values(&new_summary)
            .get_result(conn)?;

        for record in new_summary_records.iter_mut() {
            record.summary_id = Some(inserted.id);
        }

        diesel::insert_into(summary_records::table)
            .values(&new_summary_records)
            .execute(conn)?;

        Ok(())
    }
}

#[derive(Debug, Insertable, Serialize, Deserialize)]
#[diesel(table_name = summaries)]
pub struct NewSummary {
    pub total_invested: f64,
    pub total_jpy_value: f64,
    pub pl: f64,
}
