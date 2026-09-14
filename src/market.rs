use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::Value;
use std::time::{Duration, Instant};
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};

pub const UI_PERIOD: Duration = Duration::from_millis(100);
const HTTP_TIMEOUT: Duration = Duration::from_secs(10);
const WS_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_BACKOFF: Duration = Duration::from_secs(30);
const CANDLE_LIMIT: usize = 3;
const HOUR_MS: i64 = 3_600_000;

const REST_URL: &str =
    "https://data-api.binance.vision/api/v3/klines?symbol=SOLUSDT&interval=1h&limit=3";
const TICKER_URL: &str = "https://data-api.binance.vision/api/v3/ticker/24hr?symbol=SOLUSDT";
const WS_URL: &str =
    "wss://data-stream.binance.vision:443/stream?streams=solusdt@trade/solusdt@ticker";

#[derive(Clone, Debug, Default, PartialEq)]
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
}

#[derive(Debug, Deserialize)]
struct TickerResponse {
    #[serde(rename = "lastPrice")]
    last_price: String,
    #[serde(rename = "priceChangePercent")]
    price_change_percent: String,
}

#[derive(Debug, Deserialize)]
struct StreamEnvelope {
    data: MarketEvent,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "e")]
enum MarketEvent {
    #[serde(rename = "trade")]
    Trade {
        #[serde(rename = "p")]
        price: String,
        #[serde(rename = "T")]
        timestamp: i64,
    },
    #[serde(rename = "24hrTicker")]
    Ticker {
        #[serde(rename = "c")]
        price: String,
        #[serde(rename = "P")]
        change_percent: String,
    },
    #[serde(other)]
    Other,
}

fn parse_number(value: &str) -> Option<f64> {
    value
        .parse::<f64>()
        .ok()
        .filter(|number| number.is_finite())
}

fn number(value: &Value) -> Option<f64> {
    value
        .as_str()
        .and_then(parse_number)
        .or_else(|| value.as_f64().filter(|number| number.is_finite()))
}

async fn fetch_candles(client: &reqwest::Client) -> reqwest::Result<Vec<Candle>> {
    let rows: Vec<Vec<Value>> = client
        .get(REST_URL)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(rows
        .into_iter()
        .filter_map(|row| {
            if row.len() < 5 {
                return None;
            }

            let candle = Candle {
                open_time: row[0].as_i64()?,
                open: number(&row[1])?,
                high: number(&row[2])?,
                low: number(&row[3])?,
                close: number(&row[4])?,
            };

            (candle.open_time >= 0
                && candle.open > 0.0
                && candle.high > 0.0
                && candle.low > 0.0
                && candle.close > 0.0)
                .then_some(candle)
        })
        .collect())
}

async fn fetch_ticker(client: &reqwest::Client) -> reqwest::Result<(f64, f64)> {
    let payload: TickerResponse = client
        .get(TICKER_URL)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok((
        parse_number(&payload.last_price).unwrap_or_default(),
        parse_number(&payload.price_change_percent).unwrap_or_default(),
    ))
}

fn apply_trade(candles: &mut Vec<Candle>, price: f64, timestamp: i64) {
    if !price.is_finite() || price <= 0.0 || timestamp < 0 {
        return;
    }

    let bucket = timestamp - timestamp.rem_euclid(HOUR_MS);
    if let Some(last) = candles.last_mut()
        && last.open_time == bucket
    {
        last.high = last.high.max(price);
        last.low = last.low.min(price);
        last.close = price;
        return;
    }

    candles.push(Candle {
        open_time: bucket,
        open: price,
        high: price,
        low: price,
        close: price,
    });

    if candles.len() > CANDLE_LIMIT {
        candles.drain(..candles.len() - CANDLE_LIMIT);
    }
}

fn snapshot(price: f64, change_percent: f64, candles: &[Candle]) -> Snapshot {
    Snapshot {
        price,
        change_percent,
        candles: candles.to_vec(),
    }
}

