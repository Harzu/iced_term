use crate::actions::Action;
use crate::backend::BackendCommand;
use crate::backend::{settings::BackendSettings, Backend};
use crate::bindings::{Binding, BindingAction, BindingsLayout, InputKind};
use crate::font::TermFont;
use crate::theme::TermTheme;
use crate::{ColorPalette, FontSettings};
use alacritty_terminal::event::Event as AlacrittyEvent;
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
    ChangeFont(FontSettings),
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

            let mut shutdown = false;
            loop {
                match event_rx.recv().await {
                    Some(event) => {
                        match event {
                            AlacrittyEvent::Exit => shutdown = true,
                            _ => {},
                        };

                        let cmd = Command::ProcessBackendCommand(
                            BackendCommand::ProcessAlacrittyEvent(event),
                        );
                        output
                            .send(Event::CommandReceived(id, cmd))
                            .await
                            .unwrap_or_else(|_| {
                                panic!("iced_term subscription {}: sending BackendEventReceived event is failed", id)
                            });
                    },
                    None => {
                        if !shutdown {
                            panic!("iced_term subscription {}: terminal event channel closed unexpected", id);
                        }
                    },
                }
            }
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
                self.theme = TermTheme::new(palette);
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
