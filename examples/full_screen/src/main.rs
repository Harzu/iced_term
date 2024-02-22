use iced::advanced::graphics::core::Element;
use iced::widget::container;
use iced::{
    executor, window, Application, Command, Font, Length, Settings, Size,
    Subscription, Theme,
};

fn main() -> iced::Result {
    App::run(Settings {
        antialiasing: false,
        window: window::Settings {
            size: Size {
                width: 1280.0,
                height: 720.0,
            },
            ..window::Settings::default()
        },
        ..Settings::default()
    })
}

#[derive(Debug, Clone)]
pub enum Message {
    IcedTermEvent(iced_term::Event),
}

struct App {
    term: iced_term::Term,
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let system_shell = std::env::var("SHELL")
            .expect("SHELL variable is not defined")
            .to_string();
        let term_id = 0;
        let term_settings = iced_term::TermSettings {
            font: iced_term::FontSettings {
                size: 14.0,
                font_type: Font::default(),
                ..Default::default()
            },
            theme: iced_term::ColorPalette::default(),
            backend: iced_term::BackendSettings {
                shell: system_shell.to_string(),
            },
        };

        (
            Self {
                term: iced_term::Term::new(term_id, term_settings.clone()),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Terminal app")
    }

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match message {
            Message::IcedTermEvent(iced_term::Event::CommandReceived(
                _,
                cmd,
            )) => match self.term.update(cmd) {
                iced_term::actions::Action::Shutdown => {
                    window::close(window::Id::MAIN)
                },
                _ => Command::none(),
            },
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        self.term.subscription().map(Message::IcedTermEvent)
    }

    fn view(&self) -> Element<Message, Theme, iced::Renderer> {
        container(iced_term::term_view(&self.term).map(Message::IcedTermEvent))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
