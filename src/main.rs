mod api;
mod cli;
mod config;
mod claw;
mod monitor;
mod notify;
mod skill;
mod types;
mod utils;
#[cfg(test)]
mod tests;

use anyhow::Result;
use cli::Cli;
use clap::Parser;
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    fmt()
        .with_env_filter(
            EnvFilter::try_from_env("BINANCE_CLAW_LOG")
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .compact()
        .init();

    let cli = Cli::parse();
    cli.run().await
}
