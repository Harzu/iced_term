use iced::advanced::text;
use iced::{Font, Size, Theme};
use iced_graphics::renderer::Renderer;
use iced_tiny_skia::{Backend, Settings};

#[derive(Clone)]
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
    let backend = Backend::new(Settings {
        default_font: Font::default(),
        default_text_size: font_size,
    });

    let renderer: Renderer<Backend, Theme> = Renderer::new(backend);
    text::Renderer::measure(
        &renderer,
        "A",
        font_size,
        iced::widget::text::LineHeight::Relative(1.2),
        Font::default(),
        Size {
            width: 0.0,
            height: 0.0,
        },
        iced::widget::text::Shaping::Advanced,
    )
}
