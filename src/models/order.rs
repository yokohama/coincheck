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
    pub amount: f64,
    pub order_type: String,
    pub pari: f64,
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
    pub amount: f64,
}
