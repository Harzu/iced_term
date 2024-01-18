use iced_core::keyboard::{KeyCode, Modifiers};
use std::collections::HashMap;

// RENAME TO SHORTCATS
#[derive(Clone)]
pub enum KeyboardShortcatsAction {
    Copy,
    Paste,
    Char(char),
    Ignore,
}

#[derive(Clone)]
pub struct KeyboardShortcatsLayout {
    layout_map: HashMap<(KeyCode, Modifiers), KeyboardShortcatsAction>,
}

impl KeyboardShortcatsLayout {
    pub fn new() -> Self {
        let layout_map = HashMap::from([
            // CTRL shortcats
            ((KeyCode::A, Modifiers::CTRL), KeyboardShortcatsAction::Char('\x01')),
            ((KeyCode::B, Modifiers::CTRL), KeyboardShortcatsAction::Char('\x02')),
            ((KeyCode::C, Modifiers::CTRL), KeyboardShortcatsAction::Char('\x03')),
            ((KeyCode::D, Modifiers::CTRL), KeyboardShortcatsAction::Char('\x04')),
            ((KeyCode::E, Modifiers::CTRL), KeyboardShortcatsAction::Char('\x05')),
            ((KeyCode::F, Modifiers::CTRL), KeyboardShortcatsAction::Char('\x06')),
            ((KeyCode::G, Modifiers::CTRL), KeyboardShortcatsAction::Char('\x07')),
            ((KeyCode::H, Modifiers::CTRL), KeyboardShortcatsAction::Char('\x08')),
            ((KeyCode::I, Modifiers::CTRL), KeyboardShortcatsAction::Char('\t')),
            ((KeyCode::J, Modifiers::CTRL), KeyboardShortcatsAction::Char('\n')),
            ((KeyCode::K, Modifiers::CTRL), KeyboardShortcatsAction::Char('\x0B')),
            ((KeyCode::L, Modifiers::CTRL), KeyboardShortcatsAction::Char('\x0C')),
            ((KeyCode::M, Modifiers::CTRL), KeyboardShortcatsAction::Char('\r')),
            ((KeyCode::N, Modifiers::CTRL), KeyboardShortcatsAction::Char('\x0E')),
            ((KeyCode::O, Modifiers::CTRL), KeyboardShortcatsAction::Char('\x15')),
            ((KeyCode::P, Modifiers::CTRL), KeyboardShortcatsAction::Char('\x10')),
            ((KeyCode::Q, Modifiers::CTRL), KeyboardShortcatsAction::Char('\x11')),
            ((KeyCode::R, Modifiers::CTRL), KeyboardShortcatsAction::Char('\x12')),
            ((KeyCode::S, Modifiers::CTRL), KeyboardShortcatsAction::Char('\x13')),
            ((KeyCode::T, Modifiers::CTRL), KeyboardShortcatsAction::Char('\x14')),
            ((KeyCode::U, Modifiers::CTRL), KeyboardShortcatsAction::Char('\x15')),
            ((KeyCode::V, Modifiers::CTRL), KeyboardShortcatsAction::Char('\x16')),
            ((KeyCode::W, Modifiers::CTRL), KeyboardShortcatsAction::Char('\x17')),
            ((KeyCode::X, Modifiers::CTRL), KeyboardShortcatsAction::Char('\x18')),
            ((KeyCode::Y, Modifiers::CTRL), KeyboardShortcatsAction::Char('\x19')),
            ((KeyCode::Z, Modifiers::CTRL), KeyboardShortcatsAction::Char('\x1A')),

            // COPY/PASTE
            ((KeyCode::C, Modifiers::CTRL & Modifiers::SHIFT), KeyboardShortcatsAction::Paste)
        ]);

        Self { layout_map }
    }

    pub fn get_action(
        &self,
        key_code: KeyCode,
        modifiers: Modifiers,
    ) -> KeyboardShortcatsAction {
        println!("{:?}", modifiers);

        match self.layout_map.get(&(key_code, modifiers)) {
            Some(action) => action.clone(),
            None => KeyboardShortcatsAction::Ignore,
        }
    }
}
