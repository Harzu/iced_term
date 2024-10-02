pub mod actions;
pub mod bindings;
pub mod settings;
mod backend;
mod font;
mod terminal;
mod theme;
mod view;
mod subscription;

pub use alacritty_terminal::term::TermMode;
pub use subscription::TerminalSubscription;
pub use terminal::{Command, Event, Terminal};
pub use theme::{ColorPalette, Theme};
pub use view::{term_view, TermView, TermViewState};
