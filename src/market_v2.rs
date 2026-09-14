use crate::settings::Settings;
use serde::Deserialize;
use serde_json::Value;
use std::time::Duration;

const API_ROOT: &str = "https://data-api.binance.vision/api/v3";

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

fn finite(value: &str) -> Option<f64> {
    value.parse::<f64>().ok().filter(|number| number.is_finite())
}

fn json_number(value: &Value) -> Option<f64> {
    value
        .as_str()
        .and_then(finite)
        .or_else(|| value.as_f64().filter(|number| number.is_finite()))
}

async fn fetch(client: &reqwest::Client, settings: Settings) -> Option<Snapshot> {
    let ticker_url = format!("{API_ROOT}/ticker/24hr");
    let ticker: TickerResponse = client
        .get(ticker_url)
        .query(&[("symbol", "SOLUSDT")])
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?
        .json()
        .await
        .ok()?;

    let klines_url = format!("{API_ROOT}/klines");
    let rows: Vec<Vec<Value>> = client
        .get(klines_url)
        .query(&[
            ("symbol", "SOLUSDT"),
            ("interval", settings.timeframe.as_binance()),
            ("limit", "3"),
        ])
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?
        .json()
        .await
        .ok()?;

    let candles = rows
        .into_iter()
        .filter_map(|row| {
            if row.len() < 5 {
                return None;
            }
            Some(Candle {
                open_time: row[0].as_i64()?,
                open: json_number(&row[1])?,
                high: json_number(&row[2])?,
                low: json_number(&row[3])?,
                close: json_number(&row[4])?,
            })
        })
        .collect::<Vec<_>>();

    let price = finite(&ticker.last_price)?;
    let change_percent = finite(&ticker.price_change_percent).unwrap_or_default();
    (price > 0.0 && candles.len() == 3).then_some(Snapshot {
        price,
        change_percent,
        candles,
    })
}

fn stream(settings: Settings) -> impl cosmic::iced::futures::Stream<Item = Snapshot> {
    async_stream::stream! {
        let client = reqwest::Client::builder()
            .user_agent(concat!("Ticker/", env!("CARGO_PKG_VERSION")))
            .timeout(Duration::from_secs(10))
            .build()
            .expect("valid HTTP client configuration");
        let period = Duration::from_millis(settings.update_interval_ms);

        loop {
            if let Some(snapshot) = fetch(&client, settings).await {
                yield snapshot;
            }
            tokio::time::sleep(period).await;
        }
    }
}

pub fn subscription(settings: Settings) -> cosmic::iced::Subscription<Snapshot> {
    cosmic::iced::Subscription::run_with(settings, move |settings| stream(*settings))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_finite_numbers() {
        assert_eq!(finite("123.45"), Some(123.45));
        assert_eq!(finite("NaN"), None);
        assert_eq!(finite("inf"), None);
    }
}
