// use iced::advanced::text;
use iced::{Font, Size};
// use iced_core::text::Renderer;
use iced_graphics::{geometry, text::{self, FontSystem}};
// use iced_graphics::renderer::Renderer;
// use iced_tiny_skia::{Backend, Renderer, Settings};

#[derive(Debug, Clone)]
pub struct FontSettings {
    pub size: f32,
    pub font_type: Font,
}

impl Default for FontSettings {
    fn default() -> Self {
        Self {
            size: 14.0,
            font_type: Font::MONOSPACE,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TermFont {
    size: f32,
    font_type: Font,
    measure: Size<f32>,
}

impl TermFont {
    pub fn new(settings: FontSettings) -> Self {
        Self {
            size: settings.size,
            font_type: settings.font_type,
            measure: font_measure(settings.size),
        }
    }

    pub fn size(&self) -> f32 {
        self.size
    }

    pub fn font_type(&self) -> Font {
        self.font_type
    }

    pub fn measure(&self) -> Size<f32> {
        self.measure
    }
}

fn font_measure(font_size: f32) -> Size<f32> {
    Size {
        width: 16.0,
        height: 14.0,
    }
    // text::measure(buffer)

    // let backend = Backend::new();
    // let renderer: Renderer<Backend> = Renderer::new(
    //     backend,
    //     Font::default(),
    //     iced_core::Pixels(font_size),
    // );


    // Renderer::measure(
    //     &renderer,
    //     "A",
    //     font_size,
    //     iced::widget::text::LineHeight::Relative(1.2),
    //     Font::default(),
    //     Size {
    //         width: 0.0,
    //         height: 0.0,
    //     },
    //     iced::widget::text::Shaping::Advanced,
    // )
}
