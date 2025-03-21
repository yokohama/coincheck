mod db;
mod models;
mod repositories;
mod api;
mod schema;
mod error;

use dotenvy::dotenv;

use tokio;

use error::AppError;
use api::coincheck::client::CoincheckClient;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    dotenv().expect(".env file not found");

    api::slack::send_orderd_information("btc", "sell", 1.1).await?;

    let client = CoincheckClient::new()?;

    // 所持している全部の通貨を取得
    let my_currencies = repositories::balance::my_currencies(&client).await?;
    println!("{:#?}", my_currencies);

    // 所持している通貨の中からトレーディング対象の仮想通貨を取得
    let my_trading_currencies = repositories::balance::my_trading_currencies(&client).await?;
    println!("{:#?}", my_trading_currencies);

    // 保有量を取得
    print_my_balances(&client).await?;

    Ok(())
}

async fn print_my_balances(client: &CoincheckClient) -> Result<(), AppError> {
    println!("#-- 通貨保有量 ");
    let my_balances = repositories::balance::my_balancies(&client).await?;
    println!("{:#?}", my_balances);
    println!("");

    Ok(())
}
