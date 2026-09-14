use crate::market::Candle;

const UP_COLOR: &str = "#10B981";
const DOWN_COLOR: &str = "#F05252";
const EMPTY_COLOR: &str = "#69707D";

fn candle_color(candle: &Candle) -> &'static str {
    if candle.close >= candle.open {
        UP_COLOR
    } else {
        DOWN_COLOR
    }
}

pub fn render(candles: &[Candle]) -> String {
    let start = candles.len().saturating_sub(3);
    let visible = &candles[start..];

    if visible.is_empty() {
        return format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"><rect x="8" y="8" width="48" height="48" rx="14" fill="{EMPTY_COLOR}"/></svg>"#
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
    let y = |price: f64| 8.0 + (high - price) / range * 48.0;
    let xs = [12.0, 32.0, 52.0];
    let width = 10.0;

    let mut body = String::new();
    for (index, candle) in visible.iter().enumerate() {
        let x = xs[index];
        let color = candle_color(candle);
        let top = y(candle.open.max(candle.close));
        let bottom = y(candle.open.min(candle.close));
        let body_height = (bottom - top).max(5.0);
        body.push_str(&format!(
            r#"<line x1="{x:.1}" y1="{high_y:.1}" x2="{x:.1}" y2="{low_y:.1}" stroke="{color}" stroke-width="3" stroke-linecap="round"/><rect x="{left:.1}" y="{top:.1}" width="{width:.1}" height="{body_height:.1}" rx="3" fill="{color}"/>"#,
            high_y = y(candle.high),
            low_y = y(candle.low),
            left = x - width / 2.0,
        ));
    }

    format!(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64">{body}</svg>"#)
}

pub fn direction(change_percent: f64) -> String {
    let (color, points) = if change_percent >= 0.0 {
        (UP_COLOR, "32,7 52,31 40,31 40,57 24,57 24,31 12,31")
    } else {
        (DOWN_COLOR, "24,7 40,7 40,33 52,33 32,57 12,33 24,33")
    };

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"><polygon points="{points}" fill="{color}"/></svg>"#
    )
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
    fn empty_render_uses_neutral_icon() {
        let svg = render(&[]);
        assert!(svg.contains(EMPTY_COLOR));
    }

    #[test]
    fn render_uses_only_last_three_candles() {
        let candles = vec![
            candle(1.0, 2.0, 1.0, 2.0),
            candle(2.0, 3.0, 2.0, 3.0),
            candle(3.0, 4.0, 3.0, 4.0),
            candle(4.0, 5.0, 4.0, 5.0),
        ];
        let svg = render(&candles);

        assert_eq!(svg.matches("<rect").count(), 3);
    }

    #[test]
    fn direction_matches_change_sign() {
        assert!(direction(0.0).contains(UP_COLOR));
        assert!(direction(-0.01).contains(DOWN_COLOR));
    }
}
