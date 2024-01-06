use std::fs::File;
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{canvas, container};
use iced::{Element, Font, Length, Point, Rectangle, Size, Theme, Color, Subscription};
use iced::mouse::{Cursor, ScrollDelta};
use iced::widget::canvas::{Cache, Geometry, Path, Text};
use iced_graphics::core::Widget;
use iced_graphics::core::widget::Tree;
use iced_graphics::geometry::Renderer;
use crate::font;
use crate::backend::{Pty, RenderableCell, BackendSettings};

#[derive(Debug, Clone)]
pub enum Event {
    DataUpdated(u64, Vec<u8>),
    InputReceived(u64, char),
    ContainerScrolled(u64, (f32, f32)),
    Resized(u64, Size<f32>),
    Ignored(u64)
}

#[derive(Debug, Clone)]
pub enum Command {
    Focus,
    LostFocus,
    WriteToPTY(char),
    RenderData(Vec<u8>),
    Scroll(i32),
    Resize(Size<f32>),
}

pub struct Term {
    id: u64,
    font_size: f32,
    font_measure: Size<f32>,
    padding: u16,
    cache: Cache,
    is_focused: bool,
    renderable_content: Vec<RenderableCell>,
    backend: Pty,
    size: Size<f32>,
}


pub fn data_received_subscription(id: u64, reader: File) -> Subscription<Event> {
    iced::subscription::unfold(format!("iced_term_{}", id), reader, move |reader| async move {
        match Pty::read(&reader).await {
            Some(data) => (Event::DataUpdated(id, data), reader),
            None => (Event::Ignored(id), reader)
        }
    })
}

