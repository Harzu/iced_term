use alacritty_terminal::term::TermMode;
use iced_core::keyboard::{KeyCode, Modifiers};

#[derive(Clone, Hash, Debug, PartialEq, Eq)]
pub enum BindingAction {
    Copy,
    Paste,
    Char(char),
    Esc(String),
    Ignore,
}

#[derive(Clone, Hash, Debug, PartialEq, Eq)]
pub enum InputKind {
    Char(char),
    KeyCode(KeyCode),
}

#[derive(Clone, Hash, Debug, PartialEq, Eq)]
pub struct Binding<T> {
    pub target: T,
    pub modifiers: Modifiers,
    pub terminal_mode_include: TermMode,
    pub terminal_mode_exclude: TermMode,
}

pub type KeyboardBinding = Binding<InputKind>;

#[macro_export]
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
        macro_rules! input_kind_match {
            (KeyboardBinding, $char:literal) => {{
                InputKind::Char($char)
            }};
            (KeyboardBinding, $key:ident) => {{
                InputKind::KeyCode(KeyCode::$key)
            }};
        }

        let mut v = Vec::new();

        $(
            let mut _input_modifiers = Modifiers::empty();
            $(_input_modifiers = $input_modifiers;)*
            let mut _terminal_mode_include = TermMode::empty();
            $(_terminal_mode_include.insert($terminal_mode_include);)*
            let mut _terminal_mode_exclude = TermMode::empty();
            $(_terminal_mode_exclude.insert($terminal_mode_exclude);)*

            let binding = $binding_type {
                target: input_kind_match!($binding_type, $input_kind),
                modifiers: _input_modifiers,
                terminal_mode_include: _terminal_mode_include,
                terminal_mode_exclude: _terminal_mode_exclude,
            };

            v.push((binding, $action.into()));
        )*

        v
    }};
}

#[derive(Clone, Debug)]
pub struct BindingsLayout {
    layout: Vec<(Binding<InputKind>, BindingAction)>,
}

impl Default for BindingsLayout {
    fn default() -> Self {
        BindingsLayout::new()
    }
}

impl BindingsLayout {
    pub fn new() -> Self {
        let mut layout = Self {
            layout: default_keyboard_bindings(),
        };
        layout.add_bindings(platform_keyboard_bindings());
        layout
    }

    pub fn add_bindings(
        &mut self,
        bindings: Vec<(Binding<InputKind>, BindingAction)>,
    ) {
        for (binding, action) in bindings {
            match self
                .layout
                .iter()
                .position(|(layout_binding, _)| layout_binding == &binding)
            {
                Some(position) => self.layout[position] = (binding, action),
                None => self.layout.push((binding, action)),
            }
        }
    }

    pub fn get_action(
        &self,
        input: InputKind,
        modifiers: Modifiers,
        terminal_mode: TermMode,
    ) -> BindingAction {
        for (binding, action) in &self.layout {
            let is_trigered = binding.target == input
                && binding.modifiers == modifiers
                && terminal_mode.contains(binding.terminal_mode_include)
                && !terminal_mode.intersects(binding.terminal_mode_exclude);

            if is_trigered {
                return action.clone();
            };
        }

        BindingAction::Ignore
    }
}

