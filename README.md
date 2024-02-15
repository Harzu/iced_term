<div align="center">

<img src="docs/iced_logo.svg" width="140px" />
<img src="docs/iced_term_logo.png" width="140px" />

# iced_term

![GitHub License](https://img.shields.io/github/license/Harzu/iced_term)
![Crates.io Downloads (recent)](https://img.shields.io/crates/dr/iced_term)

Terminal emulator widget powered by ICED framework and alacritty terminal backend.

<a href="./examples/full_screen">
  <img src="examples/full_screen/assets/screenshot.png" width="275px">
</a>
<a href="./examples/split_view/assets/screenshot.png">
  <img src="examples/split_view/assets/screenshot.png" width="273px">
</a>

</div>

## Features

The widget is currently under development and does not provide full terminal features make sure that widget is covered everything you want.

- PTY content rendering
- Multiple instance support
- Basic keyboard input
- Adding custom keyboard or mouse bindings
- Resizing
- Scrolling
- Focusing
- Selecting
- Changing Font/Color scheme
- Hyperlinks processing (hover/open)

This widget tested on MacOS and Linux and is not tested on Windows.

## Installation

```toml
iced_term = "0.3.0"
```

## Overview

Interacting with the widget is happened via:

**Commands** - you can send commands to widget for changing the widget state.

```rust
#[derive(Debug, Clone)]
pub enum Command {
    InitBackend(Sender<alacritty_terminal::event::Event>),
    ChangeTheme(Box<ColorPalette>),
    ChangeFont(FontSettings),
    AddBindings(Vec<(Binding<InputKind>, BindingAction)>),
    ProcessBackendCommand(BackendCommand),
}
```

**Events** - widget is produced some events that can be handled in application. Every event has the first `u64` argument that is **terminal instance id**.

```rust
#[derive(Debug, Clone)]
pub enum Event {
    CommandReceived(u64, Command),
}
```

Right now there is the only internal **CommandReceived** event that is needed for backend <-> view communication. You can also handle this event unwrap the command and process command additionally if you want.

**Actions** - widget's method `update(&mut self, cmd: Command)` returns **Action** that you can handle after widget updated.

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Redraw,
    Shutdown,
    ChangeTitle,
    Ignore,
}
```

For creating workable widget instance you need to do a few steps:

Add widget to your app struct

```rust
struct App {
    term: iced_term::Term,
}
```

Create pure instance in app constructor

```rust
impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let system_shell = std::env::var("SHELL")
            .expect("SHELL variable is not defined")
            .to_string();
        let term_id = 0;
        let term_settings = iced_term::TermSettings {
            font: iced_term::FontSettings {
                size: 14.0,
                ..iced_term::FontSettings::default()
            },
            theme: iced_term::ColorPalette::default(),
            backend: iced_term::BackendSettings {
                shell: system_shell.to_string(),
            },
        };

        (
            Self {
                term: iced_term::Term::new(term_id, term_settings.clone()),
            },
            Command::none(),
        )
    }
}
```

Add message that contained widget events to application message enum

```rust
#[derive(Debug, Clone)]
pub enum Message {
    // ... other messages
    IcedTermEvent(iced_term::Event),
}
```

Add **IcedTermEvent** processing to application `update` method

```rust
impl Application for App {
    // ... other methods
    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match message {
            Message::IcedTermEvent(iced_term::Event::CommandReceived(
                _,
                cmd,
            )) => match self.term.update(cmd) {
                iced_term::actions::Action::Shutdown => window::close(),
                _ => Command::none(),
            },
        }
    }
}
```

Add view to your application

```rust
impl Application for App {
    // ... other methods
    fn view(&self) -> Element<Message, iced::Renderer> {
        container(iced_term::term_view(&self.term).map(Message::IcedTermEvent))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
```

Activate backend events subscription in your app

```rust
impl Application for App {
    // ... other methods
    fn subscription(&self) -> Subscription<Message> {
        self.term.subscription().map(Message::IcedTermEvent)
    }
}
```

Make main func

```rust
fn main() -> iced::Result {
    App::run(Settings {
        window: window::Settings {
            size: (1280, 720),
            ..window::Settings::default()
        },
        ..Settings::default()
    })
}
```

Run your application

```shell
cargo run --release
```

## Examples

You can also look at [examples](./examples) directory for more information about widget using.

- [full_screen](./examples/full_screen/) - The basic example of terminal emulator.
- [split_view](./examples/split_view/) - The example based on split_view iced widget that show how multiple instance feature work.
- [custom_bindings](./examples/custom_bindings/) - The example that show how you can add custom keyboard or mouse bindings to your terminal emulator app.
- [themes](./examples/themes/) - The example that show how you can change terminal color scheme.
- [fonts](./examples/fonts/) - The examples that show how you can change font type or font size in your terminal emulator app.

## Dependencies

 - [iced (0.10.0)](https://github.com/iced-rs/iced/tree/master)
 - [alacritty_terminal (0.21.0)](https://github.com/alacritty/alacritty/tree/master/alacritty_terminal)
 - [tokio (1.23.0)](https://github.com/tokio-rs/tokio)
 - [open (5)](https://github.com/Byron/open-rs)