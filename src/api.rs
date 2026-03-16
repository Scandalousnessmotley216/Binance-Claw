use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use std::time::Duration;
use crate::config::BinanceConfig;
use crate::types::{TickerPrice, Ticker24h, OrderBook, SymbolInfo};

/// Binance REST API client
pub struct BinanceClient {
    client: Client,
    config: BinanceConfig,
}

impl BinanceClient {
    pub fn new(config: BinanceConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .user_agent(format!(
                "binance-claw/{} (github.com/deepcon3/Binance-Claw)",
                env!("CARGO_PKG_VERSION")
            ))
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self { client, config })
    }

    /// GET /api/v3/ticker/price — single symbol
    pub async fn get_price(&self, symbol: &str) -> Result<f64> {
        let url = format!(
            "{}/api/v3/ticker/price?symbol={}",
            self.config.api_url,
            symbol.to_uppercase()
        );
        let resp: TickerPrice = self
            .client
            .get(&url)
            .send()
            .await
            .context("Request failed")?
            .error_for_status()
            .context("Binance API error")?
            .json()
            .await
            .context("Failed to parse price response")?;

        resp.price
            .parse::<f64>()
            .with_context(|| format!("Invalid price string: {}", resp.price))
    }

    /// GET /api/v3/ticker/price — all symbols
    pub async fn get_all_prices(&self) -> Result<Vec<TickerPrice>> {
        let url = format!("{}/api/v3/ticker/price", self.config.api_url);
        self.client
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
            .context("Failed to parse all-prices response")
    }

    /// GET /api/v3/ticker/24hr — 24h stats for a symbol
    pub async fn get_24h_ticker(&self, symbol: &str) -> Result<Ticker24h> {
        let url = format!(
            "{}/api/v3/ticker/24hr?symbol={}",
            self.config.api_url,
            symbol.to_uppercase()
        );
        self.client
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
            .context("Failed to parse 24h ticker")
    }

    /// GET /api/v3/depth — order book
    pub async fn get_order_book(&self, symbol: &str, limit: u32) -> Result<OrderBook> {
        let url = format!(
            "{}/api/v3/depth?symbol={}&limit={}",
            self.config.api_url,
            symbol.to_uppercase(),
            limit
        );
        self.client
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
            .context("Failed to parse order book")
    }

    /// GET /api/v3/exchangeInfo — list available symbols
    pub async fn list_symbols(&self, quote: Option<&str>) -> Result<Vec<SymbolInfo>> {
        let url = format!("{}/api/v3/exchangeInfo", self.config.api_url);
        let resp: serde_json::Value = self
            .client
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let symbols = resp["symbols"]
            .as_array()
            .ok_or_else(|| anyhow!("No symbols array in exchange info"))?
            .iter()
            .filter_map(|s| {
                let symbol = s["symbol"].as_str()?.to_string();
                let status = s["status"].as_str()?.to_string();
                let base_asset = s["baseAsset"].as_str()?.to_string();
                let quote_asset = s["quoteAsset"].as_str()?.to_string();

                if let Some(q) = quote {
                    if !quote_asset.eq_ignore_ascii_case(q) {
                        return None;
                    }
                }

                Some(SymbolInfo { symbol, status, base_asset, quote_asset })
            })
            .collect();

        Ok(symbols)
    }

    /// GET /api/v3/ping — connectivity check
    pub async fn ping(&self) -> Result<()> {
        let url = format!("{}/api/v3/ping", self.config.api_url);
        self.client
            .get(&url)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// GET /api/v3/time — server time
    pub async fn server_time(&self) -> Result<u64> {
        let url = format!("{}/api/v3/time", self.config.api_url);
        let resp: serde_json::Value = self
            .client
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        resp["serverTime"]
            .as_u64()
            .ok_or_else(|| anyhow!("No serverTime in response"))
    }
}
