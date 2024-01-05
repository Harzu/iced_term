use std::fs::File;
use std::io::Write;
use std::io::Result;
use alacritty_terminal::tty::EventedReadWrite;
use alacritty_terminal::vte::ansi;
use alacritty_terminal::event::{EventListener, OnResize, WindowSize};
use alacritty_terminal::term::{test::TermSize, cell};
use tokio::io::AsyncReadExt;
use crate::backend::RenderableCell;
use crate::backend::Settings;

pub struct Pty {
    id: u64,
    pty: alacritty_terminal::tty::Pty,
    term: alacritty_terminal::Term<EventProxy>,
    reader: File,
    parser: ansi::Processor,
}

impl Pty {
    pub fn new(id: u64, settings: Settings) -> Result<Self> {
        let mut pty_config = alacritty_terminal::tty::Options::default();
        pty_config.shell = Some(alacritty_terminal::tty::Shell::new(settings.shell, vec![]));
        let config = alacritty_terminal::term::Config::default();
        let window_size = alacritty_terminal::event::WindowSize {
            cell_width: 13,
            cell_height: 20,
            num_cols: settings.cols,
            num_lines: settings.rows,
        };

        let mut pty = alacritty_terminal::tty::new(&pty_config, window_size, id)?;
        let term_size = TermSize::new(settings.cols as usize, settings.rows as usize);
        let reader = pty.reader().try_clone()?;

        Ok(Self {
            id,
            pty,
            reader,
            term: alacritty_terminal::Term::new(config, &term_size, EventProxy {}),
            parser: ansi::Processor::new()
        })
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub async fn try_read(reader: &File) -> Option<Vec<u8>> {
        let mut file = tokio::fs::File::from(reader.try_clone().unwrap());
        let mut buf = [0; 4096];
        match file.read(&mut buf).await {
            Ok(_) => Some(buf.to_vec()),
            _ => None
        }
    }

    // pub fn resize(&mut self, rows: u16, cols: u16) {
    //     let size = WindowSize {
    //         cell_width: 1,
    //         cell_height: 1,
    //         num_cols: cols,
    //         num_lines: rows,
    //     };

    //     self.pty.on_resize(size);
    //     self.term.resize(TermSize::new(
    //         size.num_cols as usize,
    //         size.num_lines as usize,
    //     ));
    // }

    pub fn resize(
        &mut self,
        container_width: u32,
        container_height: u32,
        padding: u16,
        font_width: f32,
        font_height: f32,
    ) {
        let container_width = container_width.max(1) - padding as u32;
        let container_height = container_height.max(1) - padding as u32;
        let rows = (container_height as f32 / font_height).round() as u16;
        let cols = (container_width as f32 / font_width).round() as u16;

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

            // if cell.flags.contains(cell::Flags::DIM) || cell.flags.contains(cell::Flags::DIM_BOLD) {
            //     fg = ansi::Color::(fg.r(), fg.g(), fg.b(), 66);
            // }

            if cell.flags.contains(cell::Flags::INVERSE) {
                let clone_fg = fg.clone();
                fg = bg;
                bg = clone_fg;
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
    fn send_event(&self, _: alacritty_terminal::event::Event) {}
}
