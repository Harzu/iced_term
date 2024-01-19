use alacritty_terminal::term::TermMode;
use iced_core::keyboard::{KeyCode, Modifiers};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BindingAction {
    Copy,
    Paste,
    Char(char),
    ESC(String),
    Ignore,
}

#[derive(Clone, Hash, Debug, PartialEq, Eq)]
pub struct Binding<T> {
    target: T,
    modifiers: Modifiers,
    terminal_mode_include: TermMode,
    terminal_mode_exclude: TermMode,
}

type KeyboardBinding = Binding<KeyCode>;

macro_rules! generate_bindings {
    (
        $tp:ident;
        $(
            $key:tt$(::$button:ident)?
            $(,$key_modifiers:expr)*
            $(,+$terminal_mode_include:expr)*
            $(,~$terminal_mode_exclude:expr)*
            ;$action:expr
        );*
        $(;)*
    ) => {{
        let mut v = HashMap::new();

        $(
            let mut _key_modifiers = Modifiers::empty();
            $(_key_modifiers = $key_modifiers;)*
            let mut _terminal_mode_include = TermMode::empty();
            $(_terminal_mode_include.insert($terminal_mode_include);)*
            let mut _terminal_mode_exclude = TermMode::empty();
            $(_terminal_mode_exclude.insert($terminal_mode_exclude);)*

            let binding = $tp {
                target: $key$(::$button)?,
                modifiers: _key_modifiers,
                terminal_mode_include: _terminal_mode_include,
                terminal_mode_exclude: _terminal_mode_exclude,
            };

            v.insert(binding, $action.into());
        )*

        v
    }};
}

#[derive(Clone)]
pub struct KeyboardShortcatsLayout {
    bindings: HashMap<Binding<KeyCode>, BindingAction>,
}

impl KeyboardShortcatsLayout {
    pub fn new() -> Self {
        let bindings = generate_bindings!(
            KeyboardBinding;
            KeyCode::Tab;         BindingAction::Char('\t');
            KeyCode::Enter;       BindingAction::Char('\r');
            KeyCode::NumpadEnter; BindingAction::Char('\r');
            KeyCode::Backspace;   BindingAction::Char('\x7F');
            KeyCode::Escape;      BindingAction::Char('\x1B');
            KeyCode::Insert;      BindingAction::ESC("\x1B[2~".into());
            KeyCode::Delete;      BindingAction::ESC("\x1B[3~".into());
            KeyCode::PageUp;      BindingAction::ESC("\x1B[5~".into());
            KeyCode::PageDown;    BindingAction::ESC("\x1B[6~".into());
            KeyCode::F1;          BindingAction::ESC("\x1BOP".into());
            KeyCode::F2;          BindingAction::ESC("\x1BOQ".into());
            KeyCode::F3;          BindingAction::ESC("\x1BOR".into());
            KeyCode::F4;          BindingAction::ESC("\x1BOS".into());
            KeyCode::F5;          BindingAction::ESC("\x1B[15~".into());
            KeyCode::F6;          BindingAction::ESC("\x1B[17~".into());
            KeyCode::F7;          BindingAction::ESC("\x1B[18~".into());
            KeyCode::F8;          BindingAction::ESC("\x1B[19~".into());
            KeyCode::F9;          BindingAction::ESC("\x1B[20~".into());
            KeyCode::F10;         BindingAction::ESC("\x1B[21~".into());
            KeyCode::F11;         BindingAction::ESC("\x1B[23~".into());
            KeyCode::F12;         BindingAction::ESC("\x1B[24~".into());
            KeyCode::End;         BindingAction::ESC("\x1B[F".into());
            KeyCode::Home;        BindingAction::ESC("\x1B[H".into());
            KeyCode::Up;          BindingAction::ESC("\x1B[A".into());
            KeyCode::Down;        BindingAction::ESC("\x1B[B".into());
            KeyCode::Left;        BindingAction::ESC("\x1B[D".into());
            KeyCode::Right;       BindingAction::ESC("\x1B[C".into());

            // APP_CURSOR Terminal mode
            KeyCode::End,   +TermMode::APP_CURSOR; BindingAction::ESC("\x1BOF".into());
            KeyCode::Home,  +TermMode::APP_CURSOR; BindingAction::ESC("\x1BOH".into());
            KeyCode::Up,    +TermMode::APP_CURSOR; BindingAction::ESC("\x1BOA".into());
            KeyCode::Down,  +TermMode::APP_CURSOR; BindingAction::ESC("\x1BOB".into());
            KeyCode::Left,  +TermMode::APP_CURSOR; BindingAction::ESC("\x1BOD".into());
            KeyCode::Right, +TermMode::APP_CURSOR; BindingAction::ESC("\x1BOC".into());

            // Control
            KeyCode::Up,       Modifiers::CTRL; BindingAction::ESC("\x1B[1;5A".into());
            KeyCode::Down,     Modifiers::CTRL; BindingAction::ESC("\x1B[1;5B".into());
            KeyCode::Left,     Modifiers::CTRL; BindingAction::ESC("\x1B[1;5D".into());
            KeyCode::Right,    Modifiers::CTRL; BindingAction::ESC("\x1B[1;5C".into());
            KeyCode::End,      Modifiers::CTRL; BindingAction::ESC("\x1B[1;5F".into());
            KeyCode::Home,     Modifiers::CTRL; BindingAction::ESC("\x1B[1;5H".into());
            KeyCode::Delete,   Modifiers::CTRL; BindingAction::ESC("\x1B[3;5~".into());
            KeyCode::PageUp,   Modifiers::CTRL; BindingAction::ESC("\x1B[5;5~".into());
            KeyCode::PageDown, Modifiers::CTRL; BindingAction::ESC("\x1B[6;5~".into());
            KeyCode::F1,       Modifiers::CTRL; BindingAction::ESC("\x1BO;5P".into());
            KeyCode::F2,       Modifiers::CTRL; BindingAction::ESC("\x1BO;5Q".into());
            KeyCode::F3,       Modifiers::CTRL; BindingAction::ESC("\x1BO;5R".into());
            KeyCode::F4,       Modifiers::CTRL; BindingAction::ESC("\x1BO;5S".into());
            KeyCode::F5,       Modifiers::CTRL; BindingAction::ESC("\x1B[15;5~".into());
            KeyCode::F6,       Modifiers::CTRL; BindingAction::ESC("\x1B[17;5~".into());
            KeyCode::F7,       Modifiers::CTRL; BindingAction::ESC("\x1B[18;5~".into());
            KeyCode::F8,       Modifiers::CTRL; BindingAction::ESC("\x1B[19;5~".into());
            KeyCode::F9,       Modifiers::CTRL; BindingAction::ESC("\x1B[20;5~".into());
            KeyCode::F10,      Modifiers::CTRL; BindingAction::ESC("\x1B[21;5~".into());
            KeyCode::F11,      Modifiers::CTRL; BindingAction::ESC("\x1B[23;5~".into());
            KeyCode::F12,      Modifiers::CTRL; BindingAction::ESC("\x1B[24;5~".into());
        );

        Self { bindings }
    }

    pub fn get_action(
        &self,
        key_code: KeyCode,
        key_modifiers: Modifiers,
        terminal_mode: TermMode,
    ) -> BindingAction {
        for (binding, action) in &self.bindings {
            let is_triggered = binding.target == key_code
                && binding.modifiers == key_modifiers
                && terminal_mode.contains(binding.terminal_mode_include)
                && !terminal_mode.intersects(binding.terminal_mode_exclude);

            if is_triggered {
                return action.clone();
            };
        }

        BindingAction::Ignore
    }
}
