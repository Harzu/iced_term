use iced::advanced::graphics::core::Element;
use iced::font::{Family, Stretch, Weight};
use iced::widget::container;
use iced::{
    window, Font, Length, Size, Subscription, Task, Theme
};

fn main() -> iced::Result {
    iced::application("full_screen", App::update, App::view)
        .antialiasing(false)
        .window_size(Size { width: 1280.0, height: 720.0 })
        .subscription(App::subscription)
        .run_with(App::new)
}

#[derive(Debug, Clone)]
pub enum Message {
    IcedTermEvent(iced_term::Event),
}

struct App {
    term: iced_term::Terminal,
}

impl App {
    fn new() -> (Self, Task<Message>) {
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
                    ..Default::default()
                },
                ..Default::default()
            },
            theme: iced_term::settings::ThemeSettings::default(),
            backend: iced_term::settings::BackendSettings {
                shell: system_shell.to_string(),
            },
        };

        (
            Self {
                term: iced_term::Terminal::new(term_id, term_settings.clone()),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::IcedTermEvent(iced_term::Event::CommandReceived(
                _,
                cmd,
            )) => match self.term.update(cmd) {
                iced_term::actions::Action::Shutdown => {
                    window::get_latest().and_then(window::close)
                },
                _ => Task::none(),
            },
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let term_subscription = iced_term::TerminalSubscription::new(self.term.term_id());
        let term_event_stream = term_subscription.event_stream();
        Subscription::run_with_id(self.term.term_id(), term_event_stream).map(Message::IcedTermEvent)
    }

    fn view(&self) -> Element<Message, Theme, iced::Renderer> {
        container(iced_term::term_view(&self.term).map(Message::IcedTermEvent))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
