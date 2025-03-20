use std::error::Error;

use diesel::prelude::*;

use crate::models;

pub fn create(
    conn: &mut PgConnection, 
    new_summary: models::summary::NewSummary,
    new_summary_records: Vec<models::summary_record::NewSummaryRecord>,
) -> Result<(), Box<dyn Error>> {

    models::summary::Summary::create(conn, new_summary, new_summary_records)
}
