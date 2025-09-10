use iced::advanced::graphics::core::Element;
use iced::font::{Family, Stretch, Weight};
use iced::widget::{button, column, container, row};
use iced::{window, Font, Length, Size, Subscription, Task, Theme};
use iced_term::TerminalView;

const TERM_FONT_JET_BRAINS_BYTES: &[u8] = include_bytes!(
    "../assets/fonts/JetBrains/JetBrainsMonoNerdFontMono-Bold.ttf"
);

const TERM_FONT_3270_BYTES: &[u8] =
    include_bytes!("../assets/fonts/3270/3270NerdFont-Regular.ttf");

fn main() -> iced::Result {
    iced::application(App::title, App::update, App::view)
        .antialiasing(false)
        .window_size(Size {
            width: 1280.0,
            height: 720.0,
        })
        .subscription(App::subscription)
        .font(TERM_FONT_JET_BRAINS_BYTES)
        .font(TERM_FONT_3270_BYTES)
        .run_with(App::new)
}

#[derive(Debug, Clone)]
pub enum Event {
    Terminal(iced_term::Event),
    FontLoaded(Result<(), iced::font::Error>),
    FontChanged(String),
    FontSizeInc,
    FontSizeDec,
}

struct App {
    title: String,
    term: iced_term::Terminal,
    font_setting: iced_term::settings::FontSettings,
}

impl App {
    fn new() -> (Self, Task<Event>) {
        let system_shell = std::env::var("SHELL")
            .expect("SHELL variable is not defined")
            .to_string();
        let term_id = 0;
        let term_settings = iced_term::settings::Settings {
            font: iced_term::settings::FontSettings {
                size: 14.0,
                font_type: Font {
                    weight: Weight::Bold,
                    family: Family::Name("JetBrainsMono Nerd Font Mono"),
                    stretch: Stretch::Normal,
                    ..Font::default()
                },
                ..Default::default()
            },
            theme: iced_term::settings::ThemeSettings::default(),
            backend: iced_term::settings::BackendSettings {
                program: system_shell,
                ..Default::default()
            },
        };

        (
            Self {
                title: String::from("fonts"),
                term: iced_term::Terminal::new(term_id, term_settings.clone()),
                font_setting: term_settings.font,
            },
            Task::none(),
        )
    }

    fn title(&self) -> String {
        self.title.clone()
    }

    fn subscription(&self) -> Subscription<Event> {
        let term_subscription = iced_term::Subscription::new(self.term.id);
        let term_event_stream = term_subscription.event_stream();
        Subscription::run_with_id(self.term.id, term_event_stream)
            .map(Event::Terminal)
    }

    fn update(&mut self, event: Event) -> Task<Event> {
        match event {
            Event::FontLoaded(_) => Task::none(),
            Event::FontChanged(name) => {
                if name.as_str() == "3270" {
                    self.font_setting.font_type = Font {
                        weight: Weight::Normal,
                        family: Family::Name("3270 Nerd Font"),
                        ..Font::default()
                    };
                } else {
                    self.font_setting.font_type = Font {
                        weight: Weight::Bold,
                        family: Family::Name("JetBrainsMono Nerd Font Mono"),
                        ..Font::default()
                    };
                };

                self.term.update(iced_term::Command::ChangeFont(
                    self.font_setting.clone(),
                ));
                Task::none()
            },
            Event::FontSizeInc => {
                self.font_setting.size += 1.0;
                self.term.update(iced_term::Command::ChangeFont(
                    self.font_setting.clone(),
                ));
                Task::none()
            },
            Event::FontSizeDec => {
                if self.font_setting.size > 0.0 {
                    self.font_setting.size -= 1.0;
                    self.term.update(iced_term::Command::ChangeFont(
                        self.font_setting.clone(),
                    ));
                }
                Task::none()
            },
            Event::Terminal(iced_term::Event::CommandReceived(_, cmd)) => {
                match self.term.update(cmd) {
                    iced_term::actions::Action::Shutdown => {
                        window::get_latest().and_then(window::close)
                    },
                    iced_term::actions::Action::ChangeTitle(title) => {
                        self.title = title;
                        Task::none()
                    },
                    _ => Task::none(),
                }
            },
        }
    }

    fn view(&self) -> Element<Event, Theme, iced::Renderer> {
        let content = column![
            row![
                button("JetBrains")
                    .width(Length::Fill)
                    .padding(8)
                    .on_press(Event::FontChanged("JetBrains Mono".to_string())),
                button("3270")
                    .width(Length::Fill)
                    .padding(8)
                    .on_press(Event::FontChanged("3270".to_string())),
            ],
            row![
                button("+ size")
                    .width(Length::Fill)
                    .padding(8)
                    .on_press(Event::FontSizeInc),
                button("- size")
                    .width(Length::Fill)
                    .padding(8)
                    .on_press(Event::FontSizeDec),
            ],
            row![TerminalView::show(&self.term).map(Event::Terminal)],
        ];

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
