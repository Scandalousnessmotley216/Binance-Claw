use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::config::AppConfig;
use crate::types::{MiniTickerEvent, TradeEvent};

pub type PriceSender = broadcast::Sender<PriceUpdate>;
pub type PriceReceiver = broadcast::Receiver<PriceUpdate>;

/// A price update emitted from the monitor
#[derive(Debug, Clone)]
pub struct PriceUpdate {
    pub symbol: String,
    pub price: f64,
    pub timestamp_ms: u64,
}

/// Spawns a WebSocket monitor for the given symbols.
/// Returns a broadcast channel receiver for price updates.
pub async fn spawn_ws_monitor(
    config: Arc<AppConfig>,
    symbols: Vec<String>,
) -> Result<PriceReceiver> {
    let (tx, rx) = broadcast::channel::<PriceUpdate>(512);
    let tx_clone = tx.clone();

    tokio::spawn(async move {
        let mut attempt = 0u32;
        let max_attempts = config.monitor.ws_reconnect_attempts;

        loop {
            attempt += 1;
            if attempt > max_attempts {
                error!("WebSocket: max reconnect attempts ({}) reached", max_attempts);
                break;
            }

            match run_ws_stream(&config, &symbols, &tx_clone).await {
                Ok(_) => {
                    info!("WebSocket stream closed cleanly");
                    break;
                }
                Err(e) => {
                    warn!("WebSocket error (attempt {}): {}", attempt, e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                }
            }
        }
    });

    Ok(rx)
}

async fn run_ws_stream(
    config: &AppConfig,
    symbols: &[String],
    tx: &PriceSender,
) -> Result<()> {
    // Build combined stream URL: wss://stream.binance.com:9443/stream?streams=btcusdt@miniTicker/...
    let streams: Vec<String> = symbols
        .iter()
        .map(|s| format!("{}@miniTicker", s.to_lowercase()))
        .collect();
    let stream_path = streams.join("/");

    let url = format!(
        "{}/stream?streams={}",
        config.binance.ws_url, stream_path
    );

    info!("Connecting to Binance WebSocket: {} stream(s)", symbols.len());
    debug!("WS URL: {}", url);

    let (ws_stream, _) = connect_async(&url)
        .await
        .context("Failed to connect WebSocket")?;

    info!("WebSocket connected ✓");

    let (mut _write, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Ok(envelope) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(data) = envelope.get("data") {
                        if let Ok(event) = serde_json::from_value::<MiniTickerEvent>(data.clone()) {
                            if let Ok(price) = event.close_price.parse::<f64>() {
                                let update = PriceUpdate {
                                    symbol: event.symbol.clone(),
                                    price,
                                    timestamp_ms: chrono::Utc::now().timestamp_millis() as u64,
                                };
                                let _ = tx.send(update);
                            }
                        }
                    }
                }
            }
            Ok(Message::Ping(payload)) => {
                debug!("WS Ping received");
                // tungstenite auto-handles pong
                let _ = payload;
            }
            Ok(Message::Close(_)) => {
                info!("WebSocket server closed the connection");
                break;
            }
            Err(e) => {
                return Err(e.into());
            }
            _ => {}
        }
    }

    Ok(())
}

/// Polls prices via REST API at a fixed interval (fallback for single symbols)
pub async fn spawn_rest_monitor(
    config: Arc<AppConfig>,
    symbol: String,
    tx: PriceSender,
) {
    let interval_ms = config.monitor.poll_interval_ms;
    let client = crate::api::BinanceClient::new(config.binance.clone())
        .expect("Failed to build REST client for monitor");

    tokio::spawn(async move {
        let mut interval =
            tokio::time::interval(tokio::time::Duration::from_millis(interval_ms));
        loop {
            interval.tick().await;
            match client.get_price(&symbol).await {
                Ok(price) => {
                    let update = PriceUpdate {
                        symbol: symbol.clone(),
                        price,
                        timestamp_ms: chrono::Utc::now().timestamp_millis() as u64,
                    };
                    let _ = tx.send(update);
                }
                Err(e) => {
                    warn!("REST poll error for {}: {}", symbol, e);
                }
            }
        }
    });
}
