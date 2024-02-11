use crate::actions::Action;
use crate::backend::BackendCommand;
use crate::backend::{settings::BackendSettings, Backend};
use crate::bindings::{Binding, BindingAction, BindingsLayout, InputKind};
use crate::font::TermFont;
use crate::theme::TermTheme;
use crate::{ColorPalette, FontSettings};
use iced::futures::SinkExt;
use iced::widget::canvas::Cache;
use iced::Subscription;
use tokio::sync::mpsc::{self, Sender};

#[derive(Debug, Clone)]
pub enum Event {
    CommandReceived(u64, Command),
}

#[derive(Debug, Clone)]
pub enum Command {
    InitBackend(Sender<alacritty_terminal::event::Event>),
    ChangeTheme(Box<ColorPalette>),
    AddBindings(Vec<(Binding<InputKind>, BindingAction)>),
    ProcessBackendCommand(BackendCommand),
}

#[derive(Default, Clone)]
pub struct TermSettings {
    pub font: FontSettings,
    pub theme: ColorPalette,
    pub backend: BackendSettings,
}

pub trait ViewProxy {
    fn id(&self) -> u64;
    fn bindings(&self) -> &BindingsLayout;
    fn cache(&self) -> &Cache;
    fn backend(&self) -> &Option<Backend>;
    fn theme(&self) -> &TermTheme;
    fn font(&self) -> &TermFont;
}

pub struct Term {
    id: u64,
    font: TermFont,
    theme: TermTheme,
    cache: Cache,
    bindings: BindingsLayout,
    backend_settings: BackendSettings,
    backend: Option<Backend>,
}

impl ViewProxy for Term {
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

    fn theme(&self) -> &TermTheme {
        &self.theme
    }
}

impl Term {
    pub fn new(id: u64, settings: TermSettings) -> Self {
        Self {
            id,
            font: TermFont::new(settings.font),
            theme: TermTheme::new(Box::new(settings.theme)),
            bindings: BindingsLayout::default(),
            cache: Cache::default(),
            backend_settings: settings.backend,
            backend: None,
        }
    }

    pub fn widget_id(&self) -> iced::widget::text_input::Id {
        iced::widget::text_input::Id::new(self.id.to_string())
    }

    pub fn subscription(&self) -> Subscription<Event> {
        let id = self.id;
        iced::subscription::channel(id, 100, move |mut output| async move {
            let (event_tx, mut event_rx) = mpsc::channel(100);
            let cmd = Command::InitBackend(event_tx);
            output
                .send(Event::CommandReceived(id, cmd))
                .await
                .unwrap_or_else(|_| {
                    panic!("iced_term subscription {}: sending BackendEventSenderReceived event is failed", id)
                });

            while let Some(event) = event_rx.recv().await {
                let cmd = Command::ProcessBackendCommand(
                    BackendCommand::ProcessAlacrittyEvent(event),
                );
                output
                    .send(Event::CommandReceived(id, cmd))
                    .await
                    .unwrap_or_else(|_| {
                        panic!("iced_term subscription {}: sending BackendEventReceived event is failed", id)
                    });
            }

            panic!("iced_term subscription {}: terminal event channel closed unexpected", id);
        })
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
            Command::ChangeTheme(palette) => {
                if let Some(ref mut backend) = self.backend {
                    self.theme = TermTheme::new(palette);
                    backend.sync();
                    self.cache.clear();
                }
            },
            Command::AddBindings(bindings) => {
                self.bindings.add_bindings(bindings);
            },
            Command::ProcessBackendCommand(c) => {
                if let Some(ref mut backend) = self.backend {
                    match backend.process_command(c) {
                        Action::Redraw => self.cache.clear(),
                        Action::Shutdown => {
                            action = Action::Shutdown;
                        },
                        _ => {},
                    }
                }
            },
        }

        action
    }
}
