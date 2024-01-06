mod term;
mod font;
mod backend;

pub use term::{Term, Event, Command, data_received_subscription};
pub use backend::{Pty, BackendSettings};
use std::io::Result;

pub fn init(id: u64, font_size: f32) -> Result<(Pty, Term)> {
    let pty = backend::Pty::new(id, BackendSettings::default())?;

    Ok((
        pty,
        Term::new(id, font_size),
    ))
}


