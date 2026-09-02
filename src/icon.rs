use crate::market::Candle;

fn candle_color(candle: &Candle) -> &'static str {
    if candle.close >= candle.open {
        "#10B981"
    } else {
        "#F05252"
    }
}

pub fn render(candles: &[Candle]) -> String {
    let visible = candles.iter().rev().take(3).collect::<Vec<_>>();
    let mut ordered = visible.into_iter().rev().collect::<Vec<_>>();
    if ordered.is_empty() {
        return r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"><rect x="8" y="8" width="48" height="48" rx="14" fill="#69707D"/></svg>"##.to_string();
    }

    let low = ordered
        .iter()
        .map(|c| c.low)
        .fold(f64::INFINITY, f64::min);
    let high = ordered
        .iter()
        .map(|c| c.high)
        .fold(f64::NEG_INFINITY, f64::max);
    let range = (high - low).max(1e-9);
    let y = |price: f64| 8.0 + (high - price) / range * 48.0;
    let xs = [12.0, 32.0, 52.0];
    let width = 10.0;

    let mut body = String::new();
    for (i, candle) in ordered.drain(..).enumerate() {
        let x = xs.get(i).copied().unwrap_or(32.0);
        let color = candle_color(candle);
        let top = y(candle.open.max(candle.close));
        let bottom = y(candle.open.min(candle.close));
        let body_height = (bottom - top).max(5.0);
        body.push_str(&format!(
            r##"<line x1="{x:.1}" y1="{high_y:.1}" x2="{x:.1}" y2="{low_y:.1}" stroke="{color}" stroke-width="3" stroke-linecap="round"/><rect x="{left:.1}" y="{top:.1}" width="{width:.1}" height="{body_height:.1}" rx="3" fill="{color}"/>"##,
            x = x,
            high_y = y(candle.high),
            low_y = y(candle.low),
            color = color,
            left = x - width / 2.0,
            top = top,
            width = width,
            body_height = body_height,
        ));
    }

    format!(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64">{body}</svg>"#)
}

pub fn direction(change_percent: f64) -> String {
    let (color, points) = if change_percent >= 0.0 {
        ("#10B981", "32,7 52,31 40,31 40,57 24,57 24,31 12,31")
    } else {
        ("#F05252", "24,7 40,7 40,33 52,33 32,57 12,33 24,33")
    };
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"><polygon points="{points}" fill="{color}"/></svg>"#
    )
}
