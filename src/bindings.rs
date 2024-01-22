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
            Home;        BindingAction::ESC("\x1b[1~".into());
            Insert;      BindingAction::ESC("\x1b[2~".into());
            Delete;      BindingAction::ESC("\x1b[3~".into());
            End;         BindingAction::ESC("\x1b[4~".into());
            PageUp;      BindingAction::ESC("\x1b[5~".into());
            PageDown;    BindingAction::ESC("\x1b[6~".into());
            End;         BindingAction::ESC("\x1b[F".into());
            Home;        BindingAction::ESC("\x1b[H".into());
            Up;          BindingAction::ESC("\x1b[A".into());
            Down;        BindingAction::ESC("\x1b[B".into());
            Left;        BindingAction::ESC("\x1b[D".into());
            Right;       BindingAction::ESC("\x1b[C".into());
            F1;          BindingAction::ESC("\x1bOP".into());
            F2;          BindingAction::ESC("\x1bOQ".into());
            F3;          BindingAction::ESC("\x1bOR".into());
            F4;          BindingAction::ESC("\x1bOS".into());
            F5;          BindingAction::ESC("\x1b[15~".into());
            F6;          BindingAction::ESC("\x1b[17~".into());
            F7;          BindingAction::ESC("\x1b[18~".into());
            F8;          BindingAction::ESC("\x1b[19~".into());
            F9;          BindingAction::ESC("\x1b[20~".into());
            F10;         BindingAction::ESC("\x1b[21~".into());
            F11;         BindingAction::ESC("\x1b[23~".into());
            F12;         BindingAction::ESC("\x1b[24~".into());
            // APP_CURSOR Terminal mode
            End,   +TermMode::APP_CURSOR; BindingAction::ESC("\x1BOF".into());
            Home,  +TermMode::APP_CURSOR; BindingAction::ESC("\x1BOH".into());
            Up,    +TermMode::APP_CURSOR; BindingAction::ESC("\x1b[OA".into());
            Down,  +TermMode::APP_CURSOR; BindingAction::ESC("\x1b[OB".into());
            Left,  +TermMode::APP_CURSOR; BindingAction::ESC("\x1b[OD".into());
            Right, +TermMode::APP_CURSOR; BindingAction::ESC("\x1b[OC".into());
            // CTRL
            Up,       Modifiers::CTRL; BindingAction::ESC("\x1b[1;5A".into());
            Down,     Modifiers::CTRL; BindingAction::ESC("\x1b[1;5B".into());
            Left,     Modifiers::CTRL; BindingAction::ESC("\x1b[1;5D".into());
            Right,    Modifiers::CTRL; BindingAction::ESC("\x1b[1;5C".into());
            End,      Modifiers::CTRL; BindingAction::ESC("\x1b[1;5F".into());
            Home,     Modifiers::CTRL; BindingAction::ESC("\x1b[1;5H".into());
            Delete,   Modifiers::CTRL; BindingAction::ESC("\x1b[3;5~".into());
            PageUp,   Modifiers::CTRL; BindingAction::ESC("\x1b[5;5~".into());
            PageDown, Modifiers::CTRL; BindingAction::ESC("\x1b[6;5~".into());
            F1,       Modifiers::CTRL; BindingAction::ESC("\x1bO;5P".into());
            F2,       Modifiers::CTRL; BindingAction::ESC("\x1bO;5Q".into());
            F3,       Modifiers::CTRL; BindingAction::ESC("\x1bO;5R".into());
            F4,       Modifiers::CTRL; BindingAction::ESC("\x1bO;5S".into());
            F5,       Modifiers::CTRL; BindingAction::ESC("\x1b[15;5~".into());
            F6,       Modifiers::CTRL; BindingAction::ESC("\x1b[17;5~".into());
            F7,       Modifiers::CTRL; BindingAction::ESC("\x1b[18;5~".into());
            F8,       Modifiers::CTRL; BindingAction::ESC("\x1b[19;5~".into());
            F9,       Modifiers::CTRL; BindingAction::ESC("\x1b[20;5~".into());
            F10,      Modifiers::CTRL; BindingAction::ESC("\x1b[21;5~".into());
            F11,      Modifiers::CTRL; BindingAction::ESC("\x1b[23;5~".into());
            F12,      Modifiers::CTRL; BindingAction::ESC("\x1b[24;5~".into());
            // SHIFT
            Tab,     Modifiers::SHIFT; BindingAction::ESC("\x1b[Z".into());
            Home,    Modifiers::SHIFT; BindingAction::ESC("\x1b[1;2H".into());
            Up,      Modifiers::SHIFT; BindingAction::ESC("\x1b[1;2A".into());
            Down,    Modifiers::SHIFT; BindingAction::ESC("\x1b[1;2B".into());
            Left,    Modifiers::SHIFT; BindingAction::ESC("\x1b[1;2D".into());
            Right,   Modifiers::SHIFT; BindingAction::ESC("\x1b[1;2C".into());
            // ALT
            Home,     Modifiers::ALT; BindingAction::ESC("\x1b[1;3H".into());
            Insert,   Modifiers::ALT; BindingAction::ESC("\x1b[3;2~".into());
            Delete,   Modifiers::ALT; BindingAction::ESC("\x1b[3;3~".into());
            PageUp,   Modifiers::ALT; BindingAction::ESC("\x1b[5;3~".into());
            PageDown, Modifiers::ALT; BindingAction::ESC("\x1b[6;3~".into());
            Up,       Modifiers::ALT; BindingAction::ESC("\x1b[1;3A".into());
            Down,     Modifiers::ALT; BindingAction::ESC("\x1b[1;3B".into());
            Left,     Modifiers::ALT; BindingAction::ESC("\x1b[1;3D".into());
            Right,    Modifiers::ALT; BindingAction::ESC("\x1b[1;3C".into());
            // SHIFT + ALT
            Home,    Modifiers::SHIFT | Modifiers::ALT; BindingAction::ESC("\x1b[1;4H".into());
            Up,      Modifiers::SHIFT | Modifiers::ALT; BindingAction::ESC("\x1b[1;4A".into());
            Down,    Modifiers::SHIFT | Modifiers::ALT; BindingAction::ESC("\x1b[1;4B".into());
            Left,    Modifiers::SHIFT | Modifiers::ALT; BindingAction::ESC("\x1b[1;4D".into());
            Right,   Modifiers::SHIFT | Modifiers::ALT; BindingAction::ESC("\x1b[1;4C".into());
            // SHIFT + CTRL
            Home,    Modifiers::SHIFT | Modifiers::CTRL; BindingAction::ESC("\x1b[1;6H".into());
            Up,      Modifiers::SHIFT | Modifiers::CTRL; BindingAction::ESC("\x1b[1;6A".into());
            Down,    Modifiers::SHIFT | Modifiers::CTRL; BindingAction::ESC("\x1b[1;6B".into());
            Left,    Modifiers::SHIFT | Modifiers::CTRL; BindingAction::ESC("\x1b[1;6D".into());
            Right,   Modifiers::SHIFT | Modifiers::CTRL; BindingAction::ESC("\x1b[1;6C".into());
            // CTRL + ALT
            Home,     Modifiers::CTRL | Modifiers::ALT; BindingAction::ESC("\x1b[1;7H".into());
            PageUp,   Modifiers::CTRL | Modifiers::ALT; BindingAction::ESC("\x1b[5;7~".into());
            PageDown, Modifiers::CTRL | Modifiers::ALT; BindingAction::ESC("\x1b[6;7~".into());
            Up,       Modifiers::CTRL | Modifiers::ALT; BindingAction::ESC("\x1b[1;7A".into());
            Down,     Modifiers::CTRL | Modifiers::ALT; BindingAction::ESC("\x1b[1;7B".into());
            Left,     Modifiers::CTRL | Modifiers::ALT; BindingAction::ESC("\x1b[1;7D".into());
            Right,    Modifiers::CTRL | Modifiers::ALT; BindingAction::ESC("\x1b[1;7C".into());
            // SHIFT + CTRL + ALT
            Home,    Modifiers::SHIFT | Modifiers::ALT; BindingAction::ESC("\x1b[1;8H".into());
            Up,      Modifiers::SHIFT | Modifiers::ALT; BindingAction::ESC("\x1b[1;8A".into());
            Down,    Modifiers::SHIFT | Modifiers::ALT; BindingAction::ESC("\x1b[1;8B".into());
            Left,    Modifiers::SHIFT | Modifiers::ALT; BindingAction::ESC("\x1b[1;8D".into());
            Right,   Modifiers::SHIFT | Modifiers::ALT; BindingAction::ESC("\x1b[1;8C".into());
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