impl Term {
    pub fn new(id: u64, font_size: f32) -> Self {
        Self {
            id,
            font_size,
            font_measure: font::font_measure(font_size),
            padding: 0,
            is_focused: true,
            renderable_content: vec![],
            cache: Cache::default(),
            backend: Pty::new(id, BackendSettings::default()).unwrap(),
            size: Size { width: 0.0, height: 0.0 },
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn pty_data_reader(&self) -> File {
        self.backend.reader()
    }

    pub fn font_size(&self) -> f32 {
        self.font_size
    }

    pub fn font_measure(&self) -> Size<f32> {
        self.font_measure
    }

    pub fn inner_padding(&self) -> u16 {
        self.padding
    }

    pub fn update(&mut self, cmd: Command) {
        match cmd {
            Command::Focus => {
                self.is_focused = true;
            }
            Command::LostFocus => {
                self.is_focused = false;
            },
            Command::WriteToPTY(c) => {
                self.backend.write_to_pty(c);
            },
            Command::RenderData(data) => {
                let content = self.backend.update(data);
                self.renderable_content = content;
                self.cache.clear();
            },
            Command::Scroll(delta) => {
                let content = self.backend.scroll(delta);
                self.renderable_content = content;
                self.cache.clear();
            },
            Command::Resize(size) => {
                let container_padding = f32::from(self.padding.saturating_mul(2));
                let container_width = (size.width - container_padding).max(1.0);
                let container_height = (size.height - container_padding).max(1.0);;
                let rows = (container_height / self.font_measure.height).floor() as u16;
                let cols = (container_width / self.font_measure.width).floor() as u16;
                let content = self.backend.resize(
                    rows,
                    cols,
                    self.font_measure.width,
                    self.font_measure.height,
                );
                self.renderable_content = content;
                self.cache.clear();
                self.size = size;
            }
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
}

#[derive(Default)]
struct Style;

impl container::StyleSheet for Style {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Color::from_rgb8(40, 39, 39).into()), // Set the background color here
            ..container::Appearance::default()
        }   
    }
}


impl canvas::Program<Event> for Term
{
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: canvas::Event,
        _bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> (canvas::event::Status, Option<Event>) {
        if self.is_focused {
            return match event {
                canvas::Event::Keyboard(e) => match e {
                    iced::keyboard::Event::CharacterReceived(c) => (canvas::event::Status::Captured, Some(Event::InputReceived(self.id(), c))),
                    _ => (canvas::event::Status::Ignored, None)
                }
                canvas::Event::Mouse(e) => match e {
                    iced::mouse::Event::WheelScrolled { delta } => {
                        if let ScrollDelta::Lines{ x, y } = delta {
                            return (canvas::event::Status::Captured, Some(Event::ContainerScrolled(self.id(), (x, y))))
                        }

                        (canvas::event::Status::Ignored, None)
                    },
                    _ => (canvas::event::Status::Ignored, None)
                }
                _ => (canvas::event::Status::Ignored, None)
            }
        }

        (canvas::event::Status::Ignored, None)
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        let geom = self.cache.draw(renderer, bounds.size(), |frame| {
            for cell in &self.renderable_content {
                let cell_width = self.font_measure.width as f64;
                let cell_height = self.font_measure.height as f64;
                
                let x = cell.column as f64 * cell_width as f64;
                let y = (cell.line as f64 + cell.display_offset as f64) * cell_height as f64;
                let fg = font::get_color(cell.fg);
                let bg = font::get_color(cell.bg);

                let size = Size::new(cell_width as f32, cell_height as f32);
                let background = Path::rectangle(
                    Point {
                        x: x as f32,
                        y: y as f32,
                    },
                    size,
                );
                frame.fill(&background, bg);

                if cell.content != ' ' && cell.content != '\t' {
                    let text = Text {
                        content: cell.content.to_string(),
                        position: Point {
                            x: x as f32 + size.width / 2.0,
                            y: y as f32 + size.height / 2.0,
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

        vec![geom]
    }
}

impl Widget<Event, iced::Renderer<Theme>> for &Term
{
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

    fn on_event(
        &mut self,
        _state: &mut Tree,
        event: iced::Event,
        _layout: iced_graphics::core::Layout<'_>,
        _cursor: iced_graphics::core::mouse::Cursor,
        _renderer: &iced::Renderer<Theme>,
        _clipboard: &mut dyn iced_graphics::core::Clipboard,
        _shell: &mut iced_graphics::core::Shell<'_, Event>,
        _viewport: &Rectangle,
    ) -> iced::event::Status {
        if self.size != _layout.bounds().size() {
            _shell.publish(Event::Resized(self.id(), _layout.bounds().size()));
        }
        
        if self.is_focused {
            return match event {
                iced::Event::Keyboard(e) => match e {
                    iced::keyboard::Event::CharacterReceived(c) => {
                        _shell.publish(Event::InputReceived(self.id(), c));
                        canvas::event::Status::Captured
                    },
                    _ => canvas::event::Status::Captured
                }
                iced::Event::Mouse(e) => match e {
                    iced::mouse::Event::WheelScrolled { delta } => {
                        if let ScrollDelta::Lines{ x, y } = delta {
                            _shell.publish(Event::ContainerScrolled(self.id(), (x, y)));
                            return iced::event::Status::Ignored
                        }

                        iced::event::Status::Ignored
                    },
                    _ => iced::event::Status::Ignored
                }
                _ => iced::event::Status::Ignored
            }
        }

        iced::event::Status::Ignored
    }

    fn draw(
        &self,
        _state: &iced::advanced::widget::Tree,
        renderer: &mut iced::Renderer<Theme>,
        _theme: &Theme,
        _style: &iced::advanced::renderer::Style,
        _layout: iced::advanced::Layout,
        _cursor: Cursor,
        viewport: &Rectangle,
    ) {
        let geom = self.cache.draw(renderer, viewport.size(), |frame| {
            for cell in &self.renderable_content {
                let cell_width = self.font_measure.width as f64;
                let cell_height = self.font_measure.height as f64;
                
                let x = cell.column as f64 * cell_width as f64;
                let y = (cell.line as f64 + cell.display_offset as f64) * cell_height as f64;
                let fg = font::get_color(cell.fg);
                let bg = font::get_color(cell.bg);

                let size = Size::new(cell_width as f32, cell_height as f32);
                let background = Path::rectangle(
                    Point {
                        x: x as f32 + _layout.position().x,
                        y: y as f32 + _layout.position().y,
                    },
                    size,
                );
                frame.fill(&background, bg);

                if cell.content != ' ' && cell.content != '\t' {
                    let text = Text {
                        content: cell.content.to_string(),
                        position: Point {
                            x: _layout.position().x + x as f32 + size.width / 2.0,
                            y:  _layout.position().y + y as f32 + size.height / 2.0,
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
}

impl<'a> From<&'a Term> for Element<'a, Event, iced::Renderer<Theme>>
{
    fn from(widget: &'a Term) -> Self {
        Self::new(widget)
    }
}
