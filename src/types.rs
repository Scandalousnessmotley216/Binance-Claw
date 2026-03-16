use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Binance ticker price response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TickerPrice {
    pub symbol: String,
    pub price: String,
}

/// Binance 24hr ticker statistics
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ticker24h {
    pub symbol: String,
    pub price_change: String,
    pub price_change_percent: String,
    pub last_price: String,
    pub high_price: String,
    pub low_price: String,
    pub volume: String,
    pub count: u64,
}

/// WebSocket stream trade event
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TradeEvent {
    #[serde(rename = "e")]
    pub event_type: String,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "p")]
    pub price: String,
    #[serde(rename = "q")]
    pub quantity: String,
    #[serde(rename = "T")]
    pub trade_time: u64,
    #[serde(rename = "m")]
    pub is_buyer_maker: bool,
}

/// WebSocket mini-ticker event
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MiniTickerEvent {
    #[serde(rename = "e")]
    pub event_type: String,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "c")]
    pub close_price: String,
    #[serde(rename = "o")]
    pub open_price: String,
    #[serde(rename = "h")]
    pub high_price: String,
    #[serde(rename = "l")]
    pub low_price: String,
    #[serde(rename = "v")]
    pub base_volume: String,
}

/// A price alert/claw target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClawTarget {
    pub symbol: String,
    pub condition: ClawCondition,
    pub target_price: f64,
    pub created_at: DateTime<Utc>,
    pub triggered: bool,
    pub triggered_at: Option<DateTime<Utc>>,
    pub triggered_price: Option<f64>,
    pub action: ClawAction,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ClawCondition {
    /// Price goes above target
    Above,
    /// Price drops below target
    Below,
    /// Price changes by X% from current
    PercentChange(f64),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ClawAction {
    /// Just notify
    Alert,
    /// Print a webhook payload (for external automation)
    Webhook(String),
    /// Print an OpenClaw skill trigger command
    OpenClawTrigger(String),
}

impl ClawTarget {
    pub fn new(
        symbol: impl Into<String>,
        condition: ClawCondition,
        target_price: f64,
        action: ClawAction,
    ) -> Self {
        Self {
            symbol: symbol.into().to_uppercase(),
            condition,
            target_price,
            created_at: Utc::now(),
            triggered: false,
            triggered_at: None,
            triggered_price: None,
            action,
        }
    }

    /// Returns true if the current price satisfies this target's condition
    pub fn check(&self, current_price: f64) -> bool {
        if self.triggered {
            return false;
        }
        match &self.condition {
            ClawCondition::Above => current_price >= self.target_price,
            ClawCondition::Below => current_price <= self.target_price,
            ClawCondition::PercentChange(pct) => {
                let change = ((current_price - self.target_price) / self.target_price).abs() * 100.0;
                change >= pct.abs()
            }
        }
    }
}

/// Order book snapshot
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderBook {
    pub last_update_id: u64,
    pub bids: Vec<[String; 2]>,
    pub asks: Vec<[String; 2]>,
}

/// Exchange info for a symbol
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SymbolInfo {
    pub symbol: String,
    pub status: String,
    pub base_asset: String,
    pub quote_asset: String,
}

/// OpenClaw skill result format
#[derive(Debug, Serialize, Deserialize)]
pub struct SkillResult {
    pub skill: String,
    pub version: String,
    pub timestamp: DateTime<Utc>,
    pub data: serde_json::Value,
    pub status: SkillStatus,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SkillStatus {
    Ok,
    Error,
    Triggered,
}

impl SkillResult {
    pub fn ok(data: serde_json::Value) -> Self {
        Self {
            skill: "binance-claw".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            timestamp: Utc::now(),
            data,
            status: SkillStatus::Ok,
        }
    }

    pub fn error(msg: &str) -> Self {
        Self {
            skill: "binance-claw".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            timestamp: Utc::now(),
            data: serde_json::json!({ "error": msg }),
            status: SkillStatus::Error,
        }
    }
}
