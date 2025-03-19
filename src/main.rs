mod db;
mod models;
mod repositories;
mod api;
mod schema;

use std::error::Error;
use dotenvy::dotenv;

use tokio;
use db::establish_connection;
use diesel::pg::PgConnection;

use api::coincheck;
use api::coincheck::client::CoincheckClient;

const CURRENCIES: &[&str] = &["btc", "eth", "shib", "avax"];

#[derive(Debug)]
struct CurrencyTotal {
    pub invested: f64,
    pub jpy_value: f64,
    pub pl: f64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().expect(".env file not found");

    let pool = establish_connection();
    let mut conn = pool.get().expect("Failed to get DB connection");
    let client = CoincheckClient::new()?;

    // 保有量を取得
    print_balances(&client).await?;

    // 資産サマリーを表示
    print_summary(&mut conn, &client).await?;

    // tickerを更新
    for currency in CURRENCIES.iter() {
        create_ticker(&mut conn, &client, currency).await?;
    };

    // 売買判断
    for currency in CURRENCIES.iter() {
        let signal = repositories::ticker::determine_trade_signal(&mut conn, currency).unwrap();
        println!("{}: {:?}", currency, signal);
    };
    println!("");

    Ok(())
}

async fn currency_summary(
    conn: &mut PgConnection, 
    client: &CoincheckClient, 
    currency: &str
) -> Result<CurrencyTotal, Box<dyn Error>> {

    let transaction_summary = repositories::transaction::summary(conn, currency)?;
    let rate = coincheck::rate::find(&client, currency).await?;
    let jpy_value = transaction_summary.total_amount * rate.sell_rate;
    let pl = jpy_value - transaction_summary.total_invested;

    let currency_total = CurrencyTotal {
        invested: transaction_summary.total_invested,
        jpy_value,
        pl,
    };

    println!("#-- {} ", &currency.to_uppercase());
    println!("Buy rate: {}", rate.buy_rate);
    println!("Sell rate: {}", rate.sell_rate);
    println!("Spread ratio: {}", rate.spread_ratio);
    println!("投資額: {}", transaction_summary.total_invested);
    println!("資産: {}", jpy_value);
    println!("損益: {}", pl);
    println!("");

    Ok(currency_total)
}

async fn print_balances(client: &CoincheckClient) -> Result<(), Box<dyn Error>> {

    println!("#-- 通貨保有量 ");

    let mut plus_jpy_currencies = Vec::from(CURRENCIES);
    plus_jpy_currencies.push("jpy");

    let balance = coincheck::balance::find(&client).await?;
    for currency in plus_jpy_currencies {
        if let Some(c) = balance.get(currency).and_then(|v| v.as_str()) {
            println!("{}: {}", currency, c);
        }
    }
    println!("");

    Ok(())
}

async fn print_summary(
    conn: &mut PgConnection, 
    client: &CoincheckClient
) -> Result<(), Box<dyn Error>> {

    // 資産サマリーを表示
    let mut total_invested: f64 = 0.0;
    let mut total_jpy_value: f64 = 0.0;
    let mut total_pl: f64 = 0.0;
    for currency in CURRENCIES.iter() {
        let currency_total = currency_summary(conn, &client, currency).await?;
        total_invested += currency_total.invested;
        total_jpy_value += currency_total.jpy_value;
        total_pl += currency_total.pl;
    };

    println!("#-- Total ");
    println!("投資額: {}", total_invested);
    println!("総資産: {}", total_jpy_value);
    println!("損益: {}", total_pl);
    println!("");

    Ok(())
}

async fn create_ticker(
    conn: &mut PgConnection,
    client: &CoincheckClient,
    currency: &str,
) -> Result<(), Box<dyn Error>> {

    let mut new_ticker = coincheck::ticker::find(&client, &currency).await?;
    new_ticker.pair = Some(currency.to_string());
    models::ticker::Ticker::create(conn, new_ticker)?;

    Ok(())
}
