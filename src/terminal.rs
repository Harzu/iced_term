use alacritty_terminal::{
    ansi::{self, Processor},
    event::{EventListener, OnResize, WindowSize},
    term::{test::TermSize, RenderableContent},
    tty::{EventedReadWrite, Pty},
    Term,
};
use std::{
    fs::File,
    io::{Read, Write},
};

#[derive(Clone)]
pub struct EventProxy;

impl EventProxy {}

impl EventListener for EventProxy {
    fn send_event(&self, _: alacritty_terminal::event::Event) {}
}

pub struct Terminal {
    tty: Pty,
    term: Term<EventProxy>,
    parser: Processor,
}

impl Terminal {
    pub fn new(shell: String) -> Self {
        let mut config = alacritty_terminal::config::Config::default();

        config.pty_config.shell = Some(alacritty_terminal::config::Program::WithArgs {
            program: shell,
            args: vec![],
        });

        let size = WindowSize {
            cell_width: 1,
            cell_height: 1,
            num_cols: 100,
            num_lines: 50,
        };
        let term_size = TermSize::new(100, 50);
        let event_proxy = EventProxy {};
        let tty = alacritty_terminal::tty::new(&config.pty_config, size, 0).unwrap();
        let term = alacritty_terminal::Term::new(&config, &term_size, event_proxy);
        let parser = ansi::Processor::new();

        Self { tty, term, parser }
    }

    pub fn resize(&mut self, rows: u16, cols: u16) {
        let size = WindowSize {
            cell_width: 1,
            cell_height: 1,
            num_cols: cols,
            num_lines: rows,
        };

        self.tty.on_resize(size);
        self.term.resize(TermSize::new(
            size.num_cols as usize,
            size.num_lines as usize,
        ));
    }

    pub fn new_reader(&mut self) -> File {
        self.tty.reader().try_clone().unwrap()
    }

    pub fn update(&mut self, data: Vec<u8>) {
        for item in data.to_vec() {
            self.parser.advance(&mut self.term, item);
        }
    }

    pub fn read_sync(reader: &mut File) -> Option<Vec<u8>> {
        let mut buf = [0; 4096];
        if let Ok(_) = reader.read(&mut buf) {
            return Some(buf.to_vec());
        };
        None
    }

    pub fn write_to_pty(&mut self, c: char) {
        self.tty.writer().write_all(&[c as u8]).unwrap();
    }

    pub fn content(&self) -> RenderableContent {
        self.term.renderable_content()
    }
}
