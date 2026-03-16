use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::sync::Arc;
use tracing::info;
use toml;

use crate::api::BinanceClient;
use crate::claw::{ClawEngine};
use crate::config::AppConfig;
use crate::monitor::{spawn_ws_monitor, spawn_rest_monitor, PriceSender};
use crate::notify::Notifier;
use crate::skill::{print_skill_output, skill_manifest};
use crate::types::{ClawAction, ClawCondition, ClawTarget, SkillResult};
use crate::utils::{fmt_change, fmt_price, fmt_volume, print_banner};

#[derive(Parser)]
#[command(
    name = "binance-claw",
    version,
    author = "deepcon3",
    about = "⚡ Lightning-fast Binance price sniper & OpenClaw skill",
    long_about = "Binance-Claw monitors crypto prices in real-time via WebSocket streams,\n\
                  fires alerts when your price targets are hit, and integrates as an\n\
                  OpenClaw skill for automation workflows.",
    after_help = "EXAMPLES:\n  \
        binance-claw price BTCUSDT\n  \
        binance-claw watch ETHUSDT\n  \
        binance-claw claw BTCUSDT above 70000\n  \
        binance-claw claw ETHUSDT below 3000 --once\n  \
        binance-claw stats SOLUSDT --json\n  \
        binance-claw book BTCUSDT --limit 10\n  \
        binance-claw symbols --quote USDT\n  \
        binance-claw skill --json"
)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output in OpenClaw JSON format
    #[arg(long, global = true)]
    openclaw: bool,

    /// Suppress banner
    #[arg(long, global = true, short = 'q')]
    quiet: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Get the current price of a symbol
    Price {
        /// Trading pair (e.g. BTCUSDT, ETHUSDT)
        symbol: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Watch a symbol's price in real-time
    Watch {
        /// Trading pair (e.g. BTCUSDT)
        symbol: String,
        /// Polling interval in ms (REST fallback, default: 1000)
        #[arg(long, default_value = "1000")]
        interval: u64,
        /// Output as JSON stream
        #[arg(long)]
        json: bool,
        /// Use REST polling instead of WebSocket
        #[arg(long)]
        rest: bool,
    },

    /// Set a price alert (claw trigger) for a symbol
    Claw {
        /// Trading pair (e.g. BTCUSDT)
        symbol: String,
        /// Condition: above | below | change
        condition: String,
        /// Target price
        price: f64,
        /// Fire only once, then exit
        #[arg(long)]
        once: bool,
        /// Webhook URL to POST when triggered
        #[arg(long)]
        webhook: Option<String>,
        /// OpenClaw trigger command on fire
        #[arg(long)]
        openclaw_trigger: Option<String>,
    },

    /// Show 24h ticker statistics
    Stats {
        /// Trading pair (e.g. BTCUSDT)
        symbol: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show order book depth
    Book {
        /// Trading pair (e.g. BTCUSDT)
        symbol: String,
        /// Number of levels to show (default: 5)
        #[arg(long, default_value = "5")]
        limit: u32,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// List available trading symbols
    Symbols {
        /// Filter by quote asset (e.g. USDT, BTC)
        #[arg(long)]
        quote: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Check Binance API connectivity
    Ping,

    /// Show OpenClaw skill manifest
    Skill {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show config file path and contents
    Config {
        /// Show raw TOML
        #[arg(long)]
        show: bool,
        /// Open config directory
        #[arg(long)]
        path: bool,
    },
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let mut cfg = AppConfig::load()?;
        cfg.apply_env();

        if !self.quiet {
            if !matches!(self.command, Commands::Skill { .. }) {
                print_banner();
            }
        }

        let client = BinanceClient::new(cfg.binance.clone())
            .context("Failed to initialize Binance client")?;

        let openclaw = self.openclaw || cfg.openclaw.enabled;

        match self.command {
            Commands::Price { symbol, json } => {
                cmd_price(&client, &symbol, json, openclaw).await?
            }
            Commands::Watch { symbol, interval, json, rest } => {
                cmd_watch(&cfg, &symbol, interval, json, rest, openclaw).await?
            }
            Commands::Claw { symbol, condition, price, once, webhook, openclaw_trigger } => {
                cmd_claw(&cfg, &symbol, &condition, price, once, webhook, openclaw_trigger, openclaw).await?
            }
            Commands::Stats { symbol, json } => {
                cmd_stats(&client, &symbol, json, openclaw).await?
            }
            Commands::Book { symbol, limit, json } => {
                cmd_book(&client, &symbol, limit, json, openclaw).await?
            }
            Commands::Symbols { quote, json } => {
                cmd_symbols(&client, quote.as_deref(), json, openclaw).await?
            }
            Commands::Ping => cmd_ping(&client).await?,
            Commands::Skill { json } => cmd_skill(json),
            Commands::Config { show, path } => cmd_config(show, path)?,
        }

        Ok(())
    }
}

// ─── Command implementations ────────────────────────────────────────────────

async fn cmd_price(
    client: &BinanceClient,
    symbol: &str,
    json: bool,
    openclaw: bool,
) -> Result<()> {
    let price = client.get_price(symbol).await?;

    if openclaw || json {
        let result = SkillResult::ok(serde_json::json!({
            "symbol": symbol.to_uppercase(),
            "price": price,
        }));
        print_skill_output(&result);
    } else {
        println!(
            "{} {} {}",
            symbol.to_uppercase().cyan().bold(),
            "→".dimmed(),
            fmt_price(price).yellow().bold()
        );
    }
    Ok(())
}

async fn cmd_watch(
    cfg: &AppConfig,
    symbol: &str,
    interval: u64,
    json: bool,
    rest: bool,
    openclaw: bool,
) -> Result<()> {
    let symbol_up = symbol.to_uppercase();
    println!(
        "👁  Watching {} — press {} to stop\n",
        symbol_up.cyan().bold(),
        "Ctrl+C".dimmed()
    );

    let cfg = Arc::new(cfg.clone());
    let (tx, mut rx) = tokio::sync::broadcast::channel(512);

    if rest {
        let mut cfg2 = (*cfg).clone();
        cfg2.monitor.poll_interval_ms = interval;
        spawn_rest_monitor(Arc::new(cfg2), symbol_up.clone(), tx).await;
    } else {
        let mut rx2 = spawn_ws_monitor(cfg.clone(), vec![symbol_up.clone()]).await?;
        // re-broadcast from ws channel
        let tx2 = tx.clone();
        tokio::spawn(async move {
            while let Ok(u) = rx2.recv().await {
                let _ = tx2.send(u);
            }
        });
    }

    let mut last_price: Option<f64> = None;

    loop {
        match rx.recv().await {
            Ok(update) if update.symbol == symbol_up => {
                let price = update.price;
                let arrow = match last_price {
                    Some(lp) if price > lp => "▲".green().to_string(),
                    Some(lp) if price < lp => "▼".red().to_string(),
                    _ => "─".dimmed().to_string(),
                };
                last_price = Some(price);

                if openclaw || json {
                    let result = SkillResult::ok(serde_json::json!({
                        "symbol": symbol_up,
                        "price": price,
                        "timestamp_ms": update.timestamp_ms,
                    }));
                    print_skill_output(&result);
                } else {
                    let ts = chrono::Utc::now().format("%H:%M:%S");
                    print!(
                        "\r  {} {} {}   ",
                        ts.to_string().dimmed(),
                        arrow,
                        fmt_price(price).yellow().bold()
                    );
                    use std::io::Write;
                    std::io::stdout().flush().ok();
                }
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
            Err(_) => break,
            _ => {}
        }
    }
    Ok(())
}

async fn cmd_claw(
    cfg: &AppConfig,
    symbol: &str,
    condition_str: &str,
    price: f64,
    once: bool,
    webhook: Option<String>,
    openclaw_trigger: Option<String>,
    openclaw: bool,
) -> Result<()> {
    let symbol_up = symbol.to_uppercase();

    let condition = match condition_str.to_lowercase().as_str() {
        "above" | ">" | ">=" => ClawCondition::Above,
        "below" | "<" | "<=" => ClawCondition::Below,
        s if s.ends_with('%') => {
            let pct: f64 = s.trim_end_matches('%').parse().unwrap_or(5.0);
            ClawCondition::PercentChange(pct)
        }
        s => {
            let pct: f64 = s.parse().unwrap_or(5.0);
            ClawCondition::PercentChange(pct)
        }
    };

    let action = if let Some(url) = webhook {
        ClawAction::Webhook(url)
    } else if let Some(cmd) = openclaw_trigger {
        ClawAction::OpenClawTrigger(cmd)
    } else {
        ClawAction::Alert
    };

    let notifier = Notifier::new(cfg.alerts.desktop_notify, cfg.alerts.bell);
    let mut engine = ClawEngine::new(notifier, openclaw);

    let target = ClawTarget::new(symbol_up.clone(), condition, price, action);
    engine.add_target(target);

    // Get current price for context
    let client = BinanceClient::new(cfg.binance.clone())?;
    let current = client.get_price(&symbol_up).await?;
    println!(
        "  Current price : {}",
        fmt_price(current).yellow()
    );
    println!(
        "  Watching      : {} target(s)\n",
        engine.target_count().to_string().cyan()
    );

    // Start WebSocket monitor
    let cfg_arc = Arc::new(cfg.clone());
    let mut rx = spawn_ws_monitor(cfg_arc, vec![symbol_up.clone()]).await?;

    loop {
        match rx.recv().await {
            Ok(update) => {
                let fired = engine.process_update(&update).await;
                if once && !fired.is_empty() {
                    break;
                }
                if engine.all_triggered() {
                    break;
                }
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
            Err(_) => break,
        }
    }

    Ok(())
}

async fn cmd_stats(
    client: &BinanceClient,
    symbol: &str,
    json: bool,
    openclaw: bool,
) -> Result<()> {
    let t = client.get_24h_ticker(symbol).await?;

    if openclaw || json {
        let result = SkillResult::ok(serde_json::to_value(&t)?);
        print_skill_output(&result);
        return Ok(());
    }

    println!("  {} — 24h Stats", t.symbol.cyan().bold());
    println!("  ─────────────────────────────");
    println!("  Last price   : {}", fmt_price(t.last_price.parse().unwrap_or(0.0)).yellow().bold());
    println!("  Change       : {}", fmt_change(&t.price_change_percent));
    println!("  High         : {}", fmt_price(t.high_price.parse().unwrap_or(0.0)).green());
    println!("  Low          : {}", fmt_price(t.low_price.parse().unwrap_or(0.0)).red());
    println!("  Volume       : {}", fmt_volume(&t.volume).cyan());
    println!("  Trades       : {}", t.count.to_string().dimmed());
    Ok(())
}

async fn cmd_book(
    client: &BinanceClient,
    symbol: &str,
    limit: u32,
    json: bool,
    openclaw: bool,
) -> Result<()> {
    let book = client.get_order_book(symbol, limit).await?;

    if openclaw || json {
        let result = SkillResult::ok(serde_json::to_value(&book)?);
        print_skill_output(&result);
        return Ok(());
    }

    println!("  {} — Order Book (top {})", symbol.to_uppercase().cyan().bold(), limit);
    println!("  {:>18}  {:>18}", "ASK (sell)".red(), "BID (buy)".green());
    println!("  {:>18}  {:>18}", "─────────────────".dimmed(), "─────────────────".dimmed());

    let asks: Vec<_> = book.asks.iter().take(limit as usize).collect();
    let bids: Vec<_> = book.bids.iter().take(limit as usize).collect();

    let rows = asks.len().max(bids.len());
    for i in 0..rows {
        let ask = asks.get(i).map(|a| format!("{} @ {}", a[1], a[0])).unwrap_or_default();
        let bid = bids.get(i).map(|b| format!("{} @ {}", b[1], b[0])).unwrap_or_default();
        println!("  {:>18}  {:>18}", ask.red(), bid.green());
    }
    Ok(())
}

async fn cmd_symbols(
    client: &BinanceClient,
    quote: Option<&str>,
    json: bool,
    openclaw: bool,
) -> Result<()> {
    let symbols = client.list_symbols(quote).await?;
    let active: Vec<_> = symbols.iter().filter(|s| s.status == "TRADING").collect();

    if openclaw || json {
        let result = SkillResult::ok(serde_json::to_value(&active)?);
        print_skill_output(&result);
        return Ok(());
    }

    let filter_msg = quote
        .map(|q| format!(" (quote: {})", q.to_uppercase()))
        .unwrap_or_default();

    println!(
        "  {} active trading pairs{}:\n",
        active.len().to_string().cyan().bold(),
        filter_msg.dimmed()
    );

    let cols = 6usize;
    for (i, s) in active.iter().enumerate() {
        print!("  {:12}", s.symbol.cyan());
        if (i + 1) % cols == 0 {
            println!();
        }
    }
    if active.len() % cols != 0 {
        println!();
    }
    Ok(())
}

async fn cmd_ping(client: &BinanceClient) -> Result<()> {
    let start = std::time::Instant::now();
    client.ping().await?;
    let elapsed = start.elapsed();
    println!(
        "  Binance API {} — {:.1}ms latency",
        "ONLINE".green().bold(),
        elapsed.as_secs_f64() * 1000.0
    );
    let ts = client.server_time().await?;
    println!(
        "  Server time   : {}",
        chrono::DateTime::from_timestamp_millis(ts as i64)
            .map(|t: chrono::DateTime<chrono::Utc>| t.format("%Y-%m-%d %H:%M:%S UTC").to_string())
            .unwrap_or_else(|| ts.to_string())
            .dimmed()
    );
    Ok(())
}

fn cmd_skill(json: bool) {
    let manifest = skill_manifest();
    if json {
        println!("{}", serde_json::to_string_pretty(&manifest).unwrap());
    } else {
        println!("{}", serde_json::to_string_pretty(&manifest).unwrap());
    }
}

fn cmd_config(show: bool, path: bool) -> Result<()> {
    let config_path = AppConfig::config_path()?;

    if path {
        println!("{}", config_path.display());
        return Ok(());
    }

    println!("  Config file: {}", config_path.display().to_string().cyan());

    if show {
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            println!("\n{}", content);
        } else {
            println!("  (no config file — using defaults)");
            let cfg = AppConfig::default();
            println!("\n{}", toml::to_string_pretty(&cfg)?);
        }
    }
    Ok(())
}
