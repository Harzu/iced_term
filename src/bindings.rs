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
pub enum InputKind {
    Char(char),
    KeyCode(KeyCode),
}

#[derive(Clone, Hash, Debug, PartialEq, Eq)]
pub struct Binding<T> {
    target: T,
    modifiers: Modifiers,
    terminal_mode_include: TermMode,
    terminal_mode_exclude: TermMode,
}

type KeyboardBinding = Binding<InputKind>;

macro_rules! generate_bindings {
    (
        $binding_type:ident;
        $(
            $input_kind:tt$(::$button:ident)?
            $(,$input_modifiers:expr)*
            $(,+$terminal_mode_include:expr)*
            $(,~$terminal_mode_exclude:expr)*
            ;$action:expr
        );*
        $(;)*
    ) => {{
        let mut v = HashMap::new();

        $(
            let mut _input_modifiers = Modifiers::empty();
            $(_input_modifiers = $input_modifiers;)*
            let mut _terminal_mode_include = TermMode::empty();
            $(_terminal_mode_include.insert($terminal_mode_include);)*
            let mut _terminal_mode_exclude = TermMode::empty();
            $(_terminal_mode_exclude.insert($terminal_mode_exclude);)*

            let binding = $binding_type {
                target: input!($binding_type, $input_kind),
                modifiers: _input_modifiers,
                terminal_mode_include: _terminal_mode_include,
                terminal_mode_exclude: _terminal_mode_exclude,
            };

            v.insert(binding, $action.into());
        )*

        v
    }};
}

macro_rules! input {
    (KeyboardBinding, $char:literal) => {{
        InputKind::Char($char)
    }};
    (KeyboardBinding, $key:ident) => {{
        InputKind::KeyCode(KeyCode::$key)
    }};
}

#[derive(Clone)]
pub struct BindingsLayout {
    layout: HashMap<Binding<InputKind>, BindingAction>,
}

impl BindingsLayout {
    pub fn new() -> Self {
        let layout = generate_bindings!(
            KeyboardBinding;
            Tab;         BindingAction::Char('\t');
            Enter;       BindingAction::Char('\r');
            NumpadEnter; BindingAction::Char('\r');
            Backspace;   BindingAction::Char('\x7F');
            Escape;      BindingAction::Char('\x1B');
            Insert;      BindingAction::ESC("\x1B[2~".into());
            Delete;      BindingAction::ESC("\x1B[3~".into());
            PageUp;      BindingAction::ESC("\x1B[5~".into());
            PageDown;    BindingAction::ESC("\x1B[6~".into());
            F1;          BindingAction::ESC("\x1BOP".into());
            F2;          BindingAction::ESC("\x1BOQ".into());
            F3;          BindingAction::ESC("\x1BOR".into());
            F4;          BindingAction::ESC("\x1BOS".into());
            F5;          BindingAction::ESC("\x1B[15~".into());
            F6;          BindingAction::ESC("\x1B[17~".into());
            F7;          BindingAction::ESC("\x1B[18~".into());
            F8;          BindingAction::ESC("\x1B[19~".into());
            F9;          BindingAction::ESC("\x1B[20~".into());
            F10;         BindingAction::ESC("\x1B[21~".into());
            F11;         BindingAction::ESC("\x1B[23~".into());
            F12;         BindingAction::ESC("\x1B[24~".into());
            End;         BindingAction::ESC("\x1B[F".into());
            Home;        BindingAction::ESC("\x1B[H".into());
            Up;          BindingAction::ESC("\x1B[A".into());
            Down;        BindingAction::ESC("\x1B[B".into());
            Left;        BindingAction::ESC("\x1B[D".into());
            Right;       BindingAction::ESC("\x1B[C".into());

            // APP_CURSOR Terminal mode
            End,   +TermMode::APP_CURSOR; BindingAction::ESC("\x1BOF".into());
            Home,  +TermMode::APP_CURSOR; BindingAction::ESC("\x1BOH".into());
            Up,    +TermMode::APP_CURSOR; BindingAction::ESC("\x1BOA".into());
            Down,  +TermMode::APP_CURSOR; BindingAction::ESC("\x1BOB".into());
            Left,  +TermMode::APP_CURSOR; BindingAction::ESC("\x1BOD".into());
            Right, +TermMode::APP_CURSOR; BindingAction::ESC("\x1BOC".into());

            // Control
            Up,       Modifiers::CTRL; BindingAction::ESC("\x1B[1;5A".into());
            Down,     Modifiers::CTRL; BindingAction::ESC("\x1B[1;5B".into());
            Left,     Modifiers::CTRL; BindingAction::ESC("\x1B[1;5D".into());
            Right,    Modifiers::CTRL; BindingAction::ESC("\x1B[1;5C".into());
            End,      Modifiers::CTRL; BindingAction::ESC("\x1B[1;5F".into());
            Home,     Modifiers::CTRL; BindingAction::ESC("\x1B[1;5H".into());
            Delete,   Modifiers::CTRL; BindingAction::ESC("\x1B[3;5~".into());
            PageUp,   Modifiers::CTRL; BindingAction::ESC("\x1B[5;5~".into());
            PageDown, Modifiers::CTRL; BindingAction::ESC("\x1B[6;5~".into());
            F1,       Modifiers::CTRL; BindingAction::ESC("\x1BO;5P".into());
            F2,       Modifiers::CTRL; BindingAction::ESC("\x1BO;5Q".into());
            F3,       Modifiers::CTRL; BindingAction::ESC("\x1BO;5R".into());
            F4,       Modifiers::CTRL; BindingAction::ESC("\x1BO;5S".into());
            F5,       Modifiers::CTRL; BindingAction::ESC("\x1B[15;5~".into());
            F6,       Modifiers::CTRL; BindingAction::ESC("\x1B[17;5~".into());
            F7,       Modifiers::CTRL; BindingAction::ESC("\x1B[18;5~".into());
            F8,       Modifiers::CTRL; BindingAction::ESC("\x1B[19;5~".into());
            F9,       Modifiers::CTRL; BindingAction::ESC("\x1B[20;5~".into());
            F10,      Modifiers::CTRL; BindingAction::ESC("\x1B[21;5~".into());
            F11,      Modifiers::CTRL; BindingAction::ESC("\x1B[23;5~".into());
            F12,      Modifiers::CTRL; BindingAction::ESC("\x1B[24;5~".into());
        );

        Self { layout }
    }

    pub fn get_action(
        &self,
        input: InputKind,
        key_modifiers: Modifiers,
        terminal_mode: TermMode,
    ) -> BindingAction {
        for (binding, action) in &self.layout {
            let is_triggered = binding.target == input
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