fn default_keyboard_bindings() -> Vec<(Binding<InputKind>, BindingAction)> {
    generate_bindings!(
        KeyboardBinding;
        // ANY
        Enter;       BindingAction::Char('\x0d');
        NumpadEnter; BindingAction::Char('\x0d');
        Backspace;   BindingAction::Char('\x7f');
        Escape;      BindingAction::Char('\x1b');
        Tab;         BindingAction::Char('\x09');
        Insert;      BindingAction::Esc("\x1b[2~".into());
        Delete;      BindingAction::Esc("\x1b[3~".into());
        PageUp;      BindingAction::Esc("\x1b[5~".into());
        PageDown;    BindingAction::Esc("\x1b[6~".into());
        F1;          BindingAction::Esc("\x1bOP".into());
        F2;          BindingAction::Esc("\x1bOQ".into());
        F3;          BindingAction::Esc("\x1bOR".into());
        F4;          BindingAction::Esc("\x1bOS".into());
        F5;          BindingAction::Esc("\x1b[15~".into());
        F6;          BindingAction::Esc("\x1b[17~".into());
        F7;          BindingAction::Esc("\x1b[18~".into());
        F8;          BindingAction::Esc("\x1b[19~".into());
        F9;          BindingAction::Esc("\x1b[20~".into());
        F10;         BindingAction::Esc("\x1b[21~".into());
        F11;         BindingAction::Esc("\x1b[23~".into());
        F12;         BindingAction::Esc("\x1b[24~".into());
        F13;         BindingAction::Esc("\x1b[25~".into());
        F14;         BindingAction::Esc("\x1b[26~".into());
        F15;         BindingAction::Esc("\x1b[28~".into());
        F16;         BindingAction::Esc("\x1b[29~".into());
        F17;         BindingAction::Esc("\x1b[31~".into());
        F18;         BindingAction::Esc("\x1b[32~".into());
        F19;         BindingAction::Esc("\x1b[33~".into());
        F20;         BindingAction::Esc("\x1b[34~".into());
        // APP_CURSOR Excluding
        End,   ~TermMode::APP_CURSOR; BindingAction::Esc("\x1b[F".into());
        Home,  ~TermMode::APP_CURSOR; BindingAction::Esc("\x1b[H".into());
        Up,    ~TermMode::APP_CURSOR; BindingAction::Esc("\x1b[A".into());
        Down,  ~TermMode::APP_CURSOR; BindingAction::Esc("\x1b[B".into());
        Left,  ~TermMode::APP_CURSOR; BindingAction::Esc("\x1b[D".into());
        Right, ~TermMode::APP_CURSOR; BindingAction::Esc("\x1b[C".into());
        // APP_CURSOR Including
        End,   +TermMode::APP_CURSOR; BindingAction::Esc("\x1BOF".into());
        Home,  +TermMode::APP_CURSOR; BindingAction::Esc("\x1BOH".into());
        Up,    +TermMode::APP_CURSOR; BindingAction::Esc("\x1bOA".into());
        Down,  +TermMode::APP_CURSOR; BindingAction::Esc("\x1bOB".into());
        Left,  +TermMode::APP_CURSOR; BindingAction::Esc("\x1bOD".into());
        Right, +TermMode::APP_CURSOR; BindingAction::Esc("\x1bOC".into());
        // CTRL
        Up,        Modifiers::COMMAND; BindingAction::Esc("\x1b[1;5A".into());
        Down,      Modifiers::COMMAND; BindingAction::Esc("\x1b[1;5B".into());
        Left,      Modifiers::COMMAND; BindingAction::Esc("\x1b[1;5D".into());
        Right,     Modifiers::COMMAND; BindingAction::Esc("\x1b[1;5C".into());
        End,       Modifiers::CTRL; BindingAction::Esc("\x1b[1;5F".into());
        Home,      Modifiers::CTRL; BindingAction::Esc("\x1b[1;5H".into());
        Delete,    Modifiers::CTRL; BindingAction::Esc("\x1b[3;5~".into());
        PageUp,    Modifiers::CTRL; BindingAction::Esc("\x1b[5;5~".into());
        PageDown,  Modifiers::CTRL; BindingAction::Esc("\x1b[6;5~".into());
        F1,        Modifiers::CTRL; BindingAction::Esc("\x1bO;5P".into());
        F2,        Modifiers::CTRL; BindingAction::Esc("\x1bO;5Q".into());
        F3,        Modifiers::CTRL; BindingAction::Esc("\x1bO;5R".into());
        F4,        Modifiers::CTRL; BindingAction::Esc("\x1bO;5S".into());
        F5,        Modifiers::CTRL; BindingAction::Esc("\x1b[15;5~".into());
        F6,        Modifiers::CTRL; BindingAction::Esc("\x1b[17;5~".into());
        F7,        Modifiers::CTRL; BindingAction::Esc("\x1b[18;5~".into());
        F8,        Modifiers::CTRL; BindingAction::Esc("\x1b[19;5~".into());
        F9,        Modifiers::CTRL; BindingAction::Esc("\x1b[20;5~".into());
        F10,       Modifiers::CTRL; BindingAction::Esc("\x1b[21;5~".into());
        F11,       Modifiers::CTRL; BindingAction::Esc("\x1b[23;5~".into());
        F12,       Modifiers::CTRL; BindingAction::Esc("\x1b[24;5~".into());
        A,         Modifiers::CTRL; BindingAction::Char('\x01');
        B,         Modifiers::CTRL; BindingAction::Char('\x02');
        C,         Modifiers::CTRL; BindingAction::Char('\x03');
        D,         Modifiers::CTRL; BindingAction::Char('\x04');
        E,         Modifiers::CTRL; BindingAction::Char('\x05'); // ENQ               vt100
        F,         Modifiers::CTRL; BindingAction::Char('\x06');
        G,         Modifiers::CTRL; BindingAction::Char('\x07'); // Bell              vt100
        H,         Modifiers::CTRL; BindingAction::Char('\x08'); // Backspace         vt100
        I,         Modifiers::CTRL; BindingAction::Char('\x09'); // Tab               vt100
        J,         Modifiers::CTRL; BindingAction::Char('\x0a'); // LF (new line)     vt100
        K,         Modifiers::CTRL; BindingAction::Char('\x0b'); // VT (vertical tab) vt100
        L,         Modifiers::CTRL; BindingAction::Char('\x0c'); // FF (new page)     vt100
        M,         Modifiers::CTRL; BindingAction::Char('\x0d'); // CR                vt100
        N,         Modifiers::CTRL; BindingAction::Char('\x0e'); // SO (shift out)    vt100
        O,         Modifiers::CTRL; BindingAction::Char('\x0f'); // SI (shift in)     vt100
        P,         Modifiers::CTRL; BindingAction::Char('\x10');
        Q,         Modifiers::CTRL; BindingAction::Char('\x11');
        R,         Modifiers::CTRL; BindingAction::Char('\x12');
        S,         Modifiers::CTRL; BindingAction::Char('\x13');
        T,         Modifiers::CTRL; BindingAction::Char('\x14');
        U,         Modifiers::CTRL; BindingAction::Char('\x51');
        V,         Modifiers::CTRL; BindingAction::Char('\x16');
        W,         Modifiers::CTRL; BindingAction::Char('\x17');
        X,         Modifiers::CTRL; BindingAction::Char('\x18');
        Y,         Modifiers::CTRL; BindingAction::Char('\x19');
        Z,         Modifiers::CTRL; BindingAction::Char('\x1a');
        LBracket,  Modifiers::CTRL; BindingAction::Char('\x1b');
        RBracket,  Modifiers::CTRL; BindingAction::Char('\x1d');
        Backslash, Modifiers::CTRL; BindingAction::Char('\x1c');
        Minus,     Modifiers::CTRL; BindingAction::Char('\x1f');
        // SHIFT
        Enter,       Modifiers::SHIFT; BindingAction::Char('\x0d');
        NumpadEnter, Modifiers::SHIFT; BindingAction::Char('\x0d');
        Backspace,   Modifiers::SHIFT; BindingAction::Char('\x7f');
        Tab,         Modifiers::SHIFT; BindingAction::Esc("\x1b[Z".into());
        End,         Modifiers::SHIFT, +TermMode::ALT_SCREEN; BindingAction::Esc("\x1b[1;2F".into());
        Home,        Modifiers::SHIFT, +TermMode::ALT_SCREEN; BindingAction::Esc("\x1b[1;2H".into());
        PageUp,      Modifiers::SHIFT, +TermMode::ALT_SCREEN; BindingAction::Esc("\x1b[5;2~".into());
        PageDown,    Modifiers::SHIFT, +TermMode::ALT_SCREEN; BindingAction::Esc("\x1b[6;2~".into());
        Up,          Modifiers::SHIFT; BindingAction::Esc("\x1b[1;2A".into());
        Down,        Modifiers::SHIFT; BindingAction::Esc("\x1b[1;2B".into());
        Left,        Modifiers::SHIFT; BindingAction::Esc("\x1b[1;2D".into());
        Right,       Modifiers::SHIFT; BindingAction::Esc("\x1b[1;2C".into());
        // ALT
        Backspace, Modifiers::ALT; BindingAction::Esc("\x1b\x7f".into());
        End,       Modifiers::ALT; BindingAction::Esc("\x1b[1;3F".into());
        Home,      Modifiers::ALT; BindingAction::Esc("\x1b[1;3H".into());
        Insert,    Modifiers::ALT; BindingAction::Esc("\x1b[3;2~".into());
        Delete,    Modifiers::ALT; BindingAction::Esc("\x1b[3;3~".into());
        PageUp,    Modifiers::ALT; BindingAction::Esc("\x1b[5;3~".into());
        PageDown,  Modifiers::ALT; BindingAction::Esc("\x1b[6;3~".into());
        Up,        Modifiers::ALT; BindingAction::Esc("\x1b[1;3A".into());
        Down,      Modifiers::ALT; BindingAction::Esc("\x1b[1;3B".into());
        Left,      Modifiers::ALT; BindingAction::Esc("\x1b[1;3D".into());
        Right,     Modifiers::ALT; BindingAction::Esc("\x1b[1;3C".into());
        // SHIFT + ALT
        End,   Modifiers::SHIFT | Modifiers::ALT; BindingAction::Esc("\x1b[1;4F".into());
        Home,  Modifiers::SHIFT | Modifiers::ALT; BindingAction::Esc("\x1b[1;4H".into());
        Up,    Modifiers::SHIFT | Modifiers::ALT; BindingAction::Esc("\x1b[1;4A".into());
        Down,  Modifiers::SHIFT | Modifiers::ALT; BindingAction::Esc("\x1b[1;4B".into());
        Left,  Modifiers::SHIFT | Modifiers::ALT; BindingAction::Esc("\x1b[1;4D".into());
        Right, Modifiers::SHIFT | Modifiers::ALT; BindingAction::Esc("\x1b[1;4C".into());
        // SHIFT + CTRL
        End,   Modifiers::SHIFT | Modifiers::CTRL; BindingAction::Esc("\x1b[1;6F".into());
        Home,  Modifiers::SHIFT | Modifiers::CTRL; BindingAction::Esc("\x1b[1;6H".into());
        Up,    Modifiers::SHIFT | Modifiers::CTRL; BindingAction::Esc("\x1b[1;6A".into());
        Down,  Modifiers::SHIFT | Modifiers::CTRL; BindingAction::Esc("\x1b[1;6B".into());
        Left,  Modifiers::SHIFT | Modifiers::CTRL; BindingAction::Esc("\x1b[1;6D".into());
        Right, Modifiers::SHIFT | Modifiers::CTRL; BindingAction::Esc("\x1b[1;6C".into());
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
        Key2,  Modifiers::SHIFT | Modifiers::CTRL; BindingAction::Char('\x00'); // Null vt100
        Key6,  Modifiers::SHIFT | Modifiers::CTRL; BindingAction::Char('\x1e');
        Minus, Modifiers::SHIFT | Modifiers::CTRL; BindingAction::Char('\x1f');
        // CTRL + ALT
        End,      Modifiers::CTRL | Modifiers::ALT; BindingAction::Esc("\x1b[1;7F".into());
        Home,     Modifiers::CTRL | Modifiers::ALT; BindingAction::Esc("\x1b[1;7H".into());
        PageUp,   Modifiers::CTRL | Modifiers::ALT; BindingAction::Esc("\x1b[5;7~".into());
        PageDown, Modifiers::CTRL | Modifiers::ALT; BindingAction::Esc("\x1b[6;7~".into());
        Up,       Modifiers::CTRL | Modifiers::ALT; BindingAction::Esc("\x1b[1;7A".into());
        Down,     Modifiers::CTRL | Modifiers::ALT; BindingAction::Esc("\x1b[1;7B".into());
        Left,     Modifiers::CTRL | Modifiers::ALT; BindingAction::Esc("\x1b[1;7D".into());
        Right,    Modifiers::CTRL | Modifiers::ALT; BindingAction::Esc("\x1b[1;7C".into());
        // SHIFT + CTRL + ALT
        End,     Modifiers::SHIFT | Modifiers::CTRL | Modifiers::ALT; BindingAction::Esc("\x1b[1;8F".into());
        Home,    Modifiers::SHIFT | Modifiers::CTRL | Modifiers::ALT; BindingAction::Esc("\x1b[1;8H".into());
        Up,      Modifiers::SHIFT | Modifiers::CTRL | Modifiers::ALT; BindingAction::Esc("\x1b[1;8A".into());
        Down,    Modifiers::SHIFT | Modifiers::CTRL | Modifiers::ALT; BindingAction::Esc("\x1b[1;8B".into());
        Left,    Modifiers::SHIFT | Modifiers::CTRL | Modifiers::ALT; BindingAction::Esc("\x1b[1;8D".into());
        Right,   Modifiers::SHIFT | Modifiers::CTRL | Modifiers::ALT; BindingAction::Esc("\x1b[1;8C".into());
    )
}

