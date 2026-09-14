mod app;
mod icon;
mod market_v2;
mod settings;

fn main() -> cosmic::iced::Result {
    cosmic::applet::run::<app::AppModel>(())
}
