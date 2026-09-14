# Solana Ticker for COSMIC

A small native COSMIC panel applet that displays the live SOL/USDT price, the latest three one-hour candles, and Binance's rolling 24-hour percentage change.

## Features

- real-time SOL/USDT trades from Binance;
- Binance rolling 24h price-change percentage;
- green up arrow for positive/zero change and red down arrow for negative change;
- local reconstruction of the current one-hour OHLC candle;
- three-candle dynamic vector icon;
- UI refresh capped at 10 Hz;
- automatic reconnect with exponential backoff;
- REST resynchronization after reconnect;
- finite-value validation for market payloads;
- HTTP and WebSocket connection timeouts;
- no API key and no account required;
- designed to run as a COSMIC Panel applet, not as a standalone window.

The displayed market is **SOL/USDT**. USDT is used as the dollar proxy; it is not a direct fiat USD feed.

The percentage is Binance's **rolling 24-hour change**, not the change since 00:00 UTC or local midnight.

## Development environment

The project targets **Rust 1.95.0 or newer**, edition 2024, on `x86_64-unknown-linux-gnu`. It is compatible with a system-wide Rust installation such as the Pop!_OS/Ubuntu packages (`/usr/bin/rustc` and `/usr/bin/cargo`); `rustup` is not required to build the project.

On Ubuntu/Pop!_OS 24.04, the native build dependencies are:

```bash
sudo apt install build-essential cmake pkg-config libssl-dev libxkbcommon-dev
```

Core validation, which works without `rustfmt` or Clippy installed locally:

```bash
cargo test --locked
cargo build --release --locked
```

Or, with `just`:

```bash
just check
```

Optional quality checks are automatically skipped when their components are unavailable:

```bash
just quality
```

CI uses Rust 1.95.0 with `rustfmt` and Clippy and treats Clippy warnings as errors.

## Distribution

The project is prepared for packaging in the official COSMIC Flatpak repository used by the COSMIC Store.

Official COSMIC Flatpak repository: https://github.com/pop-os/cosmic-flatpak

## License

MIT
