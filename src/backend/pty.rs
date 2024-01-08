use crate::backend::BackendSettings;
use crate::backend::RenderableCell;
use alacritty_terminal::event::{EventListener, OnResize, WindowSize};
use alacritty_terminal::grid::Scroll;
use alacritty_terminal::term::{cell, test::TermSize};
use alacritty_terminal::tty::EventedReadWrite;
use alacritty_terminal::vte::ansi;
use std::fs::File;
use std::io::Write;
use std::io::{Read, Result};
use std::sync::Arc;

pub struct Pty {
    _id: u64,
    pub pty: alacritty_terminal::tty::Pty,
    term: alacritty_terminal::Term<EventProxy>,
    reader: File,
    parser: ansi::Processor,
    pub poller: Arc<polling::Poller>,
    pub read_interest: polling::Event,
}

impl Drop for Pty {
    fn drop(&mut self) {
        println!("drop");
        self.pty.deregister(&self.poller).unwrap();
        // self.poller.delete(&self.reader).unwrap();
    }
}

impl Pty {
    pub fn new(id: u64, settings: BackendSettings) -> Result<Self> {
        let pty_config = alacritty_terminal::tty::Options {
            shell: Some(alacritty_terminal::tty::Shell::new(
                settings.shell,
                vec![],
            )),
            ..alacritty_terminal::tty::Options::default()
        };
        let config = alacritty_terminal::term::Config::default();
        let window_size = alacritty_terminal::event::WindowSize {
            cell_width: 1,
            cell_height: 1,
            num_cols: settings.cols,
            num_lines: settings.rows,
        };

        let mut pty =
            alacritty_terminal::tty::new(&pty_config, window_size, id)?;
        let term_size =
            TermSize::new(settings.cols as usize, settings.rows as usize);
        let reader = pty.reader().try_clone()?;
        let term =
            alacritty_terminal::Term::new(config, &term_size, EventProxy {});

        let poller = Arc::new(polling::Poller::new()?);
        let interest = polling::Event::readable(id as usize);

        unsafe {
            poller.add_with_mode(
                &reader,
                interest,
                polling::PollMode::Level,
            )?;
            pty.register(&poller, interest, polling::PollMode::Level)?;
        }

        Ok(Self {
            _id: id,
            pty,
            reader,
            term,
            parser: ansi::Processor::new(),
            poller,
            read_interest: interest,
        })
    }

    pub unsafe fn poller(&self) -> Arc<polling::Poller> {
        self.poller.to_owned()
    }

    // pub fn read(reader: &File, buf: &mut [u8]) -> Option<Vec<u8>> {
    //     match reader.try_clone().unwrap().read(buf) {
    //         Ok(n) => Some(buf[..n].to_vec()),
    //         Err(_) => None,
    //     }
    // }

    // pub fn read(reader: &File, buf: &mut [u8]) -> Result<Vec<u8>> {
    //     match reader.try_clone().unwrap().read(buf) {
    //         Ok(n) => Some(buf[..n].to_vec()),
    //         Err(_) => None,
    //     }
    // }

    pub fn resize(
        &mut self,
        rows: u16,
        cols: u16,
        font_width: f32,
        font_height: f32,
    ) -> Vec<RenderableCell> {
        if rows > 0 && cols > 0 {
            let size = WindowSize {
                cell_width: font_width as u16,
                cell_height: font_height as u16,
                num_cols: cols,
                num_lines: rows,
            };

            self.pty.on_resize(size);
            self.term.resize(TermSize::new(
                size.num_cols as usize,
                size.num_lines as usize,
            ));
        }

        self.cells()
    }

    pub fn scroll(&mut self, delta_value: i32) -> Vec<RenderableCell> {
        let scroll = Scroll::Delta(delta_value);
        self.term.scroll_display(scroll);
        self.cells()
    }

    pub fn reader(&self) -> File {
        self.reader.try_clone().unwrap()
    }

    pub fn update(&mut self, data: Vec<u8>) -> Vec<RenderableCell> {
        data.iter().for_each(|item| {
            self.parser.advance(&mut self.term, *item);
        });

        self.cells()
    }

    pub fn write_to_pty(&mut self, c: char) {
        self.term.scroll_display(Scroll::Bottom);
        self.pty.writer().write_all(&[c as u8]).unwrap();
    }

    pub fn cells(&self) -> Vec<RenderableCell> {
        let mut res = vec![];
        let content = self.term.renderable_content();

        for item in content.display_iter {
            let point = item.point;
            let cell = item.cell;
            let mut fg = cell.fg;
            let mut bg = cell.bg;

            if cell.flags.contains(cell::Flags::INVERSE) {
                std::mem::swap(&mut fg, &mut bg);
            }

            res.push(RenderableCell {
                column: point.column.0,
                line: point.line.0,
                content: cell.c,
                display_offset: content.display_offset,
                fg,
                bg,
            })
        }

        res
    }
}

#[derive(Clone)]
struct EventProxy;

impl EventProxy {}

impl EventListener for EventProxy {
    fn send_event(&self, e: alacritty_terminal::event::Event) {
        println!("{:?}", e);
    }
}
