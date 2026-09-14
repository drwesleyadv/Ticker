use serde::Deserialize;
use serde_json::Value;

const HOUR_MS: i64 = 3_600_000;
const MAX_CANDLES: usize = 3;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Candle {
    pub open_time: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

impl Candle {
    pub(super) fn from_kline_row(row: &[Value]) -> Option<Self> {
        if row.len() < 5 {
            return None;
        }

        let candle = Self {
            open_time: row[0].as_i64()?,
            open: json_number(&row[1])?,
            high: json_number(&row[2])?,
            low: json_number(&row[3])?,
            close: json_number(&row[4])?,
        };

        candle.is_valid().then_some(candle)
    }

    fn is_valid(&self) -> bool {
        self.open_time >= 0
            && self.open.is_finite()
            && self.high.is_finite()
            && self.low.is_finite()
            && self.close.is_finite()
            && self.open > 0.0
            && self.high > 0.0
            && self.low > 0.0
            && self.close > 0.0
            && self.high >= self.low
    }

    fn apply_price(&mut self, price: f64) {
        self.high = self.high.max(price);
        self.low = self.low.min(price);
        self.close = price;
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Snapshot {
    pub price: f64,
    pub change_percent: f64,
    pub candles: Vec<Candle>,
    pub connected: bool,
}

#[derive(Debug, Default)]
pub(super) struct MarketState {
    price: f64,
    change_percent: f64,
    candles: Vec<Candle>,
    connected: bool,
}

impl MarketState {
    pub(super) fn snapshot(&self) -> Snapshot {
        Snapshot {
            price: self.price,
            change_percent: self.change_percent,
            candles: self.candles.clone(),
            connected: self.connected,
        }
    }

    pub(super) fn has_price(&self) -> bool {
        self.price > 0.0 && self.price.is_finite()
    }

    pub(super) fn replace_candles(&mut self, mut candles: Vec<Candle>) {
        candles.sort_unstable_by_key(|candle| candle.open_time);
        if candles.len() > MAX_CANDLES {
            candles.drain(..candles.len() - MAX_CANDLES);
        }

        if !self.has_price() {
            self.price = candles.last().map(|candle| candle.close).unwrap_or_default();
        }
        self.candles = candles;
    }

    pub(super) fn apply_ticker(&mut self, price: f64, change_percent: f64) {
        if price.is_finite() && price > 0.0 {
            self.price = price;
        }
        if change_percent.is_finite() {
            self.change_percent = change_percent;
        }
    }

    pub(super) fn apply_trade(&mut self, price: f64, timestamp: i64) {
        if !price.is_finite() || price <= 0.0 || timestamp < 0 {
            return;
        }

        self.price = price;
        let bucket = timestamp - timestamp.rem_euclid(HOUR_MS);

        if let Some(candle) = self
            .candles
            .iter_mut()
            .find(|candle| candle.open_time == bucket)
        {
            candle.apply_price(price);
            return;
        }

        if self
            .candles
            .last()
            .is_some_and(|candle| bucket < candle.open_time)
        {
            return;
        }

        self.candles.push(Candle {
            open_time: bucket,
            open: price,
            high: price,
            low: price,
            close: price,
        });

        if self.candles.len() > MAX_CANDLES {
            self.candles.drain(..self.candles.len() - MAX_CANDLES);
        }
    }

    pub(super) fn set_connected(&mut self, connected: bool) {
        self.connected = connected;
    }
}

pub(super) fn json_number(value: &Value) -> Option<f64> {
    let number = value
        .as_str()
        .and_then(|text| text.parse::<f64>().ok())
        .or_else(|| value.as_f64())?;

    number.is_finite().then_some(number)
}

#[derive(Deserialize)]
#[serde(untagged)]
enum WireNumber {
    Text(String),
    Number(f64),
}

pub(super) fn deserialize_number<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    let value = WireNumber::deserialize(deserializer)?;
    let number = match value {
        WireNumber::Text(text) => text.parse::<f64>().map_err(D::Error::custom)?,
        WireNumber::Number(number) => number,
    };

    if number.is_finite() {
        Ok(number)
    } else {
        Err(D::Error::custom("market number is not finite"))
    }
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
        let mut state = MarketState::default();
        state.replace_candles(vec![candle(0, 100.0)]);

        state.apply_trade(105.0, 1_800_000);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.candles.len(), 1);
        assert_eq!(snapshot.candles[0].close, 105.0);
        assert_eq!(snapshot.candles[0].high, 105.0);
        assert_eq!(snapshot.price, 105.0);
    }

    #[test]
    fn trade_creates_new_hour_and_keeps_three_candles() {
        let mut state = MarketState::default();
        state.replace_candles(vec![
            candle(0, 90.0),
            candle(HOUR_MS, 95.0),
            candle(HOUR_MS * 2, 100.0),
        ]);

        state.apply_trade(110.0, HOUR_MS * 3);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.candles.len(), 3);
        assert_eq!(snapshot.candles[0].open_time, HOUR_MS);
        assert_eq!(snapshot.candles[2].open, 110.0);
    }

    #[test]
    fn stale_trade_does_not_reorder_candles() {
        let mut state = MarketState::default();
        state.replace_candles(vec![candle(HOUR_MS, 100.0), candle(HOUR_MS * 2, 110.0)]);

        state.apply_trade(80.0, 0);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.candles.len(), 2);
        assert_eq!(snapshot.candles[0].open_time, HOUR_MS);
        assert_eq!(snapshot.price, 80.0);
    }

    #[test]
    fn invalid_trade_is_ignored() {
        let mut state = MarketState::default();
        state.replace_candles(vec![candle(0, 100.0)]);

        state.apply_trade(f64::NAN, HOUR_MS);
        state.apply_trade(0.0, HOUR_MS);
        state.apply_trade(120.0, -1);

        assert_eq!(state.snapshot().candles, vec![candle(0, 100.0)]);
    }
}
