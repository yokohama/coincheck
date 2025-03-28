#[allow(dead_code)]
pub enum TradeSignal {
    MarcketBuy {
        spread_threshold: Option<f64>,
        spread_ratio: Option<f64>,
        ma_short: Option<i32>,
        ma_long: Option<i32>,
        ma_win_rate: Option<f64>,
        amount: f64,
        reason: Option<String>,
    },
    MarcketSell {
        spread_threshold: Option<f64>,
        spread_ratio: Option<f64>,
        ma_short: Option<i32>,
        ma_long: Option<i32>,
        ma_win_rate: Option<f64>,
        amount: f64,
        reason: Option<String>,
    },
    Hold {
        spread_threshold: Option<f64>,
        spread_ratio: Option<f64>,
        ma_short: Option<i32>,
        ma_long: Option<i32>,
        ma_win_rate: Option<f64>,
        reason: Option<String>,
    },
    InsufficientData {
        spread_threshold: Option<f64>,
        spread_ratio: Option<f64>,
        ma_short: Option<i32>,
        ma_long: Option<i32>,
        ma_win_rate: Option<f64>,
        reason: Option<String>,
    },
}
