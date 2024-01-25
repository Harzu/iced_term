use alacritty_terminal::term::TermMode;
use iced_core::keyboard::{KeyCode, Modifiers};

#[derive(Clone, Hash, Debug, PartialEq, Eq)]
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
    action: BindingAction,
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
        let mut v = Vec::new();

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
                action: $action.into(),
            };

            v.push(binding);
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
    layout: Vec<Binding<InputKind>>,
}

impl BindingsLayout {
    pub fn new() -> Self {
        let mut layout = default_keyboard_bindings();
        layout.extend(platform_keyboard_bindings());
        Self { layout }
    }

    pub fn get_action(
        &self,
        input: InputKind,
        modifiers: Modifiers,
        terminal_mode: TermMode,
    ) -> BindingAction {
        for binding in &self.layout {
            let is_trigered = binding.target == input
                && binding.modifiers == modifiers
                && terminal_mode.contains(binding.terminal_mode_include)
                && !terminal_mode.intersects(binding.terminal_mode_exclude);

            if is_trigered {
                // println!("");
                // println!("input {} binding {:?} target {:?}", binding.target == input, binding.target, input);
                // println!("binding_modifiers {:?}", binding.modifiers);
                // println!("terminal_mode_include {:?}", binding.terminal_mode_include);
                // println!("terminal_mode_exclude {:?}", binding.terminal_mode_exclude);
                // println!("input modifiers {:?}", modifiers);
                // println!("terminal mode {:?}", terminal_mode);
                // println!("modifiers match {}", modifiers.contains(binding.modifiers));
                // println!("terminal_mode_include match {}", terminal_mode.contains(binding.terminal_mode_include));
                // println!("terminal_mode_exclude match {}", !terminal_mode.intersects(binding.terminal_mode_exclude));
                // println!("action {:?}", binding.action);
                // println!("{}", binding.terminal_mode_include.is_empty());
                // println!("diff {:?}", terminal_mode.difference(binding.terminal_mode_include));
                return binding.action.clone();
            };
        }

        BindingAction::Ignore
    }
}

