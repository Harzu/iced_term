use iced::advanced::graphics::core::Element;
use iced::font::{Family, Stretch, Weight};
use iced::widget::container;
use iced::{
    executor, window, Application, Command, Font, Length, Settings,
    Subscription, Theme,
};
use iced_term;
use std::collections::HashMap;

const TERM_FONT_JET_BRAINS_BYTES: &[u8] =
    include_bytes!("../assets/fonts/JetBrains/JetBrainsMono-Bold.ttf");

fn main() -> iced::Result {
    App::run(Settings {
        antialiasing: true,
        window: window::Settings {
            size: (800, 600),
            ..window::Settings::default()
        },
        ..Settings::default()
    })
}

#[derive(Debug, Clone)]
pub enum Message {
    TermEvent(iced_term::Event),
    FontLoaded(Result<(), iced::font::Error>),
}

struct App {
    tabs: HashMap<u64, iced_term::Term>,
    term_settings: iced_term::TermSettings,
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let system_shell = env!("SHELL").to_string();
        let tab_id = 0;
        let term_settings = iced_term::TermSettings {
            font: iced_term::FontSettings {
                size: 14.0,
                font_type: Font {
                    weight: Weight::Bold,
                    family: Family::Name("JetBrains Mono"),
                    monospaced: false,
                    stretch: Stretch::Normal,
                },
                ..iced_term::FontSettings::default()
            },
            backend: iced_term::BackendSettings {
                shell: system_shell.to_string(),
                ..iced_term::BackendSettings::default()
            },
        };
        let tab = iced_term::Term::new(tab_id, term_settings.clone());
        let mut tabs = HashMap::new();
        tabs.insert(tab_id, tab);
        (
            Self {
                tabs,
                term_settings,
            },
            Command::batch(vec![iced::font::load(TERM_FONT_JET_BRAINS_BYTES)
                .map(Message::FontLoaded)]),
        )
    }

    fn title(&self) -> String {
        String::from("Terminal app")
    }

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match message {
            Message::FontLoaded(_) => Command::none(),
            Message::TermEvent(event) => {
                match event {
                    iced_term::Event::InputReceived(id, input) => {
                        if let Some(tab) = self.tabs.get_mut(&id) {
                            tab.update(iced_term::Command::WriteToBackend(
                                input,
                            ))
                        }
                    },
                    iced_term::Event::Scrolled(id, delta) => {
                        if let Some(tab) = self.tabs.get_mut(&id) {
                            tab.update(iced_term::Command::Scroll(delta as i32))
                        }
                    },
                    iced_term::Event::Resized(id, size) => {
                        if let Some(tab) = self.tabs.get_mut(&id) {
                            tab.update(iced_term::Command::Resize(size));
                        }
                    },
                    iced_term::Event::BackendEventSenderReceived(id, tx) => {
                        println!("{}", id);

                        if let Some(tab) = self.tabs.get_mut(&id) {
                            tab.update(iced_term::Command::InitBackend(tx));
                        }
                    },
                    iced_term::Event::BackendEventReceived(id, inner_event) => {
                        if let Some(tab) = self.tabs.get_mut(&id) {
                            tab.update(
                                iced_term::Command::ProcessBackendEvent(
                                    inner_event,
                                ),
                            );
                        }
                    },
                    _ => {},
                };

                Command::none()
            },
        }
    }

    fn view(&self) -> Element<Message, iced::Renderer> {
        let tab_id = 0;
        let tab = self
            .tabs
            .get(&tab_id)
            .expect("tab with target id not found");

        let tab_view = tab.view().map(Message::TermEvent);

        container(tab_view)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut sb = vec![];
        for id in self.tabs.keys() {
            let tab = self.tabs.get(id).unwrap();
            let sub = tab.subscription().map(Message::TermEvent);
            sb.push(sub)
        }

        Subscription::batch(sb)
    }
}
