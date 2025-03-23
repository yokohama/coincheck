use diesel::prelude::*;

use crate::{
    repositories,
    models,
    api,
};
use crate::error::AppError;

pub struct Report {
    pub summary: models::summary::NewSummary,
    pub summary_records: Vec<models::summary_record::NewSummaryRecord>,
}

pub fn create(
    conn: &mut PgConnection, 
    new_summary: &models::summary::NewSummary,
    new_summary_records: &mut Vec<models::summary_record::NewSummaryRecord>,
) -> Result<(), AppError> {

    models::summary::Summary::create(conn, new_summary, new_summary_records)
}

pub async fn make_report(
    conn: &mut PgConnection,
    client: &api::coincheck::client::CoincheckClient,
) -> Result<Report, AppError> {

    let my_balancies = repositories::balance::my_balancies(&client).await?;
    let my_trading_currencies = repositories::balance::my_trading_currencies(&client).await?;

    let mut new_summary_records: Vec<models::summary_record::NewSummaryRecord> = Vec::new();
    let mut total_jpy_value: f64 = my_balancies.get("jpy").unwrap().as_f64().unwrap_or(0.0);

    if let Some(balances) = my_balancies.as_object() {
        for currency in my_trading_currencies.iter() {
            if let Some(amount) = balances.get(currency).and_then(|v| v.as_f64()) {

                let rate = api::coincheck::rate::find(&client, &currency).await?;
                let jpy_value = amount * rate.sell_rate;

                new_summary_records.push(models::summary_record::NewSummaryRecord {
                    summary_id: None,
                    currency: currency.to_string(),
                    amount,
                    rate: rate.sell_rate,
                    jpy_value,
                });

                total_jpy_value += jpy_value;
            }
        }
    } else {
        println!("JSON is not an object");
    }

    let jpy_balance = repositories::balance::get_jpy_balance(&my_balancies)?;
    new_summary_records.push(models::summary_record::NewSummaryRecord {
        summary_id: None,
        currency: "jpy".to_string(),
        amount: jpy_balance,
        rate: 0.0,
        jpy_value: jpy_balance,
    });

    let total_invested = repositories::transaction::total_invested(conn)?;
    let pl = total_jpy_value - total_invested;

    let new_summary = models::summary::NewSummary {
        total_invested,
        total_jpy_value,
        pl,
    };

    Ok(Report {
        summary: new_summary,
        summary_records: new_summary_records,
    })
}
