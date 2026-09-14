use crate::{
    icon,
    market::{self, Snapshot},
};
use cosmic::app::{Core, Task};
use cosmic::iced::{Alignment, Length, Subscription, widget::svg};
use cosmic::prelude::*;
use cosmic::widget;

const APP_ID: &str = "com.github.drwesleyadv.Ticker";
const TEXT_SIZE: u16 = 14;
const CONTENT_PADDING: [u16; 2] = [0, 6];
const CONTENT_SPACING: u16 = 4;
const DIRECTION_ICON_SCALE: f32 = 0.58;

pub struct AppModel {
    core: Core,
    snapshot: Snapshot,
}

impl Default for AppModel {
    fn default() -> Self {
        Self {
            core: Core::default(),
            snapshot: Snapshot::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    Market(Snapshot),
    Noop,
}

impl cosmic::Application for AppModel {
    type Executor = cosmic::SingleThreadExecutor;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = APP_ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Message>) {
        (
            Self {
                core,
                ..Default::default()
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        if let Message::Market(snapshot) = message {
            self.snapshot = snapshot;
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        market::subscription().map(Message::Market)
    }

    fn view(&self) -> Element<Message> {
        let panel_height = self.core.applet.suggested_size(true).1.max(1) as f32;
        let candle_icon = svg_widget(icon::render(&self.snapshot.candles), panel_height);
        let direction_icon = svg_widget(
            icon::direction(self.snapshot.change_percent),
            panel_height * DIRECTION_ICON_SCALE,
        );

        let row = widget::row()
            .push(candle_icon)
            .push(widget::text(price_label(self.snapshot.price)).size(TEXT_SIZE))
            .push(direction_icon)
            .push(
                widget::text(change_label(
                    self.snapshot.price,
                    self.snapshot.change_percent,
                ))
                .size(TEXT_SIZE),
            )
            .spacing(CONTENT_SPACING)
            .align_y(Alignment::Center);

        let content = widget::container(row)
            .height(Length::Fixed(panel_height))
            .padding(CONTENT_PADDING)
            .align_y(Alignment::Center);

        widget::button::custom(self.core.applet.autosize_window(content))
            .on_press(Message::Noop)
            .class(cosmic::theme::Button::AppletIcon)
            .into()
    }

    fn view_window(&self, _id: cosmic::iced::window::Id) -> Element<Message> {
        widget::text("").into()
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }
}

fn svg_widget(data: String, size: f32) -> cosmic::widget::Svg {
    widget::svg(svg::Handle::from_memory(data.into_bytes()))
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
}

fn price_label(price: f64) -> String {
    if price.is_finite() && price > 0.0 {
        format!("${price:.2}")
    } else {
        "$--".to_string()
    }
}

fn change_label(price: f64, change_percent: f64) -> String {
    if price.is_finite() && price > 0.0 && change_percent.is_finite() {
        format!("{change_percent:+.2}%")
    } else {
        "--%".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_available_market_values() {
        assert_eq!(price_label(123.456), "$123.46");
        assert_eq!(change_label(123.456, -1.234), "-1.23%");
    }

    #[test]
    fn hides_unavailable_market_values() {
        assert_eq!(price_label(0.0), "$--");
        assert_eq!(price_label(f64::NAN), "$--");
        assert_eq!(change_label(0.0, 1.0), "--%");
        assert_eq!(change_label(1.0, f64::NAN), "--%");
    }
}
