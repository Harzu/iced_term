use iced::advanced::graphics::core::Element;
use iced::widget::container;
use iced::{
    executor, window, Application, Command, Length, Settings, Subscription,
    Theme,
};
use iced_term::{self, BackendSettings, FontSettings, Term, TermSettings};
use std::collections::HashMap;

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
}

struct App {
    tabs: HashMap<u64, Term>,
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let system_shell = env!("SHELL");
        let tab_id = 0;
        let tab = iced_term::Term::new(
            tab_id,
            TermSettings {
                font: FontSettings { size: 14.0 },
                backend: BackendSettings {
                    shell: system_shell.to_string(),
                    ..BackendSettings::default()
                },
            },
        );
        let mut tabs = HashMap::new();
        tabs.insert(tab_id, tab);
        (Self { tabs }, Command::none())
    }

    fn title(&self) -> String {
        String::from("Terminal app")
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Message> {
        match message {
            Message::TermEvent(event) => {
                match event {
                    iced_term::Event::InputReceived(id, c) => {
                        let tab = self
                            .tabs
                            .get_mut(&id)
                            .expect("tab with target id not found");
                        tab.update(iced_term::Command::WriteToPTY(c))
                    },
                    iced_term::Event::DataUpdated(id, data) => {
                        let tab = self
                            .tabs
                            .get_mut(&id)
                            .expect("tab with target id not found");
                        tab.update(iced_term::Command::RenderData(data))
                    },
                    iced_term::Event::ContainerScrolled(id, delta) => {
                        let tab = self
                            .tabs
                            .get_mut(&id)
                            .expect("tab with target id not found");
                        tab.update(iced_term::Command::Scroll(delta as i32))
                    },
                    iced_term::Event::Resized(id, size) => {
                        let tab = self
                            .tabs
                            .get_mut(&id)
                            .expect("tab with target id not found");
                        tab.update(iced_term::Command::Resize(size));
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
            let sub = tab.data_subscription().map(Message::TermEvent);

            sb.push(sub)
        }

        Subscription::batch(sb)
    }
}
