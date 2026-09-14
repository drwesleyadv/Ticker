use crate::{
    icon,
    market_v2::{self, Snapshot},
    settings::{DraftSettings, Settings, TIMEFRAME_LABELS},
};
use cosmic::app::{Core, Task};
use cosmic::iced::core::window;
use cosmic::iced::window::Id;
use cosmic::iced::{Alignment, Length, Rectangle, Subscription, widget::svg};
use cosmic::prelude::*;
use cosmic::surface::action::{app_popup, destroy_popup};
use cosmic::widget::dropdown::popup_dropdown;
use cosmic::widget::{self, list_column, settings, toggler};

const APP_ID: &str = "com.github.drwesleyadv.Ticker";
const TEXT_SIZE: u16 = 14;
const CONTENT_PADDING: u16 = 6;
const CONTENT_SPACING: u16 = 4;

pub struct AppModel {
    core: Core,
    snapshot: Snapshot,
    settings: Settings,
    draft: DraftSettings,
    popup: Option<Id>,
    validation_error: Option<String>,
}

impl Default for AppModel {
    fn default() -> Self {
        let settings = Settings::load();
        Self {
            core: Core::default(),
            snapshot: Snapshot::default(),
            settings,
            draft: settings.into(),
            popup: None,
            validation_error: None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    Market(Snapshot),
    PopupClosed(Id),
    Surface(cosmic::surface::Action<Message>),
    DraftIntervalChanged(String),
    DraftTimeframeSelected(usize),
    DraftShowChange(bool),
    SaveSettings,
    DiscardSettings,
}

impl AppModel {
    fn close_popup(&mut self) -> Task<Message> {
        let Some(id) = self.popup.take() else {
            return Task::none();
        };
        cosmic::task::message(cosmic::Action::Surface(destroy_popup(id)))
    }

    fn preferences_view(&self) -> Element<'_, Message> {
        let interval = widget::text_input("1000", &self.draft.update_interval_ms)
            .on_input(Message::DraftIntervalChanged)
            .width(Length::Fixed(130.0));

        let timeframe = popup_dropdown(
            &TIMEFRAME_LABELS,
            Some(self.draft.timeframe_index),
            Message::DraftTimeframeSelected,
            self.popup.unwrap_or(Id::NONE),
            Message::Surface,
            |message| message,
        );

        let change_toggle = toggler(self.draft.show_change_percent)
            .on_toggle(Message::DraftShowChange);

        let actions = settings::item_row(vec![
            widget::button::standard("Descartar")
                .on_press(Message::DiscardSettings)
                .into(),
            widget::button::suggested("Salvar")
                .on_press(Message::SaveSettings)
                .into(),
        ]);

        let error = widget::text(self.validation_error.as_deref().unwrap_or(""));

        list_column()
            .add(
                settings::section()
                    .title("Ticker")
                    .add(settings::item("Atualização (ms)", interval))
                    .add(settings::item("Timeframe", timeframe))
                    .add(settings::item("Exibir variação percentual", change_toggle))
                    .add(error)
                    .add(actions),
            )
            .into()
    }
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
        let mut model = Self {
            core,
            ..Default::default()
        };
        model.draft = model.settings.into();
        (model, Task::none())
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Market(snapshot) => self.snapshot = snapshot,
            Message::PopupClosed(id) => {
                if self.popup == Some(id) {
                    self.popup = None;
                    self.draft = self.settings.into();
                    self.validation_error = None;
                }
            }
            Message::Surface(action) => {
                return cosmic::task::message(cosmic::Action::Surface(action));
            }
            Message::DraftIntervalChanged(value) => {
                self.draft.update_interval_ms = value;
                self.validation_error = None;
            }
            Message::DraftTimeframeSelected(index) => {
                self.draft.timeframe_index = index;
                self.validation_error = None;
            }
            Message::DraftShowChange(value) => {
                self.draft.show_change_percent = value;
            }
            Message::SaveSettings => match self.draft.parse() {
                Ok(settings) => match settings.save() {
                    Ok(()) => {
                        self.settings = settings;
                        self.draft = settings.into();
                        self.validation_error = None;
                        return self.close_popup();
                    }
                    Err(error) => {
                        self.validation_error = Some(format!("Não foi possível salvar: {error}"));
                    }
                },
                Err(error) => self.validation_error = Some(error),
            },
            Message::DiscardSettings => {
                self.draft = self.settings.into();
                self.validation_error = None;
                return self.close_popup();
            }
        }

        Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        market_v2::subscription(self.settings).map(Message::Market)
    }

    fn view(&self) -> Element<'_, Message> {
        let panel_height = self.core.applet.suggested_size(true).1 as f32;
        let candle_icon = widget::svg(svg::Handle::from_memory(
            icon::render(&self.snapshot.candles).into_bytes(),
        ))
        .width(Length::Fixed(panel_height))
        .height(Length::Fixed(panel_height));

        let price = if self.snapshot.price > 0.0 {
            format!("${:.2}", self.snapshot.price)
        } else {
            "$--".to_string()
        };

        let mut row = widget::row()
            .push(candle_icon)
            .push(widget::text(price).size(TEXT_SIZE));

        if self.settings.show_change_percent && self.snapshot.price > 0.0 {
            row = row.push(
                widget::text(format!("{:+.2}%", self.snapshot.change_percent)).size(TEXT_SIZE),
            );
        }

        let content = widget::container(
            row.spacing(CONTENT_SPACING).align_y(Alignment::Center),
        )
        .height(Length::Fixed(panel_height))
        .padding([0, CONTENT_PADDING])
        .align_y(Alignment::Center);

        let open_popup = self.popup;
        widget::button::custom(self.core.applet.autosize_window(content))
            .on_press_with_rectangle(move |offset, bounds| {
                if let Some(id) = open_popup {
                    Message::Surface(destroy_popup(id))
                } else {
                    Message::Surface(app_popup::<AppModel>(
                        |_| Default::default(),
                        move |state: &mut AppModel| {
                            state.draft = state.settings.into();
                            state.validation_error = None;
                            let id = Id::unique();
                            state.popup = Some(id);
                            let mut popup_settings = state.core.applet.get_popup_settings(
                                state.core.main_window_id().expect("main applet window"),
                                id,
                                None,
                                None,
                                None,
                            );
                            popup_settings.positioner.anchor_rect = Rectangle {
                                x: (bounds.x - offset.x) as i32,
                                y: (bounds.y - offset.y) as i32,
                                width: bounds.width as i32,
                                height: bounds.height as i32,
                            };
                            popup_settings
                        },
                        Some(Box::new(|state: &AppModel| {
                            Element::from(
                                state.core.applet.popup_container(state.preferences_view()),
                            )
                            .map(cosmic::Action::App)
                        })),
                    ))
                }
            })
            .class(cosmic::theme::Button::AppletIcon)
            .into()
    }

    fn view_window(&self, _id: Id) -> Element<'_, Message> {
        widget::text("").into()
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }
}
