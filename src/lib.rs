pub mod actions;
mod backend;
pub mod bindings;
mod font;
pub mod settings;
mod subscription;
mod terminal;
mod theme;
mod view;

pub use alacritty_terminal::term::TermMode;
pub use subscription::Subscription;
pub use terminal::{Command, Event, Terminal};
pub use theme::{ColorPalette, Theme};
pub use view::{term_view, TermView, TermViewState};
