use crate::backend::{BackendSettings, Pty};
use crate::font::TermFont;
use crate::{font, FontSettings};
use alacritty_terminal::term::{cell, TermMode};
use iced::alignment::{Horizontal, Vertical};
use iced::futures::SinkExt;
use iced::keyboard::KeyCode;
use iced::mouse::{Cursor, ScrollDelta};
use iced::widget::canvas::{Cache, Path, Text};
use iced::widget::container;
use iced::{
    Color, Element, Length, Point, Rectangle, Size, Subscription, Theme,
};
use iced_graphics::core::widget::Tree;
use iced_graphics::core::Widget;
use iced_graphics::geometry::Renderer;
use tokio::sync::mpsc::{self, Sender};

#[derive(Debug, Clone)]
pub enum Event {
    Scrolled(u64, f32),
    Resized(u64, Size<f32>),
    Ignored(u64),
    InputReceived(u64, Vec<u8>),
    BackendEventSenderReceived(u64, Sender<alacritty_terminal::event::Event>),
    BackendEventReceived(u64, alacritty_terminal::event::Event),
}

#[derive(Debug, Clone)]
pub enum Command {
    InitBackend(Sender<alacritty_terminal::event::Event>),
    Focus,
    LostFocus,
    WriteToBackend(Vec<u8>),
    Scroll(i32),
    Resize(Size<f32>),
    ProcessBackendEvent(alacritty_terminal::event::Event),
}

#[derive(Default, Clone)]
pub struct TermSettings {
    pub font: FontSettings,
    pub backend: BackendSettings,
}

pub struct Term {
    id: u64,
    font: TermFont,
    padding: u16,
    cache: Cache,
    is_focused: bool,
    backend_settings: BackendSettings,
    backend: Option<Pty>,
    size: Size<f32>,
}

impl Term {
    pub fn new(id: u64, settings: TermSettings) -> Self {
        Self {
            id,
            font: TermFont::new(settings.font),
            padding: 0,
            is_focused: true,
            cache: Cache::default(),
            backend_settings: settings.backend,
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

    pub fn subscription(&self) -> Subscription<Event> {
        let id = self.id();
        iced::subscription::channel(id, 100, move |mut output| async move {
            let (event_tx, mut event_rx) = mpsc::channel(100);
            output
                .send(Event::BackendEventSenderReceived(id, event_tx))
                .await
                .unwrap();

            while let Some(event) = event_rx.recv().await {
                output
                    .send(Event::BackendEventReceived(id, event))
                    .await
                    .unwrap();
            }

            panic!("terminal event channel closed");
        })
    }

    pub fn update(&mut self, cmd: Command) {
        match cmd {
            Command::InitBackend(sender) => {
                self.backend = Some(
                    Pty::new(self.id, sender, self.backend_settings.clone())
                        .unwrap_or_else(|_| {
                            panic!("init pty with ID: {} is failed", self.id);
                        }),
                )
            },
            Command::Focus => {
                self.is_focused = true;
            },
            Command::LostFocus => {
                self.is_focused = false;
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
                    let container_padding =
                        f32::from(self.padding.saturating_mul(2));
                    let container_width =
                        (size.width - container_padding).max(1.0);
                    let container_height =
                        (size.height - container_padding).max(1.0);
                    let rows = (container_height / self.font.measure().height)
                        .floor() as u16;
                    let cols = (container_width / self.font.measure().width)
                        .floor() as u16;
                    backend.resize(
                        rows,
                        cols,
                        self.font.measure().width,
                        self.font.measure().height,
                    );
                    self.cache.clear();
                    self.size = size;
                }
            },
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
                ScrollDelta::Lines { x: _, y } => Event::Scrolled(self.id, y),
                ScrollDelta::Pixels { x: _, y } => Event::Scrolled(self.id, y),
            },
            _ => Event::Ignored(self.id),
        }
    }

    fn handle_keyboard_event(&self, event: iced::keyboard::Event) -> Event {
        match event {
            iced::keyboard::Event::CharacterReceived(c) => {
                Event::InputReceived(self.id, [c as u8].to_vec())
            },
            iced::keyboard::Event::KeyPressed {
                key_code,
                modifiers: _,
            } => {
                let mut is_app_cursor_mode = false;
                if let Some(ref backend) = self.backend {
                    is_app_cursor_mode = backend.is_mode(TermMode::APP_CURSOR);
                }

                match key_code {
                    KeyCode::Up => {
                        let code = if is_app_cursor_mode {
                            b"\x1BOA"
                        } else {
                            b"\x1B[A"
                        };
                        Event::InputReceived(self.id, code.to_vec())
                    },
                    KeyCode::Down => {
                        let code = if is_app_cursor_mode {
                            b"\x1BOB"
                        } else {
                            b"\x1B[B"
                        };
                        Event::InputReceived(self.id, code.to_vec())
                    },
                    KeyCode::Right => {
                        let code = if is_app_cursor_mode {
                            b"\x1BOC"
                        } else {
                            b"\x1B[C"
                        };
                        Event::InputReceived(self.id, code.to_vec())
                    },
                    KeyCode::Left => {
                        let code = if is_app_cursor_mode {
                            b"\x1BOD"
                        } else {
                            b"\x1B[D"
                        };
                        Event::InputReceived(self.id, code.to_vec())
                    },
                    _ => Event::Ignored(self.id),
                }
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
            if let Some(ref backend) = self.backend {
                let content = backend.renderable_content();
                for indexed in content.display_iter() {
                    let cell_width = self.font.measure().width as f64;
                    let cell_height = self.font.measure().height as f64;

                    let x = indexed.point.column.0 as f64 * cell_width;
                    let y = (indexed.point.line.0 as f64
                        + content.display_offset() as f64)
                        * cell_height;

                    let mut fg = font::get_color(indexed.fg);
                    let mut bg = font::get_color(indexed.bg);

                    if indexed.cell.flags.contains(cell::Flags::INVERSE) {
                        std::mem::swap(&mut fg, &mut bg);
                    }

                    let size = Size::new(cell_width as f32, cell_height as f32);
                    let background = Path::rectangle(
                        Point {
                            x: layout.position().x + x as f32,
                            y: layout.position().y + y as f32,
                        },
                        size,
                    );
                    frame.fill(&background, bg);

                    if content.cursor.point == indexed.point {
                        let cursor_rect = Path::rectangle(
                            Point {
                                x: layout.position().x
                                    + content.cursor.point.column.0 as f32
                                        * cell_width as f32,
                                y: layout.position().y
                                    + (content.cursor.point.line.0
                                        + content.display_offset() as i32)
                                        as f32
                                        * cell_height as f32,
                            },
                            Size::new(cell_width as f32, cell_height as f32),
                        );

                        if !backend.is_mode(TermMode::ALT_SCREEN) {
                            frame.fill(&cursor_rect, fg);
                        }
                    }

                    if indexed.c != ' ' && indexed.c != '\t' {
                        let text = Text {
                            content: indexed.c.to_string(),
                            position: Point {
                                x: layout.position().x
                                    + x as f32
                                    + size.width / 2.0,
                                y: layout.position().y
                                    + y as f32
                                    + size.height / 2.0,
                            },
                            font: self.font.font_type(),
                            size: self.font.size(),
                            color: fg,
                            horizontal_alignment: Horizontal::Center,
                            vertical_alignment: Vertical::Center,
                            ..Text::default()
                        };

                        frame.fill_text(text);
                    }
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
