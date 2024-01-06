use std::collections::HashMap;
use iced::widget::container;
use iced::{
    executor, window, Command, Element, Length,
    Settings, Theme, Application, Subscription,
};
use iced_term::{self, Event, Term};

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
    TermMessage(Event),
    GlobalEvent(iced::Event),
}

struct App {
    tabs: HashMap<u64, Term>,
}

impl Application for App {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();
    
    fn new(_flags: ()) -> (Self, Command<Message>) {
        let tab_id = 0;
        let tab = iced_term::Term::new(tab_id, 10.0);
        let mut tabs = HashMap::new();
        tabs.insert(tab_id, tab);
        (
            Self { tabs },
            Command::none(),
        )
    }


    fn title(&self) -> String {
        String::from("Terminal app")
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Message> {
        match message {
            Message::TermMessage(m) => {
                match m {
                    Event::InputReceived(id, c) => {
                        let tab = self.tabs.get_mut(&id).expect("tab with target id not found");
                        tab.update(iced_term::Command::WriteToPTY(c))
                    },
                    Event::DataUpdated(id, data) => {
                        let tab = self.tabs.get_mut(&id).expect("tab with target id not found");
                        tab.update(iced_term::Command::RenderData(data))
                    },
                    Event::ContainerScrolled(id, delta) => {
                        let tab = self.tabs.get_mut(&id).expect("tab with target id not found");
                        tab.update(iced_term::Command::Scroll(delta.1 as i32))
                    }
                    _ => {}
                };

                Command::none()
            },
            Message::GlobalEvent(e) => {
                match e {
                    iced::Event::Window(window_event) => match window_event {
                        iced::window::Event::Resized { width, height } => {
                            self.tabs.iter_mut().for_each(|(id, tab)| {
                                tab.update(iced_term::Command::Resize(width, height));
                            });
                        }
                        _ => {},
                    },
                    _ => {}
                }

                Command::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut sb = vec![];
        for id in self.tabs.keys() {
            let tab = self.tabs.get(id).unwrap();
            let sub = iced_term::data_received_subscription(
                id.clone(),
                tab.pty_data_reader()
            )
                .map(|e| Message::TermMessage(e));

            sb.push(sub)
        }

        let global_event_sub = iced::subscription::events().map(|e| Message::GlobalEvent(e));
        sb.push(global_event_sub);

        Subscription::batch(sb)
    }

    fn view(&self) -> Element<Message> {
        let tab_id = 0;
        let tab = self.tabs.get(&tab_id).expect("tab with target id not found");
        let tab_view = tab.view()
            .map(move |e| Message::TermMessage(e));

        container(tab_view)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
