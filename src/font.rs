use iced::{Font, Size};
use iced_core::text::LineHeight;
use iced_graphics::text::{
    self,
    cosmic_text::{Metrics, Shaping, Wrap},
};

#[derive(Debug, Clone)]
pub struct FontSettings {
    pub size: f32,
    pub scale_factor: f32,
    pub font_type: Font,
}

impl Default for FontSettings {
    fn default() -> Self {
        Self {
            size: 14.0,
            scale_factor: 1.3,
            font_type: Font::MONOSPACE,
        }
    }
}

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
    let metrics = Metrics::new(
        font_size,
        LineHeight::Relative(scale_factor)
            .to_absolute(iced_core::Pixels(font_size))
            .into(),
    );
    let mut buffer = text::cosmic_text::Buffer::new_empty(metrics);
    let attrs = text::to_attributes(font_type);

    let (width, height) = {
        let mut font_system = text::font_system().write().unwrap();
        let font_system = font_system.raw();
        buffer.set_wrap(font_system, Wrap::None);

        // Use size of space to determine cell size
        buffer.set_text(font_system, "A", attrs, Shaping::Advanced);
        let layout = buffer.line_layout(font_system, 0).unwrap();
        let w = layout[0].w;
        (w, metrics.line_height)
    };

    Size { width, height }
}
