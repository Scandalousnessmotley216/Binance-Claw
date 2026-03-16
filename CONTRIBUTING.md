# Contributing to Binance-Claw

Thank you for considering contributing! Here's how to get started.

## Development setup

```bash
git clone https://github.com/deepcon3/Binance-Claw.git
cd Binance-Claw

# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build
cargo build

# Run tests
cargo test

# Lint
cargo clippy --all-targets --all-features -- -D warnings

# Format
cargo fmt
```

## Submitting changes

1. Fork the repository
2. Create a branch: `git checkout -b feature/my-feature`
3. Make your changes and add tests
4. Run `cargo test && cargo clippy && cargo fmt`
5. Commit: `git commit -m "feat: add my feature"`
6. Push and open a Pull Request

## Commit conventions

Use [Conventional Commits](https://www.conventionalcommits.org/):
- `feat:` — new feature
- `fix:` — bug fix
- `docs:` — documentation only
- `chore:` — maintenance

## Issues

Please use GitHub Issues to report bugs or request features. Include:
- OS and version
- `binance-claw --version` output
- Steps to reproduce
- Expected vs actual behaviour
