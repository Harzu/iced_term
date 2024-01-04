use std::cell::RefCell;
use std::rc::Rc;

use iced::{
    executor, window, Command, Element, Length,
    Renderer, Settings, Theme, Application,
};
use iced::widget::{container, column, button, text};

use iced_myterm::component;

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

// #[derive(Debug, Clone, Copy)]
// pub enum Message {
//     TabLoaded
// }

struct App {
    _width: u32,
    _height: u32,
    terminal_tab: component::ITerm,
}

impl Application for App {
    type Message = ();
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();
    
    fn new(_flags: ()) -> (Self, Command<()>) {
        (
            Self {
                terminal_tab: component::ITerm::new(0).unwrap(),
                _width: 800,
                _height: 600,
            },
            Command::none(),
        )
    }


    fn title(&self) -> String {
        String::from("Terminal app")
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<()> {
        todo!()
    }

    fn view(&self) -> Element<()> {
        // let view = self.terminal_tab;
        let content = column![];

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
