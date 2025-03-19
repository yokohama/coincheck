use chrono::NaiveDateTime;
use coincheck::db::establish_connection;
use dotenvy::dotenv;
use csv::ReaderBuilder;
use std::error::Error;
use std::fs::File;

use serde::Deserialize;

use coincheck::models::transaction::{Transaction, NewTransaction};

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct CsvTransaction {
    id: String,
    time: String,
    operation: String,
    amount: String,
    trading_currency: String,
    price: Option<String>,
    original_currency: Option<String>,
    fee: Option<String>,
    comment: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let pool = establish_connection();
    let mut conn = pool.get().expect("Failed to get DB connection");

    let file_path = "./transactions.csv";
    let file = File::open(&file_path)?;
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);

    let mut order_id = 1;
    for result in rdr.deserialize::<CsvTransaction>() {
        let record: CsvTransaction = result?;

        let operation: String;
        match record.operation.as_str() {
            "Buy" => { operation = "buy".to_string() },
            "Sell" => { operation = "sell".to_string() },
            _ => continue
        };

        let datetime_str_trimmed = &record.time[..19];
        let naive_datetime = NaiveDateTime::parse_from_str(
            datetime_str_trimmed, 
            "%Y-%m-%d %H:%M:%S"
        ).expect("Failed to parse datetime");

        let pair = format!(
            "{}_{}", 
            record.trading_currency,
            record.original_currency.as_deref().unwrap_or("N/A")
        ).to_lowercase();

        let amount = record.amount.parse::<f64>().unwrap().abs();
        let price = record.price.unwrap().parse::<f64>().unwrap().abs();
        let rate = price /amount;

        let new_transaction = NewTransaction {
            order_id,
            created_at: naive_datetime,
            rate,
            amount,
            order_type: operation,
            pair,
            price,
            fee: 0.0,
            fee_currency: "".to_string(),
        };

        let _ = Transaction::create(&mut conn, new_transaction);

        order_id = order_id + 1;
    }

    Ok(())
}