#[cfg(target_os = "macos")]
fn platform_keyboard_bindings() -> Vec<(Binding<InputKind>, BindingAction)> {
    generate_bindings!(
        KeyboardBinding;
        C, Modifiers::COMMAND; BindingAction::Copy;
        V, Modifiers::COMMAND; BindingAction::Paste;
    )
}

#[cfg(not(target_os = "macos"))]
fn platform_keyboard_bindings() -> Vec<(Binding<InputKind>, BindingAction)> {
    generate_bindings!(
        KeyboardBinding;
        C, Modifiers::SHIFT | Modifiers::COMMAND; BindingAction::Copy;
        V, Modifiers::SHIFT | Modifiers::COMMAND; BindingAction::Paste;
    )
}

#[cfg(test)]
mod tests {
    use super::{BindingAction, BindingsLayout, InputKind, KeyboardBinding};
    use alacritty_terminal::term::TermMode;
    use iced_core::keyboard::{KeyCode, Modifiers};

    #[test]
    fn add_new_custom_keyboard_binding() {
        let mut current_layout = BindingsLayout::default();
        let custom_bindings = generate_bindings!(
            KeyboardBinding;
            C, Modifiers::SHIFT | Modifiers::ALT; BindingAction::Copy;
        );
        let current_layout_length = current_layout.layout.len();
        let custom_bindings_length = custom_bindings.len();
        current_layout.add_bindings(custom_bindings.clone());
        assert_eq!(
            current_layout.layout.len(),
            current_layout_length + custom_bindings_length
        );
        let found_binding =
            current_layout.layout.iter().find(|(bind, action)| {
                bind == &custom_bindings[0].0 && action == &custom_bindings[0].1
            });
        assert!(found_binding.is_some());
    }

