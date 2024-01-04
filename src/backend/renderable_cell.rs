use alacritty_terminal::vte::ansi;

#[derive(Clone, Debug)]
pub struct RenderableCell {
    pub column: usize,
    pub line: i32,
    pub content: char,
    pub display_offset: usize,
    pub fg: ansi::Color,
    pub bg: ansi::Color,
}
