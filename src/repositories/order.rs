use diesel::prelude::*;

use crate::error::AppError;
use crate::api::coincheck::{
    self,
    client::CoincheckClient,
};
use crate::models::order::{NewOrder, Order};

pub async fn post_market_order(
    conn: &mut PgConnection, 
    client: &CoincheckClient,
    new_order: NewOrder
) -> Result<(), AppError> {

    coincheck::order::post_market_order(
        client, 
        new_order.pair.as_str(), 
        new_order.order_type.as_str(),
        new_order.amount
    ).await?;

    Order::create(conn, new_order)?;

    Ok(())
}
