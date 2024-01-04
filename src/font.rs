use alacritty_terminal::vte::ansi::NamedColor;
use iced::Color;

pub const FONT_PATH: &str = "../fonts/Hack Regular Nerd Font Complete.ttf";

pub fn get_color(c: alacritty_terminal::vte::ansi::Color) -> Color {
    match c {
        alacritty_terminal::vte::ansi::Color::Spec(rgb) => Color::from_rgb8(rgb.r, rgb.g, rgb.b),
        alacritty_terminal::vte::ansi::Color::Named(c) => match c {
            NamedColor::Foreground => Color::from_rgb8(235, 218, 177),
            NamedColor::Background => Color::from_rgb8(40, 39, 39),
            NamedColor::Green => Color::from_rgb8(152, 150, 26),
            NamedColor::Red => Color::from_rgb8(203, 35, 29),
            NamedColor::Yellow => Color::from_rgb8(214, 152, 33),
            NamedColor::Blue => Color::from_rgb8(69, 132, 135),
            NamedColor::Cyan => Color::from_rgb8(104, 156, 105),
            NamedColor::Magenta => Color::from_rgb8(176, 97, 133),
            NamedColor::White => Color::from_rgb8(168, 152, 131),
            NamedColor::Black => Color::from_rgb8(40, 39, 39),
            NamedColor::BrightBlack => Color::from_rgb8(146, 130, 115),
            NamedColor::BrightRed => Color::from_rgb8(250, 72, 52),
            NamedColor::BrightGreen => Color::from_rgb8(184, 186, 38),
            NamedColor::BrightYellow => Color::from_rgb8(249, 188, 47),
            NamedColor::BrightBlue => Color::from_rgb8(131, 164, 151),
            NamedColor::BrightMagenta => Color::from_rgb8(210, 133, 154),
            NamedColor::BrightCyan => Color::from_rgb8(142, 191, 123),
            NamedColor::BrightWhite => Color::from_rgb8(235, 218, 177),
            NamedColor::BrightForeground => Color::from_rgb8(235, 218, 177),
            _ => Color::from_rgb8(40, 39, 39),
        },
        _ => Color::from_rgb8(40, 39, 39),
    }
}
