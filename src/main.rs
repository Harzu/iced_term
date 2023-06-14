use alacritty_terminal::term::cell::Flags;
use iced::widget::canvas::Path;
use iced::widget::container;
use iced::{
    executor, window, Application, Color, Command, Element, Event, Font, Length, Point, Rectangle,
    Renderer, Settings, Size, Theme,
};
use iced::{
    keyboard,
    widget::canvas::{self, Cache, Canvas, Cursor, Frame, Geometry},
};
use iced::{widget::canvas::Text, Subscription};
use iced_native;
use std::fs::File;

mod font;
mod terminal;

fn main() -> iced::Result {
    App::run(Settings {
        antialiasing: true,
        try_opengles_first: false,
        window: window::Settings {
            size: (800, 600),
            ..window::Settings::default()
        },
        ..Settings::default()
    })
}

#[derive(Debug, Clone)]
enum Message {
    NewTerminalData(Vec<u8>),
    EventOccurred(iced_native::Event),
}

struct App {
    terminal: terminal::Terminal,
    cache: Cache,
    reader: File,
    width: u32,
    height: u32,
}

impl App {
    fn sub(reader: File) -> Subscription<Message> {
        iced_native::subscription::unfold("t", reader, move |reader| async move {
            std::thread::sleep(std::time::Duration::from_millis(1));
            let mut local_reader = reader.try_clone().unwrap();
            if let Some(data) = terminal::Terminal::read_sync(&mut local_reader) {
                return (Some(Message::NewTerminalData(data)), reader);
            }

            (None, reader)
        })
    }
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let mut terminal = terminal::Terminal::new("/bin/bash".to_string());
        let terminal_reader = terminal.new_reader();

        (
            App {
                terminal,
                cache: Cache::default(),
                reader: terminal_reader,
                width: 800,
                height: 600,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Terminal app")
    }

    fn update(&mut self, _message: Message) -> Command<Message> {
        match _message {
            Message::NewTerminalData(data) => {
                self.terminal.update(data);
                self.cache.clear();
                Command::none()
            }
            Message::EventOccurred(event) => {
                match event {
                    iced_native::Event::Window(e) => match e {
                        iced_native::window::Event::Resized { width, height } => {
                            if width != self.width || height != self.height {
                                self.height = height;
                                self.width = width;

                                let width = width.max(1);
                                let height = height.max(1);

                                let h = (height as f32 / 20.0).round() as u16;
                                let w = (width as f32 / 13.0).round() as u16;

                                println!("{}|{} (rows {} cols {})", height, width, h, w);
                                self.terminal.resize(h as u16, w as u16);
                                self.cache.clear();
                            }
                        }
                        _ => {}
                    },
                    iced_native::Event::Keyboard(e) => match e {
                        iced_native::keyboard::Event::CharacterReceived(c) => {
                            self.terminal.write_to_pty(c);
                            self.cache.clear();
                        }
                        _ => {}
                    },
                    _ => {}
                };

                Command::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut sb = vec![];
        let reader = self.reader.try_clone().unwrap();
        let output = App::sub(reader);
        let events = iced_native::subscription::events().map(Message::EventOccurred);
        sb.push(output);
        sb.push(events);
        Subscription::batch(sb)
    }

    fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
        // iced_native::text::Renderer::measure_width(
        //     &self,
        //     "W",
        //     20.0,
        //     Font::External {
        //         name: "Nerd",
        //         bytes: include_bytes!("../fonts/Hack Regular Nerd Font Complete.ttf"),
        //     },
        // );
        let canvas = Canvas::new(self).width(Length::Fill).height(Length::Fill);
        container(canvas)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl<Message> canvas::Program<Message> for App {
    type State = ();
    // type Renderer = iced_native::text::Renderer;

    fn draw(
        &self,
        _state: &Self::State,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        // iced_native::text::Renderer::measure(&mut self, text);

        let geom = self.cache.draw(bounds.size(), |frame| {
            let cell_width = 13.0;
            let cell_height = 20.0;

            let content = self.terminal.content();
            for item in content.display_iter {
                let point = item.point;
                let cell = item.cell;

                let x = point.column.0 as f64 * cell_width;
                let y = (point.line.0 as f64 + content.display_offset as f64) * cell_height;

                let mut fg = font::get_color(cell.fg);
                let mut bg = font::get_color(cell.bg);

                if cell.flags.contains(Flags::DIM) || cell.flags.contains(Flags::DIM_BOLD) {
                    fg = Color::from_rgba(fg.r, fg.g, fg.b, 0.66);
                }

                let inverse = cell.flags.contains(Flags::INVERSE);
                if inverse {
                    let clone_fg = fg.clone();
                    fg = bg;
                    bg = clone_fg;
                }

                let size = Size::new(cell_width as f32, cell_height as f32);
                let background = Path::rectangle(
                    Point {
                        x: x as f32,
                        y: y as f32,
                    },
                    size,
                );
                frame.fill(&background, bg);

                if cell.c != ' ' && cell.c != '\t' {
                    let text = Text {
                        content: cell.c.to_string(),
                        position: Point {
                            x: x as f32,
                            y: y as f32,
                        },
                        font: Font::External {
                            name: "Nerd",
                            bytes: include_bytes!("../fonts/Hack Regular Nerd Font Complete.ttf"),
                        },
                        size: 20.0,
                        color: fg,
                        ..Text::default()
                    };

                    frame.fill_text(text);
                }
            }
        });

        vec![geom]
    }
}
