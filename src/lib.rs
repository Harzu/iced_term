mod backend;
mod font;
mod term;
mod theme;

pub use backend::BackendSettings;
pub use font::FontSettings;
pub use term::{
    term_view, Command, Event, Term, TermSettings, TermView, TermViewState,
};
