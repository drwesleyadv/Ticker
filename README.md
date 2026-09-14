# Solana Ticker for COSMIC

A minimal native COSMIC panel applet for SOL/USDT. The panel shows only the original three-candle icon and the current quote by default.

Click the ticker to open its preferences.

## Preferences

- quote refresh interval in milliseconds (100–60000 ms);
- candle timeframe: 1m, 5m, 15m, 30m, 1h, 4h or 1d;
- optional Binance rolling 24-hour percentage change next to the quote;
- **Save** applies and persists the draft;
- **Discard** closes the popup without applying changes.

Preferences are stored under the user's XDG configuration directory (`$XDG_CONFIG_HOME/com.github.drwesleyadv.Ticker/settings.json`, or `~/.config/com.github.drwesleyadv.Ticker/settings.json`).

## Market data

Each refresh reads Binance public market data for **SOL/USDT** and the latest three candles for the selected timeframe. No API key or account is required. USDT is used as the dollar proxy; it is not a direct fiat USD feed.

The optional percentage is Binance's rolling 24-hour price change.

## Development environment

The project targets **Rust 1.98.1 / Cargo 1.98.1**, edition 2024, on `x86_64-unknown-linux-gnu`. `libcosmic` is pinned to a known current commit so builds remain reproducible.

On Ubuntu/Pop!_OS 24.04, install the native build dependencies with:

```bash
sudo apt install build-essential cmake pkg-config libssl-dev libxkbcommon-dev
```

Then validate and build:

```bash
cargo test --locked
cargo build --release --locked
```

CI additionally runs `cargo fmt --check` and Clippy with warnings denied.

## Manual installation

A validated `Ticker-manual-install` artifact is generated from successful pushes to `main`. It includes the complete source tree, the release binary and `install.sh`.

## License

MIT
