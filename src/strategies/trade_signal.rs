#[allow(dead_code)]
pub enum TradeSignal {
    MarcketBuy {
        amount: f64,
        reason: Option<String>,
    },
    MarcketSell {
        amount: f64,
        reason: Option<String>,
    },
    Hold {
        reason: Option<String>,
    },
    InsufficientData {
        reason: Option<String>,
    },
}
