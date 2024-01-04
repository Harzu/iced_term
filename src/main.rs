use std::collections::HashMap;
use iced::{
    executor, window, Command, Element, Length,
    Settings, Theme, Application, Subscription,
};
use iced::widget::container;

use iced_myterm::backend::Pty;
use iced_myterm::component::{self, ITermView};

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
    TermMessage(component::Message),
    GlobalEvent(iced::Event),
}

struct App {
    _width: u32,
    _height: u32,
    tabs: HashMap<u64, (Pty, component::ITermView)>,
}

impl Application for App {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();
    
    fn new(_flags: ()) -> (Self, Command<Message>) {
        let tab_id = 0;
        let tab = component::iterm(tab_id).unwrap();
        let mut tabs = HashMap::new();
        tabs.insert(tab_id, tab);
        (
            Self {
                tabs,
                _width: 800,
                _height: 600,
            },
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
                    component::Message::CharacterReceived(id, c) => {
                        let tab = self.tabs.get_mut(&id).unwrap();
                        tab.0.write_to_pty(c)
                    },
                    component::Message::DataUpdated(id, data) => {
                        let tab = self.tabs.get_mut(&id).unwrap();
                        tab.0.update(data);
                        let cells = tab.0.cells();
                        tab.1.update(cells);
                    }
                    _ => {}
                };

                Command::none()
            },
            Message::GlobalEvent(e) => {
                match e {
                    iced::Event::Window(window_event) => match window_event {
                        iced::window::Event::Resized { width, height } => {
                            let tab_id = 0;
                            let tab = self.tabs.get_mut(&tab_id).unwrap();
                            
                            let width = width.max(1);
                            let height = height.max(1);

                            let h = (height as f32 / 20.0).round() as u16;
                            let w = (width as f32 / 13.0).round() as u16;

                            println!("{}|{} (rows {} cols {})", height, width, h, w);
                            tab.0.resize(h as u16, w as u16);
                            tab.1.request_redraw();
                        }
                        _ => {},
                    },
                    _ => {}
                }

                Command::none()
            }
            _ => Command::none()
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut sb = vec![];
        for id in self.tabs.keys() {
            let tab = self.tabs.get(id).unwrap();
            let sub = ITermView::on_data_received(id.clone(), tab.0.reader())
                .map(|e| Message::TermMessage(e));

            sb.push(sub)
        }

        let global_event_sub = iced::subscription::events().map(|e| Message::GlobalEvent(e));
        sb.push(global_event_sub);

        Subscription::batch(sb)
    }

    fn view(&self) -> Element<Message> {
        let tab_id = 0;
        let tab = self.tabs.get(&tab_id).unwrap();
        let tab_view = tab.1.view()
            .map(move |e| Message::TermMessage(e));

        container(tab_view)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
