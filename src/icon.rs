use crate::market::Candle;

const MIN_ICON_SIZE: f64 = 16.0;
const MAX_ICON_SIZE: f64 = 96.0;

fn candle_color(candle: &Candle) -> &'static str {
    if candle.close >= candle.open {
        "#22C55E"
    } else {
        "#EF4444"
    }
}

fn valid(candle: &Candle) -> bool {
    [candle.open, candle.high, candle.low, candle.close]
        .into_iter()
        .all(|value| value.is_finite())
}

fn snap_half(value: f64) -> f64 {
    (value * 2.0).round() / 2.0
}

#[derive(Clone, Copy, Debug)]
struct Metrics {
    margin: f64,
    body_width: f64,
    wick_width: f64,
    min_body_height: f64,
    radius: f64,
    wick_band: f64,
}

impl Metrics {
    fn for_size(size: f64) -> Self {
        if size <= 24.0 {
            Self {
                margin: 2.0,
                body_width: (size * 0.17).max(3.5),
                wick_width: 1.25,
                min_body_height: 3.0,
                radius: 0.5,
                wick_band: size * 0.14,
            }
        } else if size <= 36.0 {
            Self {
                margin: 3.0,
                body_width: size * 0.17,
                wick_width: 1.5,
                min_body_height: 3.5,
                radius: 0.75,
                wick_band: size * 0.14,
            }
        } else {
            Self {
                margin: size * 0.085,
                body_width: size * 0.16,
                wick_width: (size * 0.045).clamp(1.75, 2.5),
                min_body_height: (size * 0.075).clamp(4.0, 5.5),
                radius: (size * 0.025).clamp(0.75, 1.5),
                wick_band: size * 0.15,
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct PriceScale {
    full_low: f64,
    full_high: f64,
    core_low: f64,
    core_high: f64,
    top: f64,
    bottom: f64,
    wick_band: f64,
}

impl PriceScale {
    fn y(self, price: f64) -> f64 {
        const EPSILON: f64 = 1e-12;

        let core_top = self.top + self.wick_band;
        let core_bottom = self.bottom - self.wick_band;
        let core_height = (core_bottom - core_top).max(1.0);

        if price > self.core_high && self.full_high > self.core_high + EPSILON {
            let ratio =
                ((price - self.core_high) / (self.full_high - self.core_high)).clamp(0.0, 1.0);
            return core_top - self.wick_band * ratio.sqrt();
        }

        if price < self.core_low && self.full_low < self.core_low - EPSILON {
            let ratio = ((self.core_low - price) / (self.core_low - self.full_low)).clamp(0.0, 1.0);
            return core_bottom + self.wick_band * ratio.sqrt();
        }

        let span = (self.core_high - self.core_low).max(EPSILON);
        let ratio = ((price - self.core_low) / span).clamp(0.0, 1.0);
        core_bottom - ratio * core_height
    }
}

#[derive(Clone, Copy, Debug)]
struct Geometry {
    x: f64,
    wick_top: f64,
    wick_bottom: f64,
    body_top: f64,
    body_height: f64,
    body_width: f64,
    wick_width: f64,
    radius: f64,
}

fn build_geometry(candles: &[&Candle], size: f64) -> Vec<Geometry> {
    if candles.is_empty() {
        return Vec::new();
    }

    let metrics = Metrics::for_size(size);
    let full_low = candles
        .iter()
        .map(|c| c.low.min(c.open).min(c.close))
        .fold(f64::INFINITY, f64::min);
    let full_high = candles
        .iter()
        .map(|c| c.high.max(c.open).max(c.close))
        .fold(f64::NEG_INFINITY, f64::max);
    let body_low = candles
        .iter()
        .map(|c| c.open.min(c.close))
        .fold(f64::INFINITY, f64::min);
    let body_high = candles
        .iter()
        .map(|c| c.open.max(c.close))
        .fold(f64::NEG_INFINITY, f64::max);

    let full_span = (full_high - full_low).max(1e-9);
    let body_span = (body_high - body_low).max(0.0);
    let minimum_core_span = full_span * 0.20;
    let (core_low, core_high) = if body_span < minimum_core_span {
        let center = (body_low + body_high) / 2.0;
        (
            center - minimum_core_span / 2.0,
            center + minimum_core_span / 2.0,
        )
    } else {
        (body_low, body_high)
    };

    let scale = PriceScale {
        full_low,
        full_high,
        core_low,
        core_high,
        top: metrics.margin,
        bottom: size - metrics.margin,
        wick_band: metrics.wick_band,
    };

    let centers = [size * 0.27, size * 0.50, size * 0.73];
    let last = candles.len().saturating_sub(1);

    candles
        .iter()
        .enumerate()
        .map(|(index, candle)| {
            let x = snap_half(centers.get(index).copied().unwrap_or(size * 0.50));
            let latest_multiplier = if index == last { 1.08 } else { 1.0 };
            let body_width = snap_half(metrics.body_width * latest_multiplier).max(3.0);
            let wick_top = snap_half(scale.y(candle.high.max(candle.open).max(candle.close)));
            let wick_bottom = snap_half(scale.y(candle.low.min(candle.open).min(candle.close)));

            let open_y = scale.y(candle.open);
            let close_y = scale.y(candle.close);
            let raw_top = open_y.min(close_y);
            let raw_bottom = open_y.max(close_y);
            let target_height = (raw_bottom - raw_top).max(metrics.min_body_height);
            let half = target_height / 2.0;
            let center = ((raw_top + raw_bottom) / 2.0)
                .clamp(metrics.margin + half, size - metrics.margin - half);
            let body_top = snap_half(center - half);
            let body_bottom = snap_half(center + half);
            let body_height = (body_bottom - body_top).max(metrics.min_body_height);

            Geometry {
                x,
                wick_top: wick_top.min(body_top),
                wick_bottom: wick_bottom.max(body_top + body_height),
                body_top,
                body_height,
                body_width,
                wick_width: metrics.wick_width,
                radius: metrics.radius.min(body_height / 2.0),
            }
        })
        .collect()
}

pub fn render(candles: &[Candle], panel_height: f32) -> String {
    let size = f64::from(panel_height).clamp(MIN_ICON_SIZE, MAX_ICON_SIZE);
    let visible = candles
        .iter()
        .rev()
        .filter(|candle| valid(candle))
        .take(3)
        .collect::<Vec<_>>();
    let ordered = visible.into_iter().rev().collect::<Vec<_>>();

    if ordered.is_empty() {
        let inset = size * 0.14;
        let side = size - inset * 2.0;
        let radius = size * 0.22;
        return format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {size:.1} {size:.1}"><rect x="{inset:.1}" y="{inset:.1}" width="{side:.1}" height="{side:.1}" rx="{radius:.1}" fill="#69707D"/></svg>"##
        );
    }

    let geometry = build_geometry(&ordered, size);
    let mut body = String::new();

    for (candle, shape) in ordered.iter().zip(geometry.iter()) {
        let color = candle_color(candle);
        let left = snap_half(shape.x - shape.body_width / 2.0);

        body.push_str(&format!(
            r##"<line x1="{x:.1}" y1="{wick_top:.1}" x2="{x:.1}" y2="{wick_bottom:.1}" stroke="{color}" stroke-width="{wick_width:.2}" stroke-linecap="round"/><rect x="{left:.1}" y="{body_top:.1}" width="{body_width:.1}" height="{body_height:.1}" rx="{radius:.2}" fill="{color}"/>"##,
            x = shape.x,
            wick_top = shape.wick_top,
            wick_bottom = shape.wick_bottom,
            color = color,
            wick_width = shape.wick_width,
            left = left,
            body_top = shape.body_top,
            body_width = shape.body_width,
            body_height = shape.body_height,
            radius = shape.radius,
        ));
    }

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {size:.1} {size:.1}" shape-rendering="geometricPrecision">{body}</svg>"#
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
    fn keeps_three_candles_readable_with_extreme_wick() {
        let candles = [
            candle(100.0, 101.0, 99.5, 100.3),
            candle(100.3, 160.0, 100.0, 100.6),
            candle(100.6, 101.1, 100.2, 100.4),
        ];
        let refs = candles.iter().collect::<Vec<_>>();
        let geometry = build_geometry(&refs, 24.0);

        assert_eq!(geometry.len(), 3);
        assert!(geometry.iter().all(|shape| shape.body_height >= 3.0));
        assert!(geometry[1].wick_top < geometry[1].body_top);
    }

    #[test]
    fn subtly_emphasizes_latest_candle() {
        let candles = [
            candle(10.0, 11.0, 9.0, 10.5),
            candle(10.5, 11.5, 10.0, 11.0),
            candle(11.0, 12.0, 10.5, 11.5),
        ];
        let refs = candles.iter().collect::<Vec<_>>();
        let geometry = build_geometry(&refs, 32.0);

        assert!(geometry[2].body_width > geometry[1].body_width);
    }

    #[test]
    fn renderer_uses_effective_panel_height() {
        let candles = [candle(10.0, 11.0, 9.0, 10.5)];
        let svg = render(&candles, 28.0);

        assert!(svg.contains(r#"viewBox="0 0 28.0 28.0""#));
    }
}
