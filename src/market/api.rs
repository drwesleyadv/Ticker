use super::model::{Candle, deserialize_number};
use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;
use std::time::Duration;

const CANDLES_URL: &str =
    "https://data-api.binance.vision/api/v3/klines?symbol=SOLUSDT&interval=1h&limit=3";
const TICKER_URL: &str =
    "https://data-api.binance.vision/api/v3/ticker/24hr?symbol=SOLUSDT";
const HTTP_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const HTTP_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Copy)]
pub(super) struct Ticker24h {
    pub price: f64,
    pub change_percent: f64,
}

#[derive(Debug, Deserialize)]
struct TickerResponse {
    #[serde(rename = "lastPrice", deserialize_with = "deserialize_number")]
    last_price: f64,
    #[serde(
        rename = "priceChangePercent",
        deserialize_with = "deserialize_number"
    )]
    price_change_percent: f64,
}

pub(super) struct MarketApi {
    client: reqwest::Client,
}

impl MarketApi {
    pub(super) fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent(concat!("Ticker/", env!("CARGO_PKG_VERSION")))
            .connect_timeout(HTTP_CONNECT_TIMEOUT)
            .timeout(HTTP_REQUEST_TIMEOUT)
            .build()
            .context("failed to build market HTTP client")?;

        Ok(Self { client })
    }

    pub(super) async fn fetch_candles(&self) -> Result<Vec<Candle>> {
        let rows: Vec<Vec<Value>> = self
            .client
            .get(CANDLES_URL)
            .send()
            .await
            .context("failed to request Binance candles")?
            .error_for_status()
            .context("Binance candles request returned an error status")?
            .json()
            .await
            .context("failed to decode Binance candles")?;

        let mut candles = rows
            .iter()
            .filter_map(|row| Candle::from_kline_row(row))
            .collect::<Vec<_>>();
        candles.sort_unstable_by_key(|candle| candle.open_time);

        Ok(candles)
    }

    pub(super) async fn fetch_ticker(&self) -> Result<Ticker24h> {
        let payload: TickerResponse = self
            .client
            .get(TICKER_URL)
            .send()
            .await
            .context("failed to request Binance 24h ticker")?
            .error_for_status()
            .context("Binance 24h ticker request returned an error status")?
            .json()
            .await
            .context("failed to decode Binance 24h ticker")?;

        Ok(Ticker24h {
            price: payload.last_price,
            change_percent: payload.price_change_percent,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ticker_response_accepts_binance_string_numbers() {
        let response: TickerResponse = serde_json::from_str(
            r#"{"lastPrice":"123.45","priceChangePercent":"-1.25"}"#,
        )
        .expect("valid fixture");

        assert_eq!(response.last_price, 123.45);
        assert_eq!(response.price_change_percent, -1.25);
    }

    #[test]
    fn ticker_response_rejects_non_finite_numbers() {
        let response = serde_json::from_str::<TickerResponse>(
            r#"{"lastPrice":"NaN","priceChangePercent":"1.0"}"#,
        );

        assert!(response.is_err());
    }
}
