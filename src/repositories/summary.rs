use diesel::prelude::*;

use crate::models;
use crate::error::AppError;

pub fn create(
    conn: &mut PgConnection, 
    new_summary: models::summary::NewSummary,
    new_summary_records: Vec<models::summary_record::NewSummaryRecord>,
) -> Result<(), AppError> {

    models::summary::Summary::create(conn, new_summary, new_summary_records)
}
