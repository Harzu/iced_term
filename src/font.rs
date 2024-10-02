use crate::settings::FontSettings;
use iced::{Font, Size};
use iced_core::{
    alignment::{Horizontal, Vertical},
    text::{LineHeight, Paragraph, Shaping as TextShaping},
    Text,
};
use iced_graphics::text::paragraph;

#[derive(Debug, Clone)]
pub struct TermFont {
    size: f32,
    font_type: Font,
    scale_factor: f32,
    measure: Size<f32>,
}

impl TermFont {
    pub fn new(settings: FontSettings) -> Self {
        Self {
            size: settings.size,
            font_type: settings.font_type,
            scale_factor: settings.scale_factor,
            measure: font_measure(
                settings.size,
                settings.scale_factor,
                settings.font_type,
            ),
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

    pub fn scale_factor(&self) -> f32 {
        self.scale_factor
    }
}

fn font_measure(
    font_size: f32,
    scale_factor: f32,
    font_type: Font,
) -> Size<f32> {
    let paragraph = paragraph::Paragraph::with_text(Text {
        content: "m",
        font: font_type,
        size: iced_core::Pixels(font_size),
        vertical_alignment: Vertical::Center,
        horizontal_alignment: Horizontal::Center,
        shaping: TextShaping::Advanced,
        line_height: LineHeight::Relative(scale_factor),
        bounds: Size::INFINITY,
        wrapping: iced_core::text::Wrapping::Glyph,
    });

    paragraph.min_bounds()
}
