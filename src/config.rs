use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub binance: BinanceConfig,
    pub monitor: MonitorConfig,
    pub alerts: AlertConfig,
    pub openclaw: OpenClawConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceConfig {
    /// Binance REST API base URL
    pub api_url: String,
    /// Binance WebSocket stream base URL
    pub ws_url: String,
    /// Optional API key (for signed endpoints)
    pub api_key: Option<String>,
    /// Optional API secret (for signed endpoints)
    pub api_secret: Option<String>,
    /// Request timeout in seconds
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    /// Polling interval in milliseconds (REST fallback)
    pub poll_interval_ms: u64,
    /// Use WebSocket streams (recommended)
    pub use_websocket: bool,
    /// Max reconnect attempts for WebSocket
    pub ws_reconnect_attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    /// Enable desktop notifications
    pub desktop_notify: bool,
    /// Enable terminal bell
    pub bell: bool,
    /// Enable sound (requires `sound` feature)
    pub sound: bool,
    /// Webhook URL to POST alert payload to
    pub webhook_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenClawConfig {
    /// Output results in OpenClaw JSON format
    pub enabled: bool,
    /// OpenClaw skill endpoint (if using remote)
    pub endpoint: Option<String>,
    /// Skill name registered in OpenClaw
    pub skill_name: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            binance: BinanceConfig {
                api_url: "https://api.binance.com".into(),
                ws_url: "wss://stream.binance.com:9443".into(),
                api_key: None,
                api_secret: None,
                timeout_secs: 10,
            },
            monitor: MonitorConfig {
                poll_interval_ms: 1000,
                use_websocket: true,
                ws_reconnect_attempts: 5,
            },
            alerts: AlertConfig {
                desktop_notify: true,
                bell: true,
                sound: false,
                webhook_url: None,
            },
            openclaw: OpenClawConfig {
                enabled: false,
                endpoint: None,
                skill_name: "binance-claw".into(),
            },
        }
    }
}

impl AppConfig {
    /// Load config from the default config file, falling back to defaults
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;

        if !path.exists() {
            let cfg = AppConfig::default();
            cfg.save()?;
            return Ok(cfg);
        }

        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config at {:?}", path))?;

        toml::from_str(&content).with_context(|| "Failed to parse config TOML")
    }

    /// Save config to disk
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)
            .with_context(|| format!("Failed to write config to {:?}", path))?;
        Ok(())
    }

    /// Returns the config file path
    pub fn config_path() -> Result<PathBuf> {
        let base = dirs::config_dir()
            .or_else(dirs::home_dir)
            .context("Could not find home/config directory")?;
        Ok(base.join("binance-claw").join("config.toml"))
    }

    /// Apply environment variable overrides
    pub fn apply_env(&mut self) {
        if let Ok(key) = std::env::var("BINANCE_API_KEY") {
            self.binance.api_key = Some(key);
        }
        if let Ok(secret) = std::env::var("BINANCE_API_SECRET") {
            self.binance.api_secret = Some(secret);
        }
        if let Ok(url) = std::env::var("BINANCE_API_URL") {
            self.binance.api_url = url;
        }
    }
}