fn run_stream() -> impl futures_util::Stream<Item = Snapshot> {
    async_stream::stream! {
        let client = reqwest::Client::builder()
            .user_agent(concat!("Ticker/", env!("CARGO_PKG_VERSION")))
            .timeout(HTTP_TIMEOUT)
            .build()
            .expect("reqwest client configuration must be valid");
        let mut backoff = Duration::from_secs(1);

        loop {
            let mut candles = fetch_candles(&client).await.unwrap_or_default();
            let (ticker_price, mut change_percent) = fetch_ticker(&client).await.unwrap_or_default();
            let mut price = if ticker_price > 0.0 {
                ticker_price
            } else {
                candles.last().map(|candle| candle.close).unwrap_or_default()
            };
            let mut last_emit = Instant::now() - UI_PERIOD;

            if price > 0.0 {
                yield snapshot(price, change_percent, &candles);
                last_emit = Instant::now();
            }

            let connection = tokio::time::timeout(WS_CONNECT_TIMEOUT, connect_async(WS_URL)).await;
            if let Ok(Ok((mut socket, _))) = connection {
                backoff = Duration::from_secs(1);

                while let Some(item) = socket.next().await {
                    match item {
                        Ok(WsMessage::Text(text)) => {
                            let Ok(envelope) = serde_json::from_str::<StreamEnvelope>(&text) else {
                                continue;
                            };

                            match envelope.data {
                                MarketEvent::Trade { price: next_price, timestamp } => {
                                    let Some(next_price) = parse_number(&next_price).filter(|value| *value > 0.0) else {
                                        continue;
                                    };
                                    price = next_price;
                                    apply_trade(&mut candles, price, timestamp);
                                }
                                MarketEvent::Ticker { price: next_price, change_percent: next_change } => {
                                    if let Some(next_price) = parse_number(&next_price).filter(|value| *value > 0.0) {
                                        price = next_price;
                                    }
                                    if let Some(next_change) = parse_number(&next_change) {
                                        change_percent = next_change;
                                    }
                                }
                                MarketEvent::Other => continue,
                            }

                            if last_emit.elapsed() >= UI_PERIOD {
                                yield snapshot(price, change_percent, &candles);
                                last_emit = Instant::now();
                            }
                        }
                        Ok(WsMessage::Ping(data)) => {
                            if socket.send(WsMessage::Pong(data)).await.is_err() {
                                break;
                            }
                        }
                        Ok(WsMessage::Close(_)) | Err(_) => break,
                        _ => {}
                    }
                }
            }

            if price > 0.0 {
                yield snapshot(price, change_percent, &candles);
            }
            tokio::time::sleep(backoff).await;
            backoff = (backoff * 2).min(MAX_BACKOFF);
        }
    }
}

pub fn subscription() -> cosmic::iced::Subscription<Snapshot> {
    cosmic::iced::Subscription::run(run_stream)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candle(open_time: i64, price: f64) -> Candle {
        Candle {
            open_time,
            open: price,
            high: price,
            low: price,
            close: price,
        }
    }

    #[test]
    fn trade_updates_current_hour() {
        let mut candles = vec![candle(0, 100.0)];
        apply_trade(&mut candles, 105.0, 1_800_000);

        assert_eq!(candles.len(), 1);
        assert_eq!(candles[0].close, 105.0);
        assert_eq!(candles[0].high, 105.0);
        assert_eq!(candles[0].low, 100.0);
    }

    #[test]
    fn trade_creates_new_hour() {
        let mut candles = vec![candle(0, 100.0)];
        apply_trade(&mut candles, 110.0, HOUR_MS);

        assert_eq!(candles.len(), 2);
        assert_eq!(candles[1].open, 110.0);
    }

    #[test]
    fn trade_history_is_bounded_to_three_candles() {
        let mut candles = vec![
            candle(0, 100.0),
            candle(HOUR_MS, 101.0),
            candle(HOUR_MS * 2, 102.0),
        ];
        apply_trade(&mut candles, 103.0, HOUR_MS * 3);

        assert_eq!(candles.len(), CANDLE_LIMIT);
        assert_eq!(candles[0].open_time, HOUR_MS);
        assert_eq!(candles[2].open_time, HOUR_MS * 3);
    }

    #[test]
    fn invalid_trade_is_ignored() {
        let original = vec![candle(0, 100.0)];
        let mut candles = original.clone();

        apply_trade(&mut candles, f64::NAN, HOUR_MS);
        apply_trade(&mut candles, 0.0, HOUR_MS);
        apply_trade(&mut candles, 100.0, -1);

        assert_eq!(candles, original);
    }

    #[test]
    fn parses_finite_market_numbers_only() {
        assert_eq!(parse_number("123.45"), Some(123.45));
        assert_eq!(parse_number("NaN"), None);
        assert_eq!(parse_number("inf"), None);
        assert_eq!(parse_number("invalid"), None);
    }
}
