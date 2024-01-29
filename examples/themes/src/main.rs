use iced::advanced::graphics::core::Element;
use iced::font::{Family, Stretch, Weight};
use iced::widget::{button, column, container, row};
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
    IcedTermEvent(iced_term::Event),
    FontLoaded(Result<(), iced::font::Error>),
    ThemeChanged(iced_term::ColorPalette),
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
            theme: iced_term::ColorPalette::default(),
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
            Message::ThemeChanged(palette) => {
                self.term
                    .update(iced_term::Command::ChangeTheme(Box::new(palette)));
                Command::none()
            },
            Message::IcedTermEvent(event) => {
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
        self.term.subscription().map(Message::IcedTermEvent)
    }

    fn view(&self) -> Element<Message, iced::Renderer> {
        let content = column![
            row![
                button("default").width(Length::Fill).padding(8).on_press(
                    Message::ThemeChanged(iced_term::ColorPalette::default())
                ),
                button("ubuntu").width(Length::Fill).padding(8).on_press(
                    Message::ThemeChanged(iced_term::ColorPalette {
                        background: String::from("#300A24"),
                        foreground: String::from("#FFFFFF"),
                        black: String::from("#2E3436"),
                        red: String::from("#CC0000"),
                        green: String::from("#4E9A06"),
                        yellow: String::from("#C4A000"),
                        blue: String::from("#3465A4"),
                        magenta: String::from("#75507B"),
                        cyan: String::from("#06989A"),
                        white: String::from("#D3D7CF"),
                        bright_black: String::from("#555753"),
                        bright_red: String::from("#EF2929"),
                        bright_green: String::from("#8AE234"),
                        bright_yellow: String::from("#FCE94F"),
                        bright_blue: String::from("#729FCF"),
                        bright_magenta: String::from("#AD7FA8"),
                        bright_cyan: String::from("#34E2E2"),
                        bright_white: String::from("#EEEEEC"),
                        ..Default::default()
                    })
                ),
                button("3024 Day").width(Length::Fill).padding(8).on_press(
                    Message::ThemeChanged(iced_term::ColorPalette {
                        background: String::from("#F7F7F7"),
                        foreground: String::from("#4A4543"),
                        black: String::from("#090300"),
                        red: String::from("#DB2D20"),
                        green: String::from("#01A252"),
                        yellow: String::from("#FDED02"),
                        blue: String::from("#01A0E4"),
                        magenta: String::from("#A16A94"),
                        cyan: String::from("#B5E4F4"),
                        white: String::from("#A5A2A2"),
                        bright_black: String::from("#5C5855"),
                        bright_red: String::from("#E8BBD0"),
                        bright_green: String::from("#3A3432"),
                        bright_yellow: String::from("#4A4543"),
                        bright_blue: String::from("#807D7C"),
                        bright_magenta: String::from("#D6D5D4"),
                        bright_cyan: String::from("#CDAB53"),
                        bright_white: String::from("#F7F7F7"),
                        ..Default::default()
                    })
                ),
            ],
            row![iced_term::term_view(&self.term).map(Message::IcedTermEvent)]
        ];

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
