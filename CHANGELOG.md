# Changelog

All notable changes to Binance-Claw will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [1.0.0] - 2025-01-15

### Added
- Real-time WebSocket price streaming via Binance combined streams
- `price` command — fetch current price for any trading pair
- `watch` command — live price stream with color direction arrows
- `claw` command — set price alerts (above / below / percent change)
- `stats` command — 24h ticker statistics (high, low, volume, change)
- `book` command — live order book depth viewer
- `symbols` command — list all available trading pairs with quote filter
- `ping` command — connectivity check with latency measurement
- `skill` command — OpenClaw skill manifest output
- `config` command — view and manage configuration
- Cross-platform desktop notifications (Linux, macOS, Windows)
- Webhook support — POST alert payload to any URL on claw trigger
- OpenClaw skill integration with JSON output format
- One-command install scripts for Linux, macOS, and Windows
- GitHub Actions CI/CD with multi-platform release builds
- TOML configuration with environment variable overrides
- Automatic WebSocket reconnection with configurable retry limit
- REST API polling fallback for environments without WebSocket
