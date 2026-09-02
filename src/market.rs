use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use std::time::{Duration, Instant};
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};

pub const UI_PERIOD: Duration = Duration::from_millis(100);
const REST_URL: &str = "https://data-api.binance.vision/api/v3/klines?symbol=SOLUSDT&interval=1h&limit=3";
const TICKER_URL: &str = "https://data-api.binance.vision/api/v3/ticker/24hr?symbol=SOLUSDT";
const WS_URL: &str = "wss://data-stream.binance.vision:443/stream?streams=solusdt@trade/solusdt@ticker";

#[derive(Clone, Debug, Default)]
pub struct Candle {
    pub open_time: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Clone, Debug, Default)]
pub struct Snapshot {
    pub price: f64,
    pub change_percent: f64,
    pub candles: Vec<Candle>,
    pub connected: bool,
}

fn number(v: &Value) -> f64 {
    v.as_str()
        .and_then(|s| s.parse().ok())
        .or_else(|| v.as_f64())
        .unwrap_or(0.0)
}

async fn fetch_candles(client: &reqwest::Client) -> Result<Vec<Candle>> {
    let rows: Vec<Vec<Value>> = client
        .get(REST_URL)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(rows
        .into_iter()
        .filter(|r| r.len() >= 5)
        .map(|r| Candle {
            open_time: r[0].as_i64().unwrap_or(0),
            open: number(&r[1]),
            high: number(&r[2]),
            low: number(&r[3]),
            close: number(&r[4]),
        })
        .collect())
}

async fn fetch_change(client: &reqwest::Client) -> Result<(f64, f64)> {
    let payload: Value = client
        .get(TICKER_URL)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok((
        number(payload.get("lastPrice").unwrap_or(&Value::Null)),
        number(payload.get("priceChangePercent").unwrap_or(&Value::Null)),
    ))
}

fn apply_trade(candles: &mut Vec<Candle>, price: f64, timestamp: i64) {
    if price <= 0.0 {
        return;
    }
    let bucket = timestamp - timestamp.rem_euclid(3_600_000);
    if let Some(last) = candles.last_mut() {
        if last.open_time == bucket {
            last.high = last.high.max(price);
            last.low = last.low.min(price);
            last.close = price;
            return;
        }
    }
    candles.push(Candle {
        open_time: bucket,
        open: price,
        high: price,
        low: price,
        close: price,
    });
    if candles.len() > 3 {
        candles.drain(0..candles.len() - 3);
    }
}

fn run_stream() -> impl futures_util::Stream<Item = Snapshot> {
    async_stream::stream! {
        let client = reqwest::Client::builder()
            .user_agent("Ticker/1.1")
            .build()
            .expect("valid reqwest client");
        let mut backoff = Duration::from_secs(1);

        loop {
            let mut candles = fetch_candles(&client).await.unwrap_or_default();
            let (ticker_price, mut change_percent) = fetch_change(&client).await.unwrap_or((0.0, 0.0));
            let mut price = if ticker_price > 0.0 {
                ticker_price
            } else {
                candles.last().map(|c| c.close).unwrap_or_default()
            };
            let mut last_emit = Instant::now() - UI_PERIOD;

            if price > 0.0 {
                yield Snapshot { price, change_percent, candles: candles.clone(), connected: false };
                last_emit = Instant::now();
            }

            match connect_async(WS_URL).await {
                Ok((mut socket, _)) => {
                    backoff = Duration::from_secs(1);
                    yield Snapshot { price, change_percent, candles: candles.clone(), connected: true };
                    last_emit = Instant::now();

                    while let Some(item) = socket.next().await {
                        match item {
                            Ok(WsMessage::Text(text)) => {
                                if let Ok(envelope) = serde_json::from_str::<Value>(&text) {
                                    let payload = envelope.get("data").unwrap_or(&envelope);
                                    match payload.get("e").and_then(Value::as_str) {
                                        Some("trade") => {
                                            let next_price = number(payload.get("p").unwrap_or(&Value::Null));
                                            let timestamp = payload.get("T").and_then(Value::as_i64).unwrap_or(0);
                                            if next_price > 0.0 {
                                                price = next_price;
                                                apply_trade(&mut candles, price, timestamp);
                                                if last_emit.elapsed() >= UI_PERIOD {
                                                    yield Snapshot { price, change_percent, candles: candles.clone(), connected: true };
                                                    last_emit = Instant::now();
                                                }
                                            }
                                        }
                                        Some("24hrTicker") => {
                                            let next_price = number(payload.get("c").unwrap_or(&Value::Null));
                                            let next_change = number(payload.get("P").unwrap_or(&Value::Null));
                                            if next_price > 0.0 {
                                                price = next_price;
                                            }
                                            change_percent = next_change;
                                            if last_emit.elapsed() >= UI_PERIOD {
                                                yield Snapshot { price, change_percent, candles: candles.clone(), connected: true };
                                                last_emit = Instant::now();
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            Ok(WsMessage::Ping(data)) => {
                                let _ = socket.send(WsMessage::Pong(data)).await;
                            }
                            Ok(WsMessage::Close(_)) | Err(_) => break,
                            _ => {}
                        }
                    }
                }
                Err(_) => {}
            }

            yield Snapshot { price, change_percent, candles: candles.clone(), connected: false };
            tokio::time::sleep(backoff).await;
            backoff = (backoff * 2).min(Duration::from_secs(30));
        }
    }
}

pub fn subscription() -> cosmic::iced::Subscription<Snapshot> {
    cosmic::iced::Subscription::run(run_stream)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trade_updates_current_hour() {
        let mut candles = vec![Candle {
            open_time: 0,
            open: 100.0,
            high: 100.0,
            low: 100.0,
            close: 100.0,
        }];
        apply_trade(&mut candles, 105.0, 1_800_000);
        assert_eq!(candles.len(), 1);
        assert_eq!(candles[0].close, 105.0);
        assert_eq!(candles[0].high, 105.0);
    }

    #[test]
    fn trade_creates_new_hour() {
        let mut candles = vec![Candle {
            open_time: 0,
            open: 100.0,
            high: 100.0,
            low: 100.0,
            close: 100.0,
        }];
        apply_trade(&mut candles, 110.0, 3_600_000);
        assert_eq!(candles.len(), 2);
        assert_eq!(candles[1].open, 110.0);
    }
}
