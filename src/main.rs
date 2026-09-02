mod app;
mod icon;
mod market;

fn main() -> cosmic::iced::Result {
    cosmic::applet::run::<app::AppModel>(())
}
