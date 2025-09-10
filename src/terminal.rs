use std::io::Result;
use crate::actions::Action;
use crate::backend::{Backend, BackendCommand};
use crate::bindings::{Binding, BindingAction, BindingsLayout, InputKind};
use crate::font::TermFont;
use crate::settings::{BackendSettings, FontSettings, Settings, ThemeSettings};
use crate::theme::{ColorPalette, Theme};
use crate::{AlacrittyEvent, Subscription};
use iced::widget::canvas::Cache;
use crossbeam_channel::{unbounded, Receiver};

#[derive(Debug, Clone)]
pub enum Event {
    CommandReceived(u64, Command),
}

#[derive(Debug, Clone)]
pub enum Command {
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
    pub(crate) backend: Backend,
    backend_event_rx: Receiver<AlacrittyEvent>,
}

impl Terminal {
    pub fn new(id: u64, settings: Settings) -> Result<Self> {
        let (backend_event_tx, backend_event_rx) = unbounded();
        let theme = Theme::new(settings.theme);
        let font = TermFont::new(settings.font);
        let measure = font.measure.clone();

        Ok(Self {
            id,
            font: font,
            theme: theme,
            bindings: BindingsLayout::default(),
            cache: Cache::default(),
            backend: Backend::new(
                id,
                backend_event_tx,
                settings.backend,
                measure,
            )?,
            backend_event_rx,
        })
    }

    pub fn backend_event_rx(&self) -> Receiver<AlacrittyEvent> {
        self.backend_event_rx.clone()
    }

    pub fn widget_id(&self) -> iced::widget::text_input::Id {
        iced::widget::text_input::Id::new(self.id.to_string())
    }

    pub fn handle(&mut self, cmd: Command) -> Action {
        let action = Action::Ignore;

        println!("terminal {:?}",cmd);


        match cmd {
            Command::ChangeTheme(color_pallete) => {
                self.theme = Theme::new(ThemeSettings::new(color_pallete));
                self.sync_and_redraw();
            },
            Command::ChangeFont(font_settings) => {
                self.font = TermFont::new(font_settings);
                self.sync_font_size();
            },
            Command::AddBindings(bindings) => {
                self.bindings.add_bindings(bindings);
            },
            Command::ProcessBackendCommand(c) => {
                if self.backend.handle(c) == Action::Redraw {
                    self.redraw();
                }
            },
            _ => {},
        }

        action
    }

    fn sync_font_size(&mut self) -> Action {
        // if let Some(ref mut backend) = self.backend {
        self.backend.handle(BackendCommand::Resize(
            None,
            Some(self.font.measure),
        ))
    }

    fn sync_and_redraw(&mut self) {
        // if let Some(ref mut backend) = self.backend {
        self.backend.sync();
        self.redraw();
        // }
    }

    fn redraw(&mut self) {
        self.cache.clear();
    }
}
