#[allow(dead_code)]
pub enum TradeSignal {
    MarcketBuy {
        ma_short: Option<i32>,
        ma_long: Option<i32>,
        ma_win_rate: Option<f64>,
        amount: f64,
        reason: Option<String>,
    },
    MarcketSell {
        ma_short: Option<i32>,
        ma_long: Option<i32>,
        ma_win_rate: Option<f64>,
        amount: f64,
        reason: Option<String>,
    },
    Hold {
        ma_short: Option<i32>,
        ma_long: Option<i32>,
        ma_win_rate: Option<f64>,
        reason: Option<String>,
    },
    InsufficientData {
        ma_short: Option<i32>,
        ma_long: Option<i32>,
        ma_win_rate: Option<f64>,
        reason: Option<String>,
    },
}
