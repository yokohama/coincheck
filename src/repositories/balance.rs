use std::error::Error;

use serde_json::Value;

use crate::api::coincheck;
use crate::api::coincheck::client::CoincheckClient;

pub async fn my_currencies(client: &CoincheckClient) -> Result<Vec<String>, Box<dyn Error>> {

    let balancies = coincheck::balance::find(&client).await?;
    let currencies = balancies
        .as_object()
        .unwrap()
        .iter()
        .filter(|(_, v)| v.as_str().unwrap_or("0.0") != "0.0" )
        .map(|(k, _)| k.to_string())
        .collect();

    Ok(currencies)
}

pub async fn my_managed_currencies(client: &CoincheckClient) -> Result<Vec<String>, Box<dyn Error>> {

    let balancies = coincheck::balance::find(&client).await?;
    let currencies = balancies
        .as_object()
        .unwrap()
        .iter()
        .filter(|(_, v)| v.as_str().unwrap_or("0.0") != "0.0" )
        .filter(|(k, _)| !k.contains("tumitate"))
        .map(|(k, _)| k.to_string())
        .collect();

    Ok(currencies)
}

pub async fn my_trading_currencies(client: &CoincheckClient) -> Result<Vec<String>, Box<dyn Error>> {

    let balancies = coincheck::balance::find(&client).await?;
    let currencies = balancies
        .as_object()
        .unwrap()
        .iter()
        .filter(|(_, v)| v.as_str().unwrap_or("0.0") != "0.0" )
        .filter(|(k, _)| !k.contains("tsumitate") && !k.contains("jpy"))
        .map(|(k, _)| k.to_string())
        .collect();

    Ok(currencies)
}

pub async fn my_balancies(client: &CoincheckClient) -> Result<Value, Box<dyn Error>> {

    let balances = coincheck::balance::find(&client).await?;
    let my_balancies: serde_json::Map<String, Value> = balances
        .as_object()
        .unwrap()
        .iter()
        .filter_map(|(k, v)| {
            v.as_str()
                .and_then(|s| s.parse::<f64>().ok())
                .filter(|&n| n != 0.0)
                .map(|num| (k.clone(), Value::from(num)))
        })
        .collect();

    Ok(Value::Object(my_balancies))
}
