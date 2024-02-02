use crate::backend::{settings::BackendSettings, Backend};
use crate::bindings::{Binding, BindingAction, BindingsLayout, InputKind};
use crate::font::TermFont;
use crate::theme::TermTheme;
use crate::{ColorPalette, FontSettings};
use alacritty_terminal::selection::SelectionType;
use iced::futures::SinkExt;
use iced::widget::canvas::Cache;
use iced::{Size, Subscription};
use tokio::sync::mpsc::{self, Sender};

#[derive(Debug, Clone)]
pub enum Event {
    Scrolled(u64, f32),
    Resized(u64, Size<f32>),
    Ignored(u64),
    InputReceived(u64, Vec<u8>),
    SelectStarted(u64, SelectionType, (f32, f32)),
    SelectUpdated(u64, (f32, f32)),
    BackendEventSenderReceived(u64, Sender<alacritty_terminal::event::Event>),
    BackendEventReceived(u64, alacritty_terminal::event::Event),
}

#[derive(Debug, Clone)]
pub enum Command {
    InitBackend(Sender<alacritty_terminal::event::Event>),
    WriteToBackend(Vec<u8>),
    Scroll(i32),
    ChangeTheme(Box<ColorPalette>),
    AddBindings(Vec<(Binding<InputKind>, BindingAction)>),
    Resize(Size<f32>),
    SelectStart(SelectionType, (f32, f32)),
    SelectUpdate((f32, f32)),
    ProcessBackendEvent(alacritty_terminal::event::Event),
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
            output
                .send(Event::BackendEventSenderReceived(id, event_tx))
                .await
                .unwrap_or_else(|_| {
                    panic!("ICED SUBSCRIPTION {}: sending BackendEventSenderReceived event is failed", id)
                });

            while let Some(event) = event_rx.recv().await {
                output
                    .send(Event::BackendEventReceived(id, event))
                    .await
                    .unwrap_or_else(|_| {
                        panic!("ICED SUBSCRIPTION {}: sending BackendEventReceived event is failed", id)
                    });
            }

            panic!("ICED SUBSCRIPTION {}: terminal event channel closed unexpected", id);
        })
    }

    pub fn update(&mut self, cmd: Command) {
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
            Command::ProcessBackendEvent(event) => {
                if let alacritty_terminal::event::Event::Wakeup = event {
                    self.cache.clear();
                }
            },
            Command::WriteToBackend(input) => {
                if let Some(ref mut backend) = self.backend {
                    backend.write_to_pty(input);
                }
            },
            Command::Scroll(delta) => {
                if let Some(ref mut backend) = self.backend {
                    backend.scroll(delta);
                    self.cache.clear();
                }
            },
            Command::Resize(size) => {
                if let Some(ref mut backend) = self.backend {
                    let font_size = self.font.measure();
                    backend.resize(
                        size.width,
                        size.height,
                        font_size.width as u16,
                        font_size.height as u16,
                    );
                    self.cache.clear();
                }
            },
            Command::ChangeTheme(palette) => {
                self.theme = TermTheme::new(palette);
                self.cache.clear();
            },
            Command::AddBindings(bindings) => {
                self.bindings.add_bindings(bindings);
            },
            Command::SelectStart(selection_type, (x, y)) => {
                if let Some(ref mut backend) = self.backend {
                    backend.start_selection(selection_type, x, y);
                    self.cache.clear();
                }
            },
            Command::SelectUpdate((x, y)) => {
                if let Some(ref mut backend) = self.backend {
                    backend.update_selection(x, y);
                    self.cache.clear();
                }
            },
        }
    }
}
