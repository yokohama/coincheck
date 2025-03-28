use crate::models::order::NewOrder;

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

impl TradeSignal {
    pub fn apply_to(&self, new_order: &mut NewOrder) {
        match self {
            TradeSignal::MarcketBuy { spread_threshold, spread_ratio, ma_short, ma_long, ma_win_rate, amount, reason } => {
                new_order.order_type = "market_buy".to_string();
                new_order.spread_threshold = *spread_threshold;
                new_order.spread_ratio = *spread_ratio;
                new_order.ma_short = *ma_short;
                new_order.ma_long = *ma_long;
                new_order.ma_win_rate = *ma_win_rate;
                new_order.jpy_amount = *amount;
                new_order.crypto_amount = 0.0;
                new_order.comment = reason.clone();
            },
            TradeSignal::MarcketSell { spread_threshold, spread_ratio, ma_short, ma_long, ma_win_rate, amount, reason } => {
                new_order.order_type = "market_sell".to_string();
                new_order.spread_threshold = *spread_threshold;
                new_order.spread_ratio = *spread_ratio;
                new_order.ma_short = *ma_short;
                new_order.ma_long = *ma_long;
                new_order.ma_win_rate = *ma_win_rate;
                new_order.jpy_amount = 0.0;
                new_order.crypto_amount = *amount;
                new_order.comment = reason.clone();
            },
            TradeSignal::Hold { spread_threshold, spread_ratio, ma_short, ma_long, ma_win_rate, reason }
            | TradeSignal::InsufficientData { spread_threshold, spread_ratio, ma_short, ma_long, ma_win_rate, reason } => {
                new_order.order_type = match self {
                    TradeSignal::Hold { .. } => "hold",
                    _ => "insufficient_data",
                }.to_string();
                new_order.spread_threshold = *spread_threshold;
                new_order.spread_ratio = *spread_ratio;
                new_order.ma_short = *ma_short;
                new_order.ma_long = *ma_long;
                new_order.ma_win_rate = *ma_win_rate;
                new_order.jpy_amount = 0.0;
                new_order.crypto_amount = 0.0;
                new_order.comment = reason.clone();
            },
        }
    }
}
