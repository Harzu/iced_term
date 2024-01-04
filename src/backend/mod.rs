mod renderable_cell;
mod settings;
mod pty;

use std::collections::HashMap;
use std::io::Result;

pub use pty::Pty;
pub use settings::Settings;
pub use renderable_cell::RenderableCell;

#[derive(Default)]
pub struct TerminalController {
    last_tab: u64,
    tabs: HashMap<u64, Pty>
}

impl TerminalController {
    pub fn new() -> Self {
        TerminalController::default()
    }

    pub fn create_tab(&mut self) -> Result<u64> {
        let pty_settings = Settings::default();
        let new_tab_id = self.last_tab + 1;
        let pty = Pty::new(new_tab_id, pty_settings)?;
        self.tabs.insert(new_tab_id, pty);
        Ok(self.last_tab)
    }

    pub fn get_tab(&self, id: u64) -> Option<&Pty> {
        self.tabs.get(&id)
    }

    pub fn get_mut_tab(&mut self, id: u64) -> Option<&mut Pty> {
        self.tabs.get_mut(&id)
    }
}