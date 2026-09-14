mod api;
mod model;
mod stream;

pub use model::{Candle, Snapshot};

pub fn subscription() -> cosmic::iced::Subscription<Snapshot> {
    cosmic::iced::Subscription::run(stream::run)
}
