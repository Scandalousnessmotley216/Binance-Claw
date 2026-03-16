use anyhow::Result;
use serde_json::Value;
use crate::types::{SkillResult, SkillStatus};

/// OpenClaw skill manifest
pub const SKILL_NAME: &str = "binance-claw";
pub const SKILL_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const SKILL_DESCRIPTION: &str =
    "Monitor Binance crypto prices in real-time. Set price alerts (claws), \
     watch order books, fetch 24h stats, and trigger automation on price events.";

/// Capabilities advertised to OpenClaw
pub fn skill_manifest() -> Value {
    serde_json::json!({
        "name": SKILL_NAME,
        "version": SKILL_VERSION,
        "description": SKILL_DESCRIPTION,
        "author": "deepcon3",
        "repository": "https://github.com/deepcon3/Binance-Claw",
        "commands": [
            {
                "name": "price",
                "description": "Get current price for a symbol",
                "args": ["symbol"],
                "example": "binance-claw price BTCUSDT"
            },
            {
                "name": "watch",
                "description": "Watch price in real-time via WebSocket",
                "args": ["symbol"],
                "flags": ["--interval", "--json"],
                "example": "binance-claw watch ETHUSDT --json"
            },
            {
                "name": "claw",
                "description": "Set a price alert/claw trigger",
                "args": ["symbol", "condition", "price"],
                "flags": ["--action", "--webhook", "--once"],
                "example": "binance-claw claw BTCUSDT above 70000"
            },
            {
                "name": "stats",
                "description": "Show 24h ticker statistics",
                "args": ["symbol"],
                "flags": ["--json"],
                "example": "binance-claw stats SOLUSDT --json"
            },
            {
                "name": "book",
                "description": "Display order book depth",
                "args": ["symbol"],
                "flags": ["--limit", "--json"],
                "example": "binance-claw book BTCUSDT --limit 10"
            },
            {
                "name": "symbols",
                "description": "List available trading pairs",
                "flags": ["--quote", "--json"],
                "example": "binance-claw symbols --quote USDT"
            },
            {
                "name": "ping",
                "description": "Check Binance API connectivity",
                "example": "binance-claw ping"
            },
            {
                "name": "skill",
                "description": "Show OpenClaw skill manifest",
                "flags": ["--json"],
                "example": "binance-claw skill --json"
            }
        ],
        "events": [
            {
                "name": "price_above",
                "description": "Emitted when price crosses above target"
            },
            {
                "name": "price_below",
                "description": "Emitted when price drops below target"
            },
            {
                "name": "percent_change",
                "description": "Emitted when price changes by specified percentage"
            }
        ],
        "openclaw": {
            "protocol": "1.0",
            "output_format": "json",
            "streaming": true
        }
    })
}

/// Format a result as OpenClaw-compatible JSON
pub fn format_result(result: &SkillResult) -> Result<String> {
    serde_json::to_string_pretty(result).map_err(Into::into)
}

/// Print OpenClaw skill output to stdout
pub fn print_skill_output(result: &SkillResult) {
    match serde_json::to_string_pretty(result) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("Failed to serialize skill output: {}", e),
    }
}