    #[test]
    fn add_many_new_custom_keyboard_bindings() {
        let mut current_layout: BindingsLayout = BindingsLayout::default();
        let custom_bindings = generate_bindings!(
            KeyboardBinding;
            'c', Modifiers::SHIFT, +TermMode::ALT_SCREEN;             BindingAction::Paste;
            C,   Modifiers::SHIFT | Modifiers::ALT;                   BindingAction::Copy;
            W,   Modifiers::ALT;                                      BindingAction::Char('W');
            Q,   Modifiers::SHIFT | Modifiers::CTRL | Modifiers::ALT; BindingAction::Esc("\x1b[1;7C".into());
        );
        let current_layout_length = current_layout.layout.len();
        let custom_bindings_length = custom_bindings.len();
        current_layout.add_bindings(custom_bindings.clone());
        assert_eq!(
            current_layout.layout.len(),
            current_layout_length + custom_bindings_length
        );
        for (custom_bind, custom_action) in custom_bindings {
            let found_binding =
                current_layout.layout.iter().find(|(bind, action)| {
                    bind == &custom_bind && action == &custom_action
                });
            assert!(found_binding.is_some());
        }
    }

    #[test]
    fn add_custom_keyboard_bindings_that_replace_current() {
        let mut current_layout = BindingsLayout::default();
        let custom_bindings = generate_bindings!(
            KeyboardBinding;
            'c',   Modifiers::SHIFT, +TermMode::ALT_SCREEN; BindingAction::Paste;
            A,     Modifiers::SHIFT | Modifiers::CTRL;      BindingAction::Char('A');
            B,     Modifiers::SHIFT | Modifiers::CTRL;      BindingAction::Char('B');
            C,     Modifiers::SHIFT | Modifiers::CTRL;      BindingAction::Copy;
        );
        let current_layout_length = current_layout.layout.len();
        current_layout.add_bindings(custom_bindings.clone());
        assert_eq!(current_layout.layout.len(), current_layout_length + 1);
        for (custom_bind, custom_action) in custom_bindings {
            let found_binding =
                current_layout.layout.iter().find(|(bind, action)| {
                    bind == &custom_bind && action == &custom_action
                });
            assert!(found_binding.is_some());
        }
        let replaced_bindings = generate_bindings!(
            KeyboardBinding;
            A,     Modifiers::SHIFT | Modifiers::CTRL; BindingAction::Char('\x01');
            B,     Modifiers::SHIFT | Modifiers::CTRL; BindingAction::Char('\x02');
            C,     Modifiers::SHIFT | Modifiers::CTRL; BindingAction::Char('\x03');
        );
        for (custom_bind, custom_action) in replaced_bindings {
            let found_binding =
                current_layout.layout.iter().find(|(bind, action)| {
                    bind == &custom_bind && action == &custom_action
                });
            assert!(found_binding.is_none());
        }
    }

    #[test]
    fn get_action() {
        let current_layout = BindingsLayout::default();
        for (bind, action) in &current_layout.layout {
            let found_action = current_layout.get_action(
                bind.target.clone(),
                bind.modifiers,
                bind.terminal_mode_include,
            );
            assert_eq!(action, &found_action);
        }
    }

    #[test]
    fn get_action_with_custom_bindings() {
        let mut current_layout = BindingsLayout::default();
        let custom_bindings = generate_bindings!(
            KeyboardBinding;
            'c',   Modifiers::SHIFT, +TermMode::ALT_SCREEN; BindingAction::Paste;
            A,     Modifiers::SHIFT | Modifiers::CTRL;      BindingAction::Char('A');
            B,     Modifiers::SHIFT | Modifiers::CTRL;      BindingAction::Char('B');
            C,     Modifiers::SHIFT | Modifiers::CTRL;      BindingAction::Copy;
        );
        current_layout.add_bindings(custom_bindings.clone());
        for (bind, action) in &current_layout.layout {
            let found_action = current_layout.get_action(
                bind.target.clone(),
                bind.modifiers,
                bind.terminal_mode_include,
            );
            assert_eq!(action, &found_action);
        }
    }
}
