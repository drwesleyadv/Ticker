# Solana Ticker for COSMIC

A small native COSMIC panel applet that displays the live SOL/USDT price and the latest three one-hour candles.

## Features

- real-time SOL/USDT trades from Binance;
- local reconstruction of the current one-hour OHLC candle;
- three-candle dynamic vector icon;
- UI refresh capped at 10 Hz;
- automatic reconnect with exponential backoff;
- REST resynchronization after reconnect;
- no API key and no account required;
- designed to run as a COSMIC Panel applet, not as a standalone window.

The displayed market is **SOL/USDT**. USDT is used as the dollar proxy; it is not a direct fiat USD feed.

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
