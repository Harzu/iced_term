use crate::backend::BackendSettings;
use alacritty_terminal::event::Notify;
use alacritty_terminal::event::{EventListener, OnResize, WindowSize};
use alacritty_terminal::event_loop::Notifier;
use alacritty_terminal::grid::Scroll;
use alacritty_terminal::index::{Column, Point, Side};
use alacritty_terminal::selection::{Selection, SelectionRange, SelectionType};
use alacritty_terminal::sync::FairMutex;
use alacritty_terminal::term::cell::Cell;
use alacritty_terminal::term::viewport_to_point;
use alacritty_terminal::term::{test::TermSize, Term, TermMode};
use alacritty_terminal::Grid;
use iced_core::Size;
use std::borrow::Cow;
use std::cmp::min;
use std::io::Result;
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct Pty {
    term: Arc<FairMutex<Term<EventProxy>>>,
    size: WindowSize,
    notifier: Notifier,
}

impl Pty {
    pub fn new(
        id: u64,
        event_sender: mpsc::Sender<alacritty_terminal::event::Event>,
        settings: BackendSettings,
        font_size: Size<f32>,
    ) -> Result<Self> {
        let pty_config = alacritty_terminal::tty::Options {
            shell: Some(alacritty_terminal::tty::Shell::new(
                settings.shell,
                vec![],
            )),
            ..alacritty_terminal::tty::Options::default()
        };
        let config = alacritty_terminal::term::Config::default();
        let window_size = WindowSize {
            cell_width: font_size.width as u16,
            cell_height: font_size.height as u16,
            num_cols: settings.cols,
            num_lines: settings.rows,
        };

        let pty = alacritty_terminal::tty::new(&pty_config, window_size.clone(), id)?;
        let term_size =
            TermSize::new(settings.cols as usize, settings.rows as usize);
        let event_proxy = EventProxy(event_sender);
        let term = Arc::new(FairMutex::new(Term::new(
            config,
            &term_size,
            event_proxy.clone(),
        )));

        let pty_event_loop = alacritty_terminal::event_loop::EventLoop::new(
            term.clone(),
            event_proxy,
            pty,
            false,
            false,
        );
        let notifier = Notifier(pty_event_loop.channel());
        let _pty_join_handle = pty_event_loop.spawn();

        Ok(Self {
            term: term.clone(),
            size: window_size,
            notifier,
        })
    }

    pub fn mode(&self) -> TermMode {
        *self.term.lock().mode()
    }

    pub fn start_selection(
        &mut self,
        selection_type: SelectionType,
        x: f32,
        y: f32,
    ) {
        let mut term = self.term.lock();
        let location = self.selection_point(x, y, term.grid().display_offset());
        term.selection = Some(Selection::new(selection_type, location, self.selection_side(x)))
    }

    pub fn update_selection(&mut self, x: f32, y: f32) {
        let mut term = self.term.lock();
        let display_offset = term.grid().display_offset();
        if let Some(ref mut selection) = term.selection {
            let location = self.selection_point(x, y, display_offset);
            selection.update(location, self.selection_side(x));
        }
    }

    fn selection_point(
        &self,
        x: f32,
        y: f32,
        display_offset: usize,
    ) -> Point {
        let col = (x as usize) / (self.size.cell_width as usize);
        let col = min(Column(col), Column(self.size.num_cols as usize - 1));

        let line = (y as usize) / (self.size.cell_height as usize);
        let line = min(line, self.size.num_lines as usize - 1);

        viewport_to_point(display_offset, Point::new(line, col))
    }

    pub fn get_selection_range(&self) -> Option<SelectionRange> {
        let term = self.term.lock();
        if let Some(selection) = &term.selection {
            return selection.to_range(&term)
        }

        None
    }

    fn selection_side(&self, x: f32) -> Side {
        let cell_x = x as usize % self.size.cell_width as usize;
        let half_cell_width = (self.size.cell_width as f32 / 2.0) as usize;

        if cell_x > half_cell_width {
            Side::Right
        } else {
            Side::Left
        }
    }

    pub fn resize(
        &mut self,
        rows: u16,
        cols: u16,
        font_width: f32,
        font_height: f32,
    ) {
        if rows > 0 && cols > 0 {
            self.size = WindowSize {
                cell_width: font_width as u16,
                cell_height: font_height as u16,
                num_cols: cols as u16,
                num_lines: rows as u16,
            };

            self.notifier.on_resize(self.size.into());
            self.term.lock().resize(TermSize::new(
                self.size.num_cols as usize,
                self.size.num_lines as usize,
            ));
        }
    }

    pub fn size(&self) -> WindowSize {
        self.size
    }

    pub fn write_to_pty<I: Into<Cow<'static, [u8]>>>(&self, input: I) {
        self.notifier.notify(input);
        self.term.lock().scroll_display(Scroll::Bottom);
    }

    pub fn scroll(&mut self, delta_value: i32) {
        if delta_value != 0 {
            let scroll = Scroll::Delta(delta_value);
            let mut term = self.term.lock();
            if term.mode().contains(TermMode::ALTERNATE_SCROLL | TermMode::ALT_SCREEN) {
                let line_cmd = if delta_value > 0 { b'A' } else { b'B' };
                let mut content = vec![];

                for _ in 0..delta_value.abs() {
                    content.push(0x1b);
                    content.push(b'O');
                    content.push(line_cmd);
                }

                self.notifier.notify(content);
            } else {
                term.grid_mut().scroll_display(scroll);
            }
        }
    }

    pub fn renderable_content(&self) -> (Grid<Cell>, Option<SelectionRange>, TermMode, WindowSize) {
        let term = self.term.lock();
        let mut selectable_range = None;
        if let Some(selection) = &term.selection {
            selectable_range = selection.to_range(&term)
        }

        (
            term.grid().clone(),
            selectable_range,
            *term.mode(),
            self.size
        )
    }
}

impl Drop for Pty {
    fn drop(&mut self) {
        let _ = self
            .notifier
            .0
            .send(alacritty_terminal::event_loop::Msg::Shutdown);
    }
}

#[derive(Clone)]
pub struct EventProxy(mpsc::Sender<alacritty_terminal::event::Event>);

impl EventListener for EventProxy {
    fn send_event(&self, event: alacritty_terminal::event::Event) {
        let _ = self.0.blocking_send(event);
    }
}
