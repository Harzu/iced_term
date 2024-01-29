mod backend;
pub mod bindings;
mod font;
mod term;
mod theme;

pub use alacritty_terminal::term::TermMode;
pub use backend::BackendSettings;
pub use font::FontSettings;
pub use term::{
    term_view, Command, Event, Term, TermSettings, TermView, TermViewState,
};
pub use theme::{ColorPalette, TermTheme};
