use std::io::Result;
// use iced::advanced::graphics::{geometry};
use iced::advanced::widget;
use iced::advanced::layout::Layout;
use iced::advanced::renderer;
use iced::{Element, Font, Length, Point, Rectangle, Size, Theme, Subscription};
use iced::mouse::Cursor;
use iced::advanced::Widget;
use iced::widget::{Canvas, canvas::Cache};
use iced::widget::canvas::{Path, Text, Renderer};
use tokio::time::sleep;
use crate::backend::{self, RenderableCell};
use crate::font;

pub struct ITerm {
    id: u64,
    pty: backend::Pty,
    view: ITermView
}

#[derive(Debug, Clone)]
pub enum Message {
    DataUpdated((u64, Vec<u8>)),
    Ignored,
}

pub enum Command {
    Redraw
}

impl ITerm {
    pub fn new(id: u64) -> Result<Self> {
        Ok(Self {
            id,
            pty: backend::Pty::new(id, backend::Settings::default())?,
            view: ITermView::default(),
        })
    }

    pub fn subscribe(&mut self) -> Subscription<Message> {
        let reader = self.pty.new_reader();
        let id = self.id.clone();
        iced::subscription::unfold(id, reader, move |reader| async move {
            sleep(std::time::Duration::from_millis(1)).await;
            match backend::Pty::try_read(&reader).await {
                Some(data) => (Message::DataUpdated((id, data)), reader),
                None => (Message::Ignored, reader)
            }
        })
    }
}

#[derive(Default)]
struct ITermView {
    cache: Cache,
    renderable_content: Vec<RenderableCell>
}

impl ITermView {
    fn update(&mut self, content: Vec<RenderableCell>) {
        self.renderable_content = content;
        self.cache.clear();
    }
}

impl<Message> Widget<Message, iced::Renderer<Theme>> for ITermView
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

    fn draw(
        &self,
        _state: &widget::Tree,
        renderer: &mut iced::Renderer<Theme>,
        _theme: &Theme,
        _style: &renderer::Style,
        _layout: Layout,
        _cursor: Cursor,
        viewport: &Rectangle,
    ) {
        let geom = self.cache.draw(renderer, viewport.size(), |frame| {
            let cell_width = 13.0;
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

        renderer.draw(vec![geom]);
    }
}

impl<'a, Message> From<ITermView> for Element<'a, Message, iced::Renderer<Theme>>
{
    fn from(widget: ITermView) -> Self {
        Self::new(widget)
    }
}

// impl<Message> canvas::Program<Message> for ITerm {
//     type State = ();

//     fn draw(
//         &self,
//         _state: &Self::State,
//         renderer: &Renderer,
//         _theme: &Theme,
//         bounds: Rectangle,
//         _cursor: Cursor,
//     ) -> Vec<Geometry> {

//         let geom = self.cache.draw(renderer, bounds.size(), |frame| {
//             let cell_width = 13.0;
//             let cell_height = 20.0;

//             for cell in self.pty.cells() {
//                 let x = cell.column as f64 * cell_width;
//                 let y = (cell.line as f64 + cell.display_offset as f64) * cell_height;
//                 let fg = font::get_color(cell.fg);
//                 let bg = font::get_color(cell.bg);

//                 let size = Size::new(cell_width as f32, cell_height as f32);
//                 let background = Path::rectangle(
//                     Point {
//                         x: x as f32,
//                         y: y as f32,
//                     },
//                     size,
//                 );
//                 frame.fill(&background, bg);

//                 if cell.content != ' ' && cell.content != '\t' {
//                     let text = Text {
//                         content: cell.content.to_string(),
//                         position: Point {
//                             x: x as f32,
//                             y: y as f32,
//                         },
//                         font: Font::default(),
//                         size: 20.0,
//                         color: fg,
//                         ..Text::default()
//                     };

//                     frame.fill_text(text);
//                 }
//             }
//         });

//         vec![geom]
//     }
// }
