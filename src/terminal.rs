use crate::actions::Action;
use crate::backend::{BackendCommand, Backend};
use crate::settings::{Settings, FontSettings, ThemeSettings, BackendSettings};
use crate::bindings::{Binding, BindingAction, BindingsLayout, InputKind};
use crate::font::TermFont;
use crate::theme::{Theme, ColorPalette};
use iced::widget::canvas::Cache;
use tokio::sync::mpsc::Sender;

#[derive(Debug, Clone)]
pub enum Event {
    CommandReceived(u64, Command),
}

#[derive(Debug, Clone)]
pub enum Command {
    InitBackend(Sender<alacritty_terminal::event::Event>),
    ChangeTheme(Box<ColorPalette>),
    ChangeFont(FontSettings),
    AddBindings(Vec<(Binding<InputKind>, BindingAction)>),
    ProcessBackendCommand(BackendCommand),
}

pub(crate) trait ViewProxy {
    fn id(&self) -> u64;
    fn bindings(&self) -> &BindingsLayout;
    fn cache(&self) -> &Cache;
    fn backend(&self) -> &Option<Backend>;
    fn theme(&self) -> &Theme;
    fn font(&self) -> &TermFont;
}

pub struct Terminal {
    id: u64,
    font: TermFont,
    theme: Theme,
    cache: Cache,
    bindings: BindingsLayout,
    backend_settings: BackendSettings,
    backend: Option<Backend>,
}

impl ViewProxy for Terminal {
    fn id(&self) -> u64 {
        self.id
    }

    fn backend(&self) -> &Option<Backend> {
        &self.backend
    }

    fn bindings(&self) -> &BindingsLayout {
        &self.bindings
    }

    fn cache(&self) -> &Cache {
        &self.cache
    }

    fn font(&self) -> &TermFont {
        &self.font
    }

    fn theme(&self) -> &Theme {
        &self.theme
    }
}

impl Terminal {
    pub fn new(id: u64, settings: Settings) -> Self {
        Self {
            id,
            font: TermFont::new(settings.font),
            theme: Theme::new(settings.theme),
            bindings: BindingsLayout::default(),
            cache: Cache::default(),
            backend_settings: settings.backend,
            backend: None,
        }
    }

    pub fn term_id(&self) -> u64 {
        self.id
    }

    pub fn widget_id(&self) -> iced::widget::text_input::Id {
        iced::widget::text_input::Id::new(self.id.to_string())
    }

    pub fn update(&mut self, cmd: Command) -> Action {
        let mut action = Action::Ignore;
        match cmd {
            Command::InitBackend(sender) => {
                self.backend = Some(
                    Backend::new(
                        self.id,
                        sender,
                        self.backend_settings.clone(),
                        self.font.measure(),
                    )
                    .unwrap_or_else(|_| {
                        panic!("init pty with ID: {} is failed", self.id);
                    }),
                );
            },
            Command::ChangeTheme(color_pallete) => {
                self.theme = Theme::new(ThemeSettings::new(color_pallete));
                action = Action::Redraw;
                self.sync_and_redraw();
            },
            Command::ChangeFont(font_settings) => {
                self.font = TermFont::new(font_settings);
                if let Some(ref mut backend) = self.backend {
                    action = backend.process_command(BackendCommand::Resize(
                        None,
                        Some(self.font.measure()),
                    ));
                    if action == Action::Redraw {
                        self.redraw();
                    }
                }
            },
            Command::AddBindings(bindings) => {
                self.bindings.add_bindings(bindings);
            },
            Command::ProcessBackendCommand(c) => {
                if let Some(ref mut backend) = self.backend {
                    action = backend.process_command(c);
                    if action == Action::Redraw {
                        self.redraw();
                    }
                }
            },
        }

        action
    }

    fn sync_and_redraw(&mut self) {
        if let Some(ref mut backend) = self.backend {
            backend.sync();
            self.redraw();
        }
    }

    fn redraw(&mut self) {
        self.cache.clear();
    }
}
