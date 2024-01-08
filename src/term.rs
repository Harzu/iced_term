use std::io::Read;
use std::num::NonZeroUsize;
use std::sync::Arc;

use crate::backend::{BackendSettings, Pty, PtyV2, RenderableCell};
use crate::{font, FontSettings};
use alacritty_terminal::tty::EventedReadWrite;
use iced::alignment::{Horizontal, Vertical};
use iced::futures::SinkExt;
use iced::mouse::{Cursor, ScrollDelta};
use iced::widget::canvas::{Cache, Path, Text};
use iced::widget::container;
use iced::{
    Color, Element, Font, Length, Point, Rectangle, Size, Subscription, Theme,
};
use iced_graphics::core::widget::Tree;
use iced_graphics::core::Widget;
use iced_graphics::geometry::Renderer;
use iced_native::subscription;
use polling::os::kqueue::{PollerKqueueExt, Signal};
use polling::PollMode;
use tokio::sync::mpsc::{self, Sender};

#[derive(Debug, Clone)]
pub enum Event {
    DataUpdated(u64, Vec<u8>),
    DataUpdated2(u64, Vec<RenderableCell>),
    InputReceived(u64, char),
    ContainerScrolled(u64, f32),
    Resized(u64, Size<f32>),
    Ignored(u64),
    TermEventTx(u64, Sender<alacritty_terminal::event::Event>),
    TermEvent(u64, alacritty_terminal::event::Event),
}

#[derive(Debug, Clone)]
pub enum Command {
    InitPty(Sender<alacritty_terminal::event::Event>),
    Focus,
    LostFocus,
    WriteToPTY(char),
    RenderData(Vec<u8>),
    RenderData2(Vec<RenderableCell>),
    Scroll(i32),
    Resize(Size<f32>),
    BackendEvent(alacritty_terminal::event::Event),
}

#[derive(Default, Clone)]
pub struct TermSettings {
    pub font: FontSettings,
    pub backend: BackendSettings,
}

pub struct Term {
    id: u64,
    font_size: f32,
    font_measure: Size<f32>,
    padding: u16,
    cache: Cache,
    is_focused: bool,
    renderable_content: Vec<RenderableCell>,
    backend: Option<PtyV2>,
    size: Size<f32>,
}

impl Term {
    pub fn new(id: u64, settings: TermSettings) -> Self {
        // let backend = Pty::new(id, settings.backend).unwrap();
        Self {
            id,
            font_size: settings.font.size,
            font_measure: font::font_measure(settings.font.size),
            padding: 0,
            is_focused: true,
            renderable_content: vec![],
            cache: Cache::default(),
            backend: None,
            size: Size {
                width: 0.0,
                height: 0.0,
            },
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn event_sub(&self) -> Subscription<Event> {
        let id = self.id();
        iced::subscription::channel(
            id.clone(),
            100,
            move |mut output| async move {
                let (event_tx, mut event_rx) = mpsc::channel(100);
                output
                    .send(Event::TermEventTx(id.clone(), event_tx))
                    .await
                    .unwrap();

                while let Some(event) = event_rx.recv().await {
                    output
                        .send(Event::TermEvent(id.clone(), event))
                        .await
                        .unwrap();
                }

                panic!("terminal event channel closed");
            },
        )
    }

    pub fn update(&mut self, cmd: Command) {
        match cmd {
            Command::InitPty(sender) => {
                self.backend = Some(
                    PtyV2::new(self.id(), sender, BackendSettings::default())
                        .unwrap(),
                )
            },
            Command::Focus => {
                self.is_focused = true;
            },
            Command::LostFocus => {
                self.is_focused = false;
            },
            Command::BackendEvent(event) => {
                if let Some(ref mut backend) = self.backend {
                    match event {
                        alacritty_terminal::event::Event::Wakeup => {
                            // println!("wakeup");
                            self.renderable_content = backend.cells();
                            self.cache.clear();
                        },
                        _ => {},
                    }
                }
            },
            Command::WriteToPTY(c) => {
                if let Some(ref mut backend) = self.backend {
                    let input = c.clone().to_string().into_bytes();
                    backend.write_to_pty(input);
                }
            },
            Command::RenderData(data) => {
                if let Some(ref mut backend) = self.backend {
                    // let content = backend.update(data);
                    // self.renderable_content = content;
                    // self.cache.clear();
                }
            },
            Command::Scroll(delta) => {
                if let Some(ref mut backend) = self.backend {
                    backend.scroll(delta);
                    self.renderable_content = backend.cells();
                    self.cache.clear();
                }
            },
            Command::Resize(size) => {
                if let Some(ref mut backend) = self.backend {
                    let container_padding =
                        f32::from(self.padding.saturating_mul(2));
                    let container_width =
                        (size.width - container_padding).max(1.0);
                    let container_height =
                        (size.height - container_padding).max(1.0);
                    let rows = (container_height / self.font_measure.height)
                        .floor() as u16;
                    let cols = (container_width / self.font_measure.width)
                        .floor() as u16;
                    backend.resize(
                        rows,
                        cols,
                        self.font_measure.width,
                        self.font_measure.height,
                    );
                    self.renderable_content = backend.cells();
                    self.cache.clear();
                    self.size = size;
                }
            },
            _ => {},
        }
    }

    pub fn view(&self) -> Element<Event> {
        container(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(self.padding)
            .style(iced::theme::Container::Custom(Box::new(Style)))
            .into()
    }

    fn handle_mouse_event(&self, event: iced::mouse::Event) -> Event {
        match event {
            iced::mouse::Event::WheelScrolled { delta } => match delta {
                ScrollDelta::Lines { x: _, y } => {
                    Event::ContainerScrolled(self.id, y)
                },
                ScrollDelta::Pixels { x: _, y } => {
                    Event::ContainerScrolled(self.id, y)
                },
            },
            _ => Event::Ignored(self.id),
        }
    }

    fn handle_keyboard_event(&self, event: iced::keyboard::Event) -> Event {
        match event {
            iced::keyboard::Event::CharacterReceived(c) => {
                Event::InputReceived(self.id, c)
            },
            _ => Event::Ignored(self.id),
        }
    }
}

#[derive(Default)]
struct Style;

impl container::StyleSheet for Style {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Color::from_rgb8(40, 39, 39).into()),
            ..container::Appearance::default()
        }
    }
}

