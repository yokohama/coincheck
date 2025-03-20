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

    // 所持している全部の通貨を取得
    let my_currencies = repositories::balance::my_currencies(&client).await?;
    println!("{:#?}", my_currencies);

    // 所持している通貨の中からpairを取得
    let my_pairs = repositories::balance::my_pairs(&my_currencies);
    println!("{:#?}", my_pairs);

    // 保有量を取得
    print_my_balances(&client).await?;

    // 資産サマリーを表示
    print_summary(&mut conn, &client, my_pairs).await?;

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

async fn print_my_balances(client: &CoincheckClient) -> Result<(), Box<dyn Error>> {

    println!("#-- 通貨保有量 ");
    let my_balances = repositories::balance::my_balances(&client).await?;
    println!("{:#?}", my_balances);
    println!("");

    Ok(())
}

async fn print_summary(
    conn: &mut PgConnection, 
    client: &CoincheckClient,
    my_currencies: Vec<String>
) -> Result<(), Box<dyn Error>> {

    // 資産サマリーを表示
    let mut total_invested: f64 = 0.0;
    let mut total_jpy_value: f64 = 0.0;
    let mut total_pl: f64 = 0.0;
    for currency in my_currencies.iter() {
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
