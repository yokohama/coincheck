use diesel::prelude::*;

use crate::error::AppError;
use crate::models::order::{NewOrder, Order};

pub fn create(
    conn: &mut PgConnection, 
    new_order: NewOrder
) -> Result<(), AppError> {

    //TODO: ここでAPIを叩いて注文

    Order::create(conn, new_order)?;

    Ok(())
}
