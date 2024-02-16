use iced::advanced::graphics::core::Element;
use iced::font::{Family, Stretch, Weight};
use iced::widget::{button, column, container, row};
use iced::{
    executor, window, Application, Command, Font, Length, Settings, Size,
    Subscription, Theme,
};

const TERM_FONT_JET_BRAINS_BYTES: &[u8] = include_bytes!(
    "../assets/fonts/JetBrains/JetBrainsMonoNerdFontMono-Bold.ttf"
);

const TERM_FONT_3270_BYTES: &[u8] =
    include_bytes!("../assets/fonts/3270/3270NerdFont-Regular.ttf");

fn main() -> iced::Result {
    App::run(Settings {
        antialiasing: true,
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
    FontLoaded(Result<(), iced::font::Error>),
    FontChanged(String),
    FontSizeInc,
    FontSizeDec,
}

struct App {
    term: iced_term::Term,
    font_setting: iced_term::FontSettings,
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
                    stretch: Stretch::Normal,
                    ..Font::default()
                },
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
                font_setting: term_settings.font,
            },
            Command::batch(vec![
                iced::font::load(TERM_FONT_JET_BRAINS_BYTES)
                    .map(Message::FontLoaded),
                iced::font::load(TERM_FONT_3270_BYTES).map(Message::FontLoaded),
            ]),
        )
    }

    fn title(&self) -> String {
        String::from("Terminal app")
    }

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match message {
            Message::FontLoaded(_) => Command::none(),
            Message::FontChanged(name) => {
                if name.as_str() == "3270" {
                    self.font_setting.font_type = Font {
                        weight: Weight::Normal,
                        family: Family::Name("IBM 3270"),
                        ..Font::default()
                    };
                } else {
                    self.font_setting.font_type = Font {
                        weight: Weight::Bold,
                        family: Family::Name("JetBrains Mono"),
                        ..Font::default()
                    };
                };

                self.term.update(iced_term::Command::ChangeFont(
                    self.font_setting.clone(),
                ));
                Command::none()
            },
            Message::FontSizeInc => {
                self.font_setting.size += 1.0;
                self.term.update(iced_term::Command::ChangeFont(
                    self.font_setting.clone(),
                ));
                Command::none()
            },
            Message::FontSizeDec => {
                if self.font_setting.size > 0.0 {
                    self.font_setting.size -= 1.0;
                    self.term.update(iced_term::Command::ChangeFont(
                        self.font_setting.clone(),
                    ));
                }
                Command::none()
            },
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
        let content = column![
            row![
                button("JetBrains").width(Length::Fill).padding(8).on_press(
                    Message::FontChanged("JetBrains Mono".to_string())
                ),
                button("3270")
                    .width(Length::Fill)
                    .padding(8)
                    .on_press(Message::FontChanged("3270".to_string())),
            ],
            row![
                button("+ size")
                    .width(Length::Fill)
                    .padding(8)
                    .on_press(Message::FontSizeInc),
                button("- size")
                    .width(Length::Fill)
                    .padding(8)
                    .on_press(Message::FontSizeDec),
            ],
            row![iced_term::term_view(&self.term).map(Message::IcedTermEvent)],
        ];

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
