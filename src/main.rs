mod repositories;
mod api;
mod schema;
mod models;
mod error;

use dotenvy::dotenv;

use tokio;

use error::AppError;
use api::coincheck::client::CoincheckClient;
use repositories::balance;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    dotenv().expect(".env file not found");

    let client = CoincheckClient::new()?;

    // 所持している全部の通貨を取得
    let my_currencies = balance::my_currencies(&client).await?;
    println!("{:#?}", my_currencies);

    // 所持している通貨の中からトレーディング対象の仮想通貨を取得
    let my_trading_currencies = balance::my_trading_currencies(&client).await?;
    println!("{:#?}", my_trading_currencies);

    // 保有量を取得
    print_my_balances(&client).await?;

    Ok(())
}

async fn print_my_balances(client: &CoincheckClient) -> Result<(), AppError> {
    println!("#-- 通貨保有量 ");
    let my_balances = balance::my_balancies(&client).await?;
    println!("{:#?}", my_balances);
    println!("");

    Ok(())
}
