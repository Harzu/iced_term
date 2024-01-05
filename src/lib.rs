mod term;
mod font;
mod backend;

pub use term::{Term, Event, data_received_subscription};
pub use backend::{Pty, Settings};
use std::io::Result;

pub fn init(id: u64, font_size: f32) -> Result<(Pty, term::Term)> {
    let pty = backend::Pty::new(id, Settings::default())?;

    Ok((
        pty,
        term::Term::new(id, font_size),
    ))
}