impl Widget<Event, iced::Renderer<Theme>> for &Term {
    fn width(&self) -> Length {
        Length::Fill
    }

    fn height(&self) -> Length {
        Length::Fill
    }

    fn layout(
        &self,
        _renderer: &iced::Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        let size = limits
            .width(Length::Fill)
            .height(Length::Fill)
            .resolve(Size::ZERO);

        iced::advanced::layout::Node::new(size)
    }

    fn draw(
        &self,
        _state: &Tree,
        renderer: &mut iced::Renderer<Theme>,
        _theme: &Theme,
        _style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout,
        _cursor: Cursor,
        viewport: &Rectangle,
    ) {
        let geom = self.cache.draw(renderer, viewport.size(), |frame| {
            for cell in &self.renderable_content {
                let cell_width = self.font_measure.width as f64;
                let cell_height = self.font_measure.height as f64;

                let x = cell.column as f64 * cell_width;
                let y = (cell.line as f64 + cell.display_offset as f64)
                    * cell_height;
                let fg = font::get_color(cell.fg);
                let bg = font::get_color(cell.bg);

                let size = Size::new(cell_width as f32, cell_height as f32);
                let background = Path::rectangle(
                    Point {
                        x: x as f32 + layout.position().x,
                        y: y as f32 + layout.position().y,
                    },
                    size,
                );
                frame.fill(&background, bg);

                if cell.content != ' ' && cell.content != '\t' {
                    let text = Text {
                        content: cell.content.to_string(),
                        position: Point {
                            x: layout.position().x
                                + x as f32
                                + size.width / 2.0,
                            y: layout.position().y
                                + y as f32
                                + size.height / 2.0,
                        },
                        font: Font::default(),
                        size: self.font_size,
                        color: fg,
                        horizontal_alignment: Horizontal::Center,
                        vertical_alignment: Vertical::Center,
                        ..Text::default()
                    };

                    frame.fill_text(text);
                }
            }
        });

        renderer.draw(vec![geom]);
    }

    fn on_event(
        &mut self,
        _state: &mut Tree,
        event: iced::Event,
        _layout: iced_graphics::core::Layout<'_>,
        _cursor: Cursor,
        _renderer: &iced::Renderer<Theme>,
        _clipboard: &mut dyn iced_graphics::core::Clipboard,
        _shell: &mut iced_graphics::core::Shell<'_, Event>,
        _viewport: &Rectangle,
    ) -> iced::event::Status {
        if self.size != _layout.bounds().size() {
            _shell.publish(Event::Resized(self.id(), _layout.bounds().size()));
        }

        if !self.is_focused {
            return iced::event::Status::Ignored;
        }

        let term_event = match event {
            iced::Event::Mouse(mouse_event) => {
                self.handle_mouse_event(mouse_event)
            },
            iced::Event::Keyboard(keyboard_event) => {
                self.handle_keyboard_event(keyboard_event)
            },
            _ => Event::Ignored(self.id),
        };

        match term_event {
            Event::Ignored(_) => iced::event::Status::Ignored,
            e => {
                _shell.publish(e);
                iced::event::Status::Captured
            },
        }
    }
}

impl<'a> From<&'a Term> for Element<'a, Event, iced::Renderer<Theme>> {
    fn from(widget: &'a Term) -> Self {
        Self::new(widget)
    }
}
