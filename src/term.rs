use std::fs::File;
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{Canvas, canvas, container};
use iced::{Element, Font, Length, Point, Rectangle, Size, Theme, Color, Subscription};
use iced::mouse::Cursor;
use iced::widget::canvas::{Cache, Geometry};
use iced::widget::canvas::{Path, Text};
use tokio::time::sleep;
use crate::font;
use crate::backend::{Pty, RenderableCell};

#[derive(Debug, Clone)]
pub enum Event {
    DataUpdated(u64, Vec<u8>),
    InputReceived(u64, char),
    Ignored(u64),
}

pub struct Term {
    id: u64,
    font_size: f32,
    font_measure: Size<f32>,
    padding: u16,
    cache: Cache,
    renderable_content: Vec<RenderableCell>
}

pub fn data_received_subscription(id: u64, reader: File) -> Subscription<Event> {
    iced::subscription::unfold(id, reader, move |reader| async move {
        sleep(std::time::Duration::from_millis(1)).await;
        match Pty::try_read(&reader).await {
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
            renderable_content: vec![],
            cache: Cache::default(),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
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

    pub fn update_and_redraw(&mut self, content: Vec<RenderableCell>) {
        self.renderable_content = content;
        self.request_redraw();
    }

    pub fn request_redraw(&self) {
        self.cache.clear();
    }

    pub fn view(&self) -> Element<Event> {
        let canvas = Canvas::new(self)
            .height(Length::Fill)
            .width(Length::Fill);

        container(canvas)
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
        match event {
            canvas::Event::Keyboard(e) => match e {
                iced::keyboard::Event::CharacterReceived(c) => {
                    (canvas::event::Status::Captured, Some(Event::InputReceived(self.id(), c)))
                },
                _ => (canvas::event::Status::Ignored, None)
            }
            _ => (canvas::event::Status::Ignored, None)
        }
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
