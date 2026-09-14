use crate::market::Candle;

const UP_COLOR: &str = "#10B981";
const DOWN_COLOR: &str = "#F05252";
const EMPTY_COLOR: &str = "#69707D";
const VIEWBOX_SIZE: f64 = 64.0;
const CHART_TOP: f64 = 8.0;
const CHART_HEIGHT: f64 = 48.0;
const CANDLE_X: [f64; 3] = [12.0, 32.0, 52.0];
const CANDLE_WIDTH: f64 = 10.0;
const MIN_BODY_HEIGHT: f64 = 5.0;

pub fn render(candles: &[Candle]) -> String {
    let start = candles.len().saturating_sub(3);
    let visible = &candles[start..];

    if visible.is_empty() {
        return format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {VIEWBOX_SIZE:.0} {VIEWBOX_SIZE:.0}"><rect x="8" y="8" width="48" height="48" rx="14" fill="{EMPTY_COLOR}"/></svg>"#
        );
    }

    let low = visible
        .iter()
        .map(|candle| candle.low)
        .fold(f64::INFINITY, f64::min);
    let high = visible
        .iter()
        .map(|candle| candle.high)
        .fold(f64::NEG_INFINITY, f64::max);
    let range = (high - low).max(f64::EPSILON);
    let y = |price: f64| CHART_TOP + (high - price) / range * CHART_HEIGHT;

    let mut body = String::with_capacity(512);
    for (index, candle) in visible.iter().enumerate() {
        let x = CANDLE_X.get(index).copied().unwrap_or(CANDLE_X[1]);
        let color = candle_color(candle);
        let top = y(candle.open.max(candle.close));
        let bottom = y(candle.open.min(candle.close));
        let body_height = (bottom - top).max(MIN_BODY_HEIGHT);
        let left = x - CANDLE_WIDTH / 2.0;

        body.push_str(&format!(
            r#"<line x1="{x:.1}" y1="{high_y:.1}" x2="{x:.1}" y2="{low_y:.1}" stroke="{color}" stroke-width="3" stroke-linecap="round"/><rect x="{left:.1}" y="{top:.1}" width="{CANDLE_WIDTH:.1}" height="{body_height:.1}" rx="3" fill="{color}"/>"#,
            high_y = y(candle.high),
            low_y = y(candle.low),
        ));
    }

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {VIEWBOX_SIZE:.0} {VIEWBOX_SIZE:.0}">{body}</svg>"#
    )
}

pub fn direction(change_percent: f64) -> String {
    let (color, points) = if change_percent >= 0.0 {
        (UP_COLOR, "32,7 52,31 40,31 40,57 24,57 24,31 12,31")
    } else {
        (DOWN_COLOR, "24,7 40,7 40,33 52,33 32,57 12,33 24,33")
    };

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {VIEWBOX_SIZE:.0} {VIEWBOX_SIZE:.0}"><polygon points="{points}" fill="{color}"/></svg>"#
    )
}

fn candle_color(candle: &Candle) -> &'static str {
    if candle.close >= candle.open {
        UP_COLOR
    } else {
        DOWN_COLOR
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candle(open: f64, high: f64, low: f64, close: f64) -> Candle {
        Candle {
            open_time: 0,
            open,
            high,
            low,
            close,
        }
    }

    #[test]
    fn renders_placeholder_without_market_data() {
        let svg = render(&[]);
        assert!(svg.contains(EMPTY_COLOR));
    }

    #[test]
    fn renders_only_the_latest_three_candles() {
        let candles = vec![
            candle(1.0, 2.0, 0.5, 1.5),
            candle(2.0, 3.0, 1.0, 2.5),
            candle(3.0, 4.0, 2.0, 3.5),
            candle(4.0, 5.0, 3.0, 4.5),
        ];

        let svg = render(&candles);
        assert_eq!(svg.matches("<rect ").count(), 3);
    }

    #[test]
    fn direction_matches_change_sign() {
        assert!(direction(0.0).contains(UP_COLOR));
        assert!(direction(1.0).contains(UP_COLOR));
        assert!(direction(-1.0).contains(DOWN_COLOR));
    }
}
