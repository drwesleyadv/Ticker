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
- no API key and no account required;
- designed to run as a COSMIC Panel applet, not as a standalone window.

The displayed market is **SOL/USDT**. USDT is used as the dollar proxy; it is not a direct fiat USD feed.

The percentage is Binance's **rolling 24-hour change**, not the change since 00:00 UTC or local midnight.

## Distribution

The project is prepared for packaging in the official COSMIC Flatpak repository used by the COSMIC Store.

Official COSMIC Flatpak repository: https://github.com/pop-os/cosmic-flatpak

## Development

```bash
cargo fmt --check
cargo test
cargo build --release
```

## License

MIT
