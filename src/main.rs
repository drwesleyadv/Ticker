mod app;
mod icon;
mod market;
mod settings;

fn main() -> cosmic::iced::Result {
    cosmic::applet::run::<app::AppModel>(())
}
