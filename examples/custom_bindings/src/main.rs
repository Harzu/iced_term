use iced::advanced::graphics::core::Element;
use iced::font::{Family, Stretch, Weight};
use iced::keyboard::{KeyCode, Modifiers};
use iced::widget::container;
use iced::{
    executor, window, Application, Command, Font, Length, Settings,
    Subscription, Theme,
};
use iced_term::{
    self,
    bindings::{Binding, BindingAction, InputKind, KeyboardBinding},
    generate_bindings, TermMode,
};

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

        let custom_bindings = vec![
            (
                Binding {
                    target: InputKind::Char('c'),
                    modifiers: Modifiers::SHIFT,
                    terminal_mode_include: TermMode::ALT_SCREEN,
                    terminal_mode_exclude: TermMode::empty(),
                },
                BindingAction::Paste,
            ),
            (
                Binding {
                    target: InputKind::KeyCode(KeyCode::A),
                    modifiers: Modifiers::SHIFT | Modifiers::CTRL,
                    terminal_mode_include: TermMode::empty(),
                    terminal_mode_exclude: TermMode::empty(),
                },
                BindingAction::Char('B'),
            ),
            (
                Binding {
                    target: InputKind::KeyCode(KeyCode::B),
                    modifiers: Modifiers::SHIFT | Modifiers::CTRL,
                    terminal_mode_include: TermMode::empty(),
                    terminal_mode_exclude: TermMode::empty(),
                },
                BindingAction::Esc("\x1b[5~".into()),
            ),
        ];
        let mut term = iced_term::Term::new(term_id, term_settings);
        term.update(iced_term::Command::AddBindings(custom_bindings));

        // You can also use generate_bindings macros
        let custom_bindings = generate_bindings!(
            KeyboardBinding;
            'l', Modifiers::SHIFT; BindingAction::Char('K');
        );
        term.update(iced_term::Command::AddBindings(custom_bindings));

        (
            Self { term },
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
        container(iced_term::term_view(&self.term).map(Message::IcedTermEvent))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
