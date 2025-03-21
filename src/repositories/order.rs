use diesel::prelude::*;

use crate::error::AppError;
use crate::api::{coincheck, slack};
use crate::models::order::{NewOrder, Order};

pub async fn post_market_order(
    conn: &mut PgConnection, 
    client: &coincheck::client::CoincheckClient,
    new_order: NewOrder
) -> Result<(), AppError> {

    let status = coincheck::order::post_market_order(
        client, 
        new_order.pair.as_str(), 
        new_order.order_type.as_str(),
        new_order.amount
    ).await?;

    if status.is_success() {
        Order::create(conn, &new_order)?;
        slack::send_orderd_information(&new_order).await?;
    }

    Ok(())
}
