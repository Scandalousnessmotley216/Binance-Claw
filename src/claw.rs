use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;
use tracing::{info, warn};
use chrono::Utc;

use crate::types::{ClawAction, ClawCondition, ClawTarget, SkillResult};
use crate::monitor::PriceUpdate;
use crate::notify::Notifier;

/// The Claw engine: holds targets and processes price updates
pub struct ClawEngine {
    /// symbol -> list of targets
    targets: HashMap<String, Vec<ClawTarget>>,
    notifier: Notifier,
    openclaw_mode: bool,
}

impl ClawEngine {
    pub fn new(notifier: Notifier, openclaw_mode: bool) -> Self {
        Self {
            targets: HashMap::new(),
            notifier,
            openclaw_mode,
        }
    }

    /// Add a claw target
    pub fn add_target(&mut self, target: ClawTarget) {
        let symbol = target.symbol.clone();
        info!(
            "🎯 Claw target added: {} {} {:.8}",
            symbol.cyan(),
            condition_label(&target.condition),
            target.target_price
        );
        self.targets.entry(symbol).or_default().push(target);
    }

    /// Total number of active targets
    pub fn target_count(&self) -> usize {
        self.targets.values().map(|v| v.len()).sum()
    }

    /// Returns symbols that have at least one active target
    pub fn watched_symbols(&self) -> Vec<String> {
        self.targets.keys().cloned().collect()
    }

    /// Process a price update, firing any matching targets
    pub async fn process_update(&mut self, update: &PriceUpdate) -> Vec<FiredTarget> {
        let symbol = update.symbol.to_uppercase();
        let price = update.price;
        let mut fired = Vec::new();

        if let Some(targets) = self.targets.get_mut(&symbol) {
            for target in targets.iter_mut() {
                if !target.triggered && target.check(price) {
                    target.triggered = true;
                    target.triggered_at = Some(Utc::now());
                    target.triggered_price = Some(price);

                    let ft = FiredTarget {
                        target: target.clone(),
                        current_price: price,
                    };

                    self.handle_fired(&ft).await;
                    fired.push(ft);
                }
            }
        }

        fired
    }

    async fn handle_fired(&self, ft: &FiredTarget) {
        let symbol = &ft.target.symbol;
        let price = ft.current_price;
        let target_price = ft.target.target_price;

        // Terminal output
        let msg = format!(
            "🔔 CLAW TRIGGERED! {} — price {} (target: {})",
            symbol.yellow().bold(),
            format!("{:.8}", price).green().bold(),
            format!("{:.8}", target_price).cyan()
        );
        println!("\n{}", msg);
        println!("   Condition : {}", condition_label(&ft.target.condition).magenta());
        println!("   Time      : {}", ft.target.triggered_at.unwrap().format("%Y-%m-%d %H:%M:%S UTC"));

        // Desktop notification
        if let Err(e) = self.notifier.send(
            &format!("Binance Claw — {}", symbol),
            &format!(
                "Price {} — target {} {}",
                price_fmt(price),
                condition_label(&ft.target.condition),
                price_fmt(target_price)
            ),
        ).await {
            warn!("Notification error: {}", e);
        }

        // OpenClaw JSON output
        if self.openclaw_mode {
            let result = SkillResult::ok(serde_json::json!({
                "symbol": symbol,
                "triggered_price": price,
                "target_price": target_price,
                "condition": format!("{:?}", ft.target.condition),
                "triggered_at": ft.target.triggered_at,
            }));
            if let Ok(json) = serde_json::to_string_pretty(&result) {
                println!("\n--- OpenClaw Skill Output ---");
                println!("{}", json);
            }
        }

        // Custom action
        match &ft.target.action {
            ClawAction::Alert => {}
            ClawAction::Webhook(url) => {
                self.fire_webhook(url, ft).await;
            }
            ClawAction::OpenClawTrigger(cmd) => {
                println!("🤖 OpenClaw trigger: {}", cmd.cyan());
            }
        }
    }

    async fn fire_webhook(&self, url: &str, ft: &FiredTarget) {
        let payload = serde_json::json!({
            "source": "binance-claw",
            "symbol": ft.target.symbol,
            "triggered_price": ft.current_price,
            "target_price": ft.target.target_price,
            "condition": format!("{:?}", ft.target.condition),
            "triggered_at": ft.target.triggered_at,
        });

        info!("Firing webhook: {}", url);
        if let Ok(client) = reqwest::Client::builder().build() {
            let _ = client
                .post(url)
                .json(&payload)
                .send()
                .await;
        }
    }

    /// Remove all triggered targets (cleanup)
    pub fn prune_triggered(&mut self) {
        for targets in self.targets.values_mut() {
            targets.retain(|t| !t.triggered);
        }
    }

    /// Returns true if all targets have been triggered
    pub fn all_triggered(&self) -> bool {
        self.targets
            .values()
            .all(|targets| targets.iter().all(|t| t.triggered))
    }
}

/// A fired target with its actual trigger price
#[derive(Debug, Clone)]
pub struct FiredTarget {
    pub target: ClawTarget,
    pub current_price: f64,
}

fn condition_label(cond: &ClawCondition) -> String {
    match cond {
        ClawCondition::Above => "≥".into(),
        ClawCondition::Below => "≤".into(),
        ClawCondition::PercentChange(p) => format!("±{}%", p),
    }
}

fn price_fmt(p: f64) -> String {
    if p < 0.01 {
        format!("{:.8}", p)
    } else if p < 1.0 {
        format!("{:.6}", p)
    } else {
        format!("{:.4}", p)
    }
}
