use iced::font::{Family, Weight};
use iced::widget::{column, container, row, text, text_editor};
use iced::{window, Font, Length, Size, Subscription, Task, Theme};
use iced_term::TerminalView;

const TERM_FONT_JET_BRAINS_BYTES: &[u8] = include_bytes!(
    "../assets/fonts/JetBrains/JetBrainsMonoNerdFontMono-Bold.ttf"
);

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title(App::title)
        .window_size(Size {
            width: 1400.0,
            height: 800.0,
        })
        .subscription(App::subscription)
        .font(TERM_FONT_JET_BRAINS_BYTES)
        .run()
}

#[derive(Debug, Clone)]
enum Event {
    EditorAction(text_editor::Action),
    Terminal(iced_term::Event),
}

struct App {
    title: String,
    editor: text_editor::Content,
    term: iced_term::Terminal,
}

impl App {
    fn new() -> (Self, Task<Event>) {
        #[cfg(not(windows))]
        let system_shell = std::env::var("SHELL")
            .expect("SHELL variable is not defined")
            .to_string();
        #[cfg(windows)]
        let system_shell = "cmd.exe".to_string();

        let term_settings = iced_term::settings::Settings {
            font: iced_term::settings::FontSettings {
                size: 14.0,
                font_type: Font {
                    weight: Weight::Bold,
                    family: Family::Name("JetBrainsMono Nerd Font Mono"),
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
                title: String::from("focus"),
                editor: text_editor::Content::with_text(
                    "fn main() {\n    println!(\"Hello from iced text editor\");\n}\n",
                ),
                term: iced_term::Terminal::new(0, term_settings)
                    .expect("failed to create the new terminal instance"),
            },
            Task::none(),
        )
    }

    fn title(&self) -> String {
        self.title.clone()
    }

    fn subscription(&self) -> Subscription<Event> {
        self.term.subscription().map(Event::Terminal)
    }

    fn update(&mut self, event: Event) -> Task<Event> {
        match event {
            Event::EditorAction(action) => {
                self.editor.perform(action);
            },
            Event::Terminal(iced_term::Event::BackendCall(_, cmd)) => {
                match self.term.handle(iced_term::Command::ProxyToBackend(cmd))
                {
                    iced_term::actions::Action::Shutdown => {
                        return window::latest().and_then(window::close)
                    },
                    iced_term::actions::Action::ChangeTitle(title) => {
                        self.title = title;
                    },
                    iced_term::actions::Action::Ignore => {},
                }
            },
        }

        Task::none()
    }

    fn view(&self) -> iced::Element<'_, Event, Theme, iced::Renderer> {
        let editor = column![
            text("Iced Text Editor").size(24),
            text_editor(&self.editor)
                .placeholder("Type your notes or code here...")
                .on_action(Event::EditorAction)
                .height(Length::Fill),
        ]
        .spacing(12)
        .width(Length::Fill)
        .height(Length::Fill);

        let terminal = column![
            text("iced_term Widget").size(24),
            TerminalView::show(&self.term).map(Event::Terminal),
        ]
        .spacing(12)
        .width(Length::Fill)
        .height(Length::Fill);

        container(
            row![
                container(editor).width(Length::FillPortion(1)).padding(16),
                container(terminal)
                    .width(Length::FillPortion(1))
                    .padding(16),
            ]
            .spacing(8)
            .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}
