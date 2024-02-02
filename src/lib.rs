mod backend;
pub mod bindings;
mod font;
mod term;
mod theme;
mod view;

pub use alacritty_terminal::term::TermMode;
pub use backend::settings::BackendSettings;
pub use font::FontSettings;
pub use term::{Command, Event, Term, TermSettings};
pub use theme::{ColorPalette, TermTheme};
pub use view::{term_view, TermView, TermViewState};
