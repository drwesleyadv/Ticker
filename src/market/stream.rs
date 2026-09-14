use super::api::MarketApi;
use super::model::{MarketState, Snapshot, deserialize_number};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use std::time::{Duration, Instant};
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};

pub(super) const UI_PERIOD: Duration = Duration::from_millis(100);
const WS_URL: &str =
    "wss://data-stream.binance.vision:443/stream?streams=solusdt@trade/solusdt@ticker";
const WS_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const INITIAL_BACKOFF: Duration = Duration::from_secs(1);
const MAX_BACKOFF: Duration = Duration::from_secs(30);

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum IncomingMessage {
    Combined { data: MarketEvent },
    Direct(MarketEvent),
}

impl IncomingMessage {
    fn into_event(self) -> MarketEvent {
        match self {
            Self::Combined { data } => data,
            Self::Direct(event) => event,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "e")]
enum MarketEvent {
    #[serde(rename = "trade")]
    Trade {
        #[serde(rename = "p", deserialize_with = "deserialize_number")]
        price: f64,
        #[serde(rename = "T")]
        timestamp: i64,
    },
    #[serde(rename = "24hrTicker")]
    Ticker24h {
        #[serde(rename = "c", deserialize_with = "deserialize_number")]
        price: f64,
        #[serde(rename = "P", deserialize_with = "deserialize_number")]
        change_percent: f64,
    },
    #[serde(other)]
    Other,
}

fn parse_event(text: &str) -> Option<MarketEvent> {
    serde_json::from_str::<IncomingMessage>(text)
        .ok()
        .map(IncomingMessage::into_event)
}

fn should_emit(last_emit: &mut Instant) -> bool {
    if last_emit.elapsed() < UI_PERIOD {
        return false;
    }

    *last_emit = Instant::now();
    true
}

pub(super) fn run() -> impl futures_util::Stream<Item = Snapshot> {
    async_stream::stream! {
        let api = match MarketApi::new() {
            Ok(api) => api,
            Err(_) => return,
        };
        let mut state = MarketState::default();
        let mut backoff = INITIAL_BACKOFF;

        loop {
            if let Ok(candles) = api.fetch_candles().await {
                state.replace_candles(candles);
            }
            if let Ok(ticker) = api.fetch_ticker().await {
                state.apply_ticker(ticker.price, ticker.change_percent);
            }

            state.set_connected(false);
            if state.has_price() {
                yield state.snapshot();
            }

            let connection = tokio::time::timeout(WS_CONNECT_TIMEOUT, connect_async(WS_URL)).await;
            if let Ok(Ok((mut socket, _response))) = connection {
                backoff = INITIAL_BACKOFF;
                state.set_connected(true);
                yield state.snapshot();
                let mut last_emit = Instant::now();

                while let Some(message) = socket.next().await {
                    match message {
                        Ok(WsMessage::Text(text)) => {
                            match parse_event(text.as_str()) {
                                Some(MarketEvent::Trade { price, timestamp }) => {
                                    state.apply_trade(price, timestamp);
                                }
                                Some(MarketEvent::Ticker24h {
                                    price,
                                    change_percent,
                                }) => {
                                    state.apply_ticker(price, change_percent);
                                }
                                Some(MarketEvent::Other) | None => continue,
                            }

                            if should_emit(&mut last_emit) {
                                yield state.snapshot();
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

            state.set_connected(false);
            if state.has_price() {
                yield state.snapshot();
            }

            tokio::time::sleep(backoff).await;
            backoff = backoff.saturating_mul(2).min(MAX_BACKOFF);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_combined_trade_event() {
        let event = parse_event(
            r#"{"stream":"solusdt@trade","data":{"e":"trade","p":"123.45","T":3600000}}"#,
        )
        .expect("trade event");

        match event {
            MarketEvent::Trade { price, timestamp } => {
                assert_eq!(price, 123.45);
                assert_eq!(timestamp, 3_600_000);
            }
            _ => panic!("unexpected event"),
        }
    }

    #[test]
    fn parses_combined_ticker_event() {
        let event = parse_event(
            r#"{"stream":"solusdt@ticker","data":{"e":"24hrTicker","c":"130.25","P":"2.50"}}"#,
        )
        .expect("ticker event");

        match event {
            MarketEvent::Ticker24h {
                price,
                change_percent,
            } => {
                assert_eq!(price, 130.25);
                assert_eq!(change_percent, 2.5);
            }
            _ => panic!("unexpected event"),
        }
    }

    #[test]
    fn ignores_unknown_market_event() {
        let event = parse_event(r#"{"data":{"e":"depthUpdate"}}"#).expect("known envelope");
        assert!(matches!(event, MarketEvent::Other));
    }
}
