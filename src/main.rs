use std::collections::HashMap;
use iced::widget::canvas::path::lyon_path::geom::euclid::default;
use iced::{
    executor, window, Command, Element, Length,
    Settings, Theme, Application, Subscription, Color,
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
    tabs: HashMap<u64, (Pty, component::ITermView)>,
}

impl Application for App {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();
    
    fn new(_flags: ()) -> (Self, Command<Message>) {
        let tab_id = 0;
        let tab = component::iterm(tab_id, 14.0).unwrap();
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
                    component::Message::CharacterReceived(id, c) => {
                        let (backend, _) = self.tabs.get_mut(&id).unwrap();
                        backend.write_to_pty(c)
                    },
                    component::Message::DataUpdated(id, data) => {
                        let (backend, view) = self.tabs.get_mut(&id).unwrap();
                        let rendarable_content = backend.update(data);
                        view.update(rendarable_content);
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
                            let (backend, view) = self.tabs.get_mut(&tab_id).unwrap();
                            let font_measure = view.font_measure.clone();
                            backend.resize(
                                width, 
                                height,
                                view.padding,
                                font_measure.width,
                                font_measure.height,
                            );
                            view.request_redraw();
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
        let (_, iterm_view) = self.tabs.get(&tab_id).unwrap();
        let tab_view = iterm_view.view()
            .map(move |e| Message::TermMessage(e));

        container(tab_view)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
