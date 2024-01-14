use alacritty_terminal::vte::ansi::{self, NamedColor};
use iced::Color;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TermTheme {
    ansi256_colors: HashMap<u8, Color>,
}

impl TermTheme {
    pub fn new() -> Self {
        Self {
            ansi256_colors: TermTheme::get_ansi256_colors(),
        }
    }

    fn get_ansi256_colors() -> HashMap<u8, Color> {
        let mut ansi256_colors = HashMap::new();

        for r in 0..6 {
            for g in 0..6 {
                for b in 0..6 {
                    // Reserve the first 16 colors for config.
                    let index = 16 + r * 36 + g * 6 + b;
                    let color = Color::from_rgb8(
                        if r == 0 { 0 } else { r * 40 + 55 },
                        if g == 0 { 0 } else { g * 40 + 55 },
                        if b == 0 { 0 } else { b * 40 + 55 },
                    );
                    ansi256_colors.insert(index, color);
                }
            }
        }

        let index: u8 = 232;
        for i in 0..24 {
            let value = i * 10 + 8;
            ansi256_colors
                .insert(index + i, Color::from_rgb8(value, value, value));
        }

        ansi256_colors
    }

    pub fn get_color(&self, c: ansi::Color) -> Color {
        match c {
            ansi::Color::Spec(rgb) => Color::from_rgb8(rgb.r, rgb.g, rgb.b),
            ansi::Color::Indexed(index) => {
                if index <= 15 {
                    return match index {
                        // Default terminal reserved colors
                        0 => Color::from_rgb8(40, 39, 39),
                        1 => Color::from_rgb8(203, 35, 29),
                        2 => Color::from_rgb8(152, 150, 26),
                        3 => Color::from_rgb8(214, 152, 33),
                        4 => Color::from_rgb8(69, 132, 135),
                        5 => Color::from_rgb8(176, 97, 133),
                        6 => Color::from_rgb8(104, 156, 105),
                        7 => Color::from_rgb8(168, 152, 131),
                        // Bright terminal reserved colors
                        8 => Color::from_rgb8(146, 130, 115),
                        9 => Color::from_rgb8(250, 72, 52),
                        10 => Color::from_rgb8(184, 186, 38),
                        11 => Color::from_rgb8(249, 188, 47),
                        12 => Color::from_rgb8(131, 164, 151),
                        13 => Color::from_rgb8(210, 133, 154),
                        14 => Color::from_rgb8(142, 191, 123),
                        15 => Color::from_rgb8(235, 218, 177),
                        _ => Color::from_rgb8(0, 0, 0),
                    };
                }

                // Other colors
                match self.ansi256_colors.get(&index) {
                    Some(color) => *color,
                    None => Color::from_rgb8(0, 0, 0),
                }
            },
            ansi::Color::Named(c) => match c {
                NamedColor::Foreground => Color::from_rgb8(235, 218, 177),
                NamedColor::Background => Color::from_rgb8(40, 39, 39),
                // Default terminal reserved colors
                NamedColor::Black => Color::from_rgb8(40, 39, 39),
                NamedColor::Red => Color::from_rgb8(203, 35, 29),
                NamedColor::Green => Color::from_rgb8(152, 150, 26),
                NamedColor::Yellow => Color::from_rgb8(214, 152, 33),
                NamedColor::Blue => Color::from_rgb8(69, 132, 135),
                NamedColor::Magenta => Color::from_rgb8(176, 97, 133),
                NamedColor::Cyan => Color::from_rgb8(104, 156, 105),
                NamedColor::White => Color::from_rgb8(168, 152, 131),
                // Bright terminal reserved colors
                NamedColor::BrightBlack => Color::from_rgb8(146, 130, 115),
                NamedColor::BrightRed => Color::from_rgb8(250, 72, 52),
                NamedColor::BrightGreen => Color::from_rgb8(184, 186, 38),
                NamedColor::BrightYellow => Color::from_rgb8(249, 188, 47),
                NamedColor::BrightBlue => Color::from_rgb8(131, 164, 151),
                NamedColor::BrightMagenta => Color::from_rgb8(210, 133, 154),
                NamedColor::BrightCyan => Color::from_rgb8(142, 191, 123),
                NamedColor::BrightWhite => Color::from_rgb8(235, 218, 177),
                NamedColor::BrightForeground => Color::from_rgb8(235, 218, 177),
                _ => Color::from_rgb8(0, 0, 0),
            },
        }
    }
}