pub fn default_keyboard_bindings() -> Vec<Binding<InputKind>> {
    generate_bindings!(
        KeyboardBinding;
        // ANY
        Enter;       BindingAction::Char('\x0d');
        NumpadEnter; BindingAction::Char('\x0d');
        Backspace;   BindingAction::Char('\x7f');
        Escape;      BindingAction::Char('\x1b');
        Tab;         BindingAction::Char('\x09');
        Insert;      BindingAction::ESC("\x1b[2~".into());
        Delete;      BindingAction::ESC("\x1b[3~".into());
        PageUp;      BindingAction::ESC("\x1b[5~".into());
        PageDown;    BindingAction::ESC("\x1b[6~".into());
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
        F13;         BindingAction::ESC("\x1b[25~".into());
        F14;         BindingAction::ESC("\x1b[26~".into());
        F15;         BindingAction::ESC("\x1b[28~".into());
        F16;         BindingAction::ESC("\x1b[29~".into());
        F17;         BindingAction::ESC("\x1b[31~".into());
        F18;         BindingAction::ESC("\x1b[32~".into());
        F19;         BindingAction::ESC("\x1b[33~".into());
        F20;         BindingAction::ESC("\x1b[34~".into());
        // APP_CURSOR Excluding
        End,   ~TermMode::APP_CURSOR; BindingAction::ESC("\x1b[F".into());
        Home,  ~TermMode::APP_CURSOR; BindingAction::ESC("\x1b[H".into());
        Up,    ~TermMode::APP_CURSOR; BindingAction::ESC("\x1b[A".into());
        Down,  ~TermMode::APP_CURSOR; BindingAction::ESC("\x1b[B".into());
        Left,  ~TermMode::APP_CURSOR; BindingAction::ESC("\x1b[D".into());
        Right, ~TermMode::APP_CURSOR; BindingAction::ESC("\x1b[C".into());
        // APP_CURSOR Including
        End,   +TermMode::APP_CURSOR; BindingAction::ESC("\x1BOF".into());
        Home,  +TermMode::APP_CURSOR; BindingAction::ESC("\x1BOH".into());
        Up,    +TermMode::APP_CURSOR; BindingAction::ESC("\x1bOA".into());
        Down,  +TermMode::APP_CURSOR; BindingAction::ESC("\x1bOB".into());
        Left,  +TermMode::APP_CURSOR; BindingAction::ESC("\x1bOD".into());
        Right, +TermMode::APP_CURSOR; BindingAction::ESC("\x1bOC".into());
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
        A,        Modifiers::CTRL; BindingAction::Char('\x01');
        B,        Modifiers::CTRL; BindingAction::Char('\x02');
        C,        Modifiers::CTRL; BindingAction::Char('\x03');
        D,        Modifiers::CTRL; BindingAction::Char('\x04');
        E,        Modifiers::CTRL; BindingAction::Char('\x05');
        F,        Modifiers::CTRL; BindingAction::Char('\x06');
        G,        Modifiers::CTRL; BindingAction::Char('\x07');
        H,        Modifiers::CTRL; BindingAction::Char('\x08');
        I,        Modifiers::CTRL; BindingAction::Char('\x09');
        J,        Modifiers::CTRL; BindingAction::Char('\x0a');
        K,        Modifiers::CTRL; BindingAction::Char('\x0b');
        L,        Modifiers::CTRL; BindingAction::Char('\x0c');
        M,        Modifiers::CTRL; BindingAction::Char('\x0d');
        N,        Modifiers::CTRL; BindingAction::Char('\x0e');
        O,        Modifiers::CTRL; BindingAction::Char('\x0f');
        P,        Modifiers::CTRL; BindingAction::Char('\x10');
        Q,        Modifiers::CTRL; BindingAction::Char('\x11');
        R,        Modifiers::CTRL; BindingAction::Char('\x12');
        S,        Modifiers::CTRL; BindingAction::Char('\x13');
        T,        Modifiers::CTRL; BindingAction::Char('\x14');
        U,        Modifiers::CTRL; BindingAction::Char('\x51');
        V,        Modifiers::CTRL; BindingAction::Char('\x16');
        W,        Modifiers::CTRL; BindingAction::Char('\x17');
        X,        Modifiers::CTRL; BindingAction::Char('\x18');
        Y,        Modifiers::CTRL; BindingAction::Char('\x19');
        Z,        Modifiers::CTRL; BindingAction::Char('\x1a');

        LBracket,  Modifiers::CTRL; BindingAction::Char('\x1b');
        RBracket,  Modifiers::CTRL; BindingAction::Char('\x1d');
        Backslash, Modifiers::CTRL; BindingAction::Char('\x1c');
        Slash,     Modifiers::CTRL; BindingAction::Char('\x1f');
        Key2,      Modifiers::CTRL; BindingAction::Char('\x00');
        // SHIFT
        Enter,       Modifiers::SHIFT; BindingAction::Char('\x0d');
        NumpadEnter, Modifiers::SHIFT; BindingAction::Char('\x0d');
        Backspace,   Modifiers::SHIFT; BindingAction::Char('\x7f');
        Tab,         Modifiers::SHIFT; BindingAction::ESC("\x1b[Z".into());
        End,         Modifiers::SHIFT, +TermMode::ALT_SCREEN; BindingAction::ESC("\x1b[1;2F".into());
        Home,        Modifiers::SHIFT, +TermMode::ALT_SCREEN; BindingAction::ESC("\x1b[1;2H".into());
        PageUp,      Modifiers::SHIFT, +TermMode::ALT_SCREEN; BindingAction::ESC("\x1b[5;2~".into());
        PageDown,    Modifiers::SHIFT, +TermMode::ALT_SCREEN; BindingAction::ESC("\x1b[6;2~".into());
        Up,          Modifiers::SHIFT; BindingAction::ESC("\x1b[1;2A".into());
        Down,        Modifiers::SHIFT; BindingAction::ESC("\x1b[1;2B".into());
        Left,        Modifiers::SHIFT; BindingAction::ESC("\x1b[1;2D".into());
        Right,       Modifiers::SHIFT; BindingAction::ESC("\x1b[1;2C".into());
        // ALT
        Backspace, Modifiers::ALT; BindingAction::ESC("\x1b\x7f".into());
        End,       Modifiers::ALT; BindingAction::ESC("\x1b[1;3F".into());
        Home,      Modifiers::ALT; BindingAction::ESC("\x1b[1;3H".into());
        Insert,    Modifiers::ALT; BindingAction::ESC("\x1b[3;2~".into());
        Delete,    Modifiers::ALT; BindingAction::ESC("\x1b[3;3~".into());
        PageUp,    Modifiers::ALT; BindingAction::ESC("\x1b[5;3~".into());
        PageDown,  Modifiers::ALT; BindingAction::ESC("\x1b[6;3~".into());
        Up,        Modifiers::ALT; BindingAction::ESC("\x1b[1;3A".into());
        Down,      Modifiers::ALT; BindingAction::ESC("\x1b[1;3B".into());
        Left,      Modifiers::ALT; BindingAction::ESC("\x1b[1;3D".into());
        Right,     Modifiers::ALT; BindingAction::ESC("\x1b[1;3C".into());
        // SHIFT + ALT
        End,   Modifiers::SHIFT | Modifiers::ALT; BindingAction::ESC("\x1b[1;4F".into());
        Home,  Modifiers::SHIFT | Modifiers::ALT; BindingAction::ESC("\x1b[1;4H".into());
        Up,    Modifiers::SHIFT | Modifiers::ALT; BindingAction::ESC("\x1b[1;4A".into());
        Down,  Modifiers::SHIFT | Modifiers::ALT; BindingAction::ESC("\x1b[1;4B".into());
        Left,  Modifiers::SHIFT | Modifiers::ALT; BindingAction::ESC("\x1b[1;4D".into());
        Right, Modifiers::SHIFT | Modifiers::ALT; BindingAction::ESC("\x1b[1;4C".into());
        // SHIFT + CTRL
        End,   Modifiers::SHIFT | Modifiers::CTRL; BindingAction::ESC("\x1b[1;6F".into());
        Home,  Modifiers::SHIFT | Modifiers::CTRL; BindingAction::ESC("\x1b[1;6H".into());
        Up,    Modifiers::SHIFT | Modifiers::CTRL; BindingAction::ESC("\x1b[1;6A".into());
        Down,  Modifiers::SHIFT | Modifiers::CTRL; BindingAction::ESC("\x1b[1;6B".into());
        Left,  Modifiers::SHIFT | Modifiers::CTRL; BindingAction::ESC("\x1b[1;6D".into());
        Right, Modifiers::SHIFT | Modifiers::CTRL; BindingAction::ESC("\x1b[1;6C".into());
        A,     Modifiers::SHIFT | Modifiers::CTRL; BindingAction::Char('\x01');
        B,     Modifiers::SHIFT | Modifiers::CTRL; BindingAction::Char('\x02');
        C,     Modifiers::SHIFT | Modifiers::CTRL; BindingAction::Char('\x03');
        D,     Modifiers::SHIFT | Modifiers::CTRL; BindingAction::Char('\x04');
        E,     Modifiers::SHIFT | Modifiers::CTRL; BindingAction::Char('\x05');
        F,     Modifiers::SHIFT | Modifiers::CTRL; BindingAction::Char('\x06');
        G,     Modifiers::SHIFT | Modifiers::CTRL; BindingAction::Char('\x07');
        H,     Modifiers::SHIFT | Modifiers::CTRL; BindingAction::Char('\x08');
        I,     Modifiers::SHIFT | Modifiers::CTRL; BindingAction::Char('\x09');
        J,     Modifiers::SHIFT | Modifiers::CTRL; BindingAction::Char('\x0a');
        K,     Modifiers::SHIFT | Modifiers::CTRL; BindingAction::Char('\x0b');
        L,     Modifiers::SHIFT | Modifiers::CTRL; BindingAction::Char('\x0c');
        M,     Modifiers::SHIFT | Modifiers::CTRL; BindingAction::Char('\x0d');
        N,     Modifiers::SHIFT | Modifiers::CTRL; BindingAction::Char('\x0e');
        O,     Modifiers::SHIFT | Modifiers::CTRL; BindingAction::Char('\x0f');
        P,     Modifiers::SHIFT | Modifiers::CTRL; BindingAction::Char('\x10');
        Q,     Modifiers::SHIFT | Modifiers::CTRL; BindingAction::Char('\x11');
        R,     Modifiers::SHIFT | Modifiers::CTRL; BindingAction::Char('\x12');
        S,     Modifiers::SHIFT | Modifiers::CTRL; BindingAction::Char('\x13');
        T,     Modifiers::SHIFT | Modifiers::CTRL; BindingAction::Char('\x14');
        U,     Modifiers::SHIFT | Modifiers::CTRL; BindingAction::Char('\x51');
        V,     Modifiers::SHIFT | Modifiers::CTRL; BindingAction::Char('\x16');
        W,     Modifiers::SHIFT | Modifiers::CTRL; BindingAction::Char('\x17');
        X,     Modifiers::SHIFT | Modifiers::CTRL; BindingAction::Char('\x18');
        Y,     Modifiers::SHIFT | Modifiers::CTRL; BindingAction::Char('\x19');
        Z,     Modifiers::SHIFT | Modifiers::CTRL; BindingAction::Char('\x1a');
        // CTRL + ALT
        End,      Modifiers::CTRL | Modifiers::ALT; BindingAction::ESC("\x1b[1;7F".into());
        Home,     Modifiers::CTRL | Modifiers::ALT; BindingAction::ESC("\x1b[1;7H".into());
        PageUp,   Modifiers::CTRL | Modifiers::ALT; BindingAction::ESC("\x1b[5;7~".into());
        PageDown, Modifiers::CTRL | Modifiers::ALT; BindingAction::ESC("\x1b[6;7~".into());
        Up,       Modifiers::CTRL | Modifiers::ALT; BindingAction::ESC("\x1b[1;7A".into());
        Down,     Modifiers::CTRL | Modifiers::ALT; BindingAction::ESC("\x1b[1;7B".into());
        Left,     Modifiers::CTRL | Modifiers::ALT; BindingAction::ESC("\x1b[1;7D".into());
        Right,    Modifiers::CTRL | Modifiers::ALT; BindingAction::ESC("\x1b[1;7C".into());
        // SHIFT + CTRL + ALT
        End,     Modifiers::SHIFT | Modifiers::ALT; BindingAction::ESC("\x1b[1;8F".into());
        Home,    Modifiers::SHIFT | Modifiers::ALT; BindingAction::ESC("\x1b[1;8H".into());
        Up,      Modifiers::SHIFT | Modifiers::ALT; BindingAction::ESC("\x1b[1;8A".into());
        Down,    Modifiers::SHIFT | Modifiers::ALT; BindingAction::ESC("\x1b[1;8B".into());
        Left,    Modifiers::SHIFT | Modifiers::ALT; BindingAction::ESC("\x1b[1;8D".into());
        Right,   Modifiers::SHIFT | Modifiers::ALT; BindingAction::ESC("\x1b[1;8C".into());
    )
}

#[cfg(target_os = "macos")]
fn platform_keyboard_bindings() -> Vec<Binding<InputKind>> {
    generate_bindings!(
        KeyboardBinding;
        C, Modifiers::LOGO; BindingAction::Copy;
        V, Modifiers::LOGO; BindingAction::Paste;
    )
}

#[cfg(target_os = "linux")]
fn platform_keyboard_bindings() -> Vec<Binding<InputKind>> {
    generate_bindings!(
        KeyboardBinding;
        C, Modifiers::SHIFT | Modifiers::CTRL; BindingAction::Copy;
        V, Modifiers::SHIFT | Modifiers::CTRL; BindingAction::Paste;
    )
}
