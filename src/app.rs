use crate::{icon, market::{self, Snapshot}};
use cosmic::app::{Core, Task};
use cosmic::iced::{Alignment, Length, Subscription, widget::svg};
use cosmic::prelude::*;
use cosmic::widget;

const APP_ID: &str = "com.github.drwesleyadv.Ticker";

pub struct AppModel {
    core: Core,
    snapshot: Snapshot,
}

impl Default for AppModel {
    fn default() -> Self {
        Self { core: Core::default(), snapshot: Snapshot::default() }
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

    fn core(&self) -> &Core { &self.core }
    fn core_mut(&mut self) -> &mut Core { &mut self.core }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Message>) {
        (Self { core, ..Default::default() }, Task::none())
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
        let panel_height = self.core.applet.suggested_size(true).1 as f32;
        let svg_data = icon::render(&self.snapshot.candles);
        let icon = widget::svg(svg::Handle::from_memory(svg_data.into_bytes()))
            .width(Length::Fixed(panel_height))
            .height(Length::Fixed(panel_height));

        let price = if self.snapshot.price > 0.0 {
            format!("${:.2}", self.snapshot.price)
        } else {
            "$--".to_string()
        };

        let row = widget::row()
            .push(icon)
            .push(widget::text(price).size(14))
            .spacing(4)
            .align_y(Alignment::Center);

        let content = widget::container(row)
            .height(Length::Fixed(panel_height))
            .padding([0, 6])
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
