use crate::actions::Action;
use crate::backend::{Backend, BackendCommand};
use crate::bindings::{Binding, BindingAction, BindingsLayout, InputKind};
use crate::font::TermFont;
use crate::settings::{BackendSettings, FontSettings, Settings, ThemeSettings};
use crate::theme::{ColorPalette, Theme};
use crate::AlacrittyEvent;
use iced::widget::canvas::Cache;
use tokio::sync::mpsc::Sender;

#[derive(Debug, Clone)]
pub enum Event {
    CommandReceived(u64, Command),
}

#[derive(Debug, Clone)]
pub enum Command {
    InitBackend(Sender<AlacrittyEvent>),
    ChangeTheme(Box<ColorPalette>),
    ChangeFont(FontSettings),
    AddBindings(Vec<(Binding<InputKind>, BindingAction)>),
    ProcessBackendCommand(BackendCommand),
}

pub struct Terminal {
    pub id: u64,
    pub(crate) font: TermFont,
    pub(crate) theme: Theme,
    pub(crate) cache: Cache,
    pub(crate) bindings: BindingsLayout,
    pub(crate) backend: Option<Backend>,
    backend_settings: BackendSettings,
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
                        self.font.measure,
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
                        Some(self.font.measure),
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
                    if action == Action::Redraw || action == Action::Shutdown {
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
