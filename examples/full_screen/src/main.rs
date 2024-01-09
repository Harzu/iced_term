use iced::advanced::graphics::core::Element;
use iced::font::{Family, Stretch, Weight};
use iced::widget::container;
use iced::{
    executor, window, Application, Command, Font, Length, Settings,
    Subscription, Theme,
};
use iced_term;

const TERM_FONT_JET_BRAINS_BYTES: &[u8] =
    include_bytes!("../assets/fonts/JetBrains/JetBrainsMono-Bold.ttf");

fn main() -> iced::Result {
    App::run(Settings {
        antialiasing: true,
        window: window::Settings {
            size: (1280, 720),
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

        (
            Self {
                term: iced_term::Term::new(term_id, term_settings.clone()),
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
                    iced_term::Event::InputReceived(_, input) => {
                        self.term
                            .update(iced_term::Command::WriteToBackend(input));
                    },
                    iced_term::Event::Scrolled(_, delta) => self
                        .term
                        .update(iced_term::Command::Scroll(delta as i32)),
                    iced_term::Event::Resized(_, size) => {
                        self.term.update(iced_term::Command::Resize(size));
                    },
                    iced_term::Event::BackendEventSenderReceived(_, tx) => {
                        self.term.update(iced_term::Command::InitBackend(tx));
                    },
                    iced_term::Event::BackendEventReceived(_, inner_event) => {
                        self.term.update(
                            iced_term::Command::ProcessBackendEvent(
                                inner_event,
                            ),
                        );
                    },
                    iced_term::Event::Ignored(_) => {},
                };

                Command::none()
            },
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        self.term.subscription().map(Message::TermEvent)
    }

    fn view(&self) -> Element<Message, iced::Renderer> {
        let tab_view = self.term.view().map(Message::TermEvent);

        container(tab_view)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
