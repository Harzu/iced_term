use std::fs::File;
use std::io::Result;
use iced::widget::{Canvas, canvas};
use iced::{Element, Font, Length, Point, Rectangle, Size, Theme, Subscription};
use iced::mouse::Cursor;
use iced::widget::canvas::{Cache, Geometry};
use iced::widget::canvas::{Path, Text};
// use iced::renderer::Renderer;
use tokio::time::sleep;
use crate::backend::{self, RenderableCell};
use crate::font;
use ab_glyph;

#[derive(Debug, Clone)]
pub enum Message {
    DataUpdated(u64, Vec<u8>),
    CharacterReceived(u64, char),
    Ignored(u64),
}

pub fn iterm(id: u64) -> Result<(backend::Pty, ITermView)> {
    let pty = backend::Pty::new(id, backend::Settings::default())?;
    Ok((
        pty,
        ITermView::new(id),
    ))
}

pub struct ITermView {
    pub id: u64,
    cache: Cache,
    renderable_content: Vec<RenderableCell>
}

impl ITermView
{
    fn new(id: u64) -> Self {
        Self {
            id,
            renderable_content: vec![],
            cache: Cache::default(),
        }
    }
}

impl ITermView {
    pub fn update(&mut self, content: Vec<RenderableCell>) {
        self.renderable_content = content;
        self.request_redraw();
    }

    pub fn request_redraw(&self) {
        self.cache.clear();
    }

    pub fn view(&self) -> Element<Message> {
        Canvas::new(self)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }

    pub fn on_data_received(id: u64, reader: File) -> Subscription<Message> {
        iced::subscription::unfold(id, reader, move |reader| async move {
            sleep(std::time::Duration::from_millis(1)).await;
            match backend::Pty::try_read(&reader).await {
                Some(data) => (Message::DataUpdated(id, data), reader),
                None => (Message::Ignored(id), reader)
            }
        })
    }
}

impl canvas::Program<Message> for ITermView
{
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: canvas::Event,
        _bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        match event {
            canvas::Event::Keyboard(e) => match e {
                iced::keyboard::Event::CharacterReceived(c) => {
                    (canvas::event::Status::Captured, Some(Message::CharacterReceived(self.id, c)))
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
            let cell_width = 15.0;
            let cell_height = 20.0;

            for cell in &self.renderable_content {
                let x = cell.column as f64 * cell_width;
                let y = (cell.line as f64 + cell.display_offset as f64) * cell_height;
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
                            x: x as f32,
                            y: y as f32,
                        },
                        font: Font::default(),
                        size: 20.0,
                        color: fg,
                        ..Text::default()
                    };

                    frame.fill_text(text);
                }
            }
        });

        vec![geom]
    }
}
