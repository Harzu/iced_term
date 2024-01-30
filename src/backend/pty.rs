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
use alacritty_terminal::term::{test::TermSize, TermMode};
use alacritty_terminal::Grid;
use std::borrow::Cow;
use std::io::Result;
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct Pty {
    _id: u64,
    term: Arc<FairMutex<alacritty_terminal::Term<EventProxy>>>,
    notifier: Notifier,
}

impl Pty {
    pub fn new(
        id: u64,
        event_sender: mpsc::Sender<alacritty_terminal::event::Event>,
        settings: BackendSettings,
    ) -> Result<Self> {
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

        let pty = alacritty_terminal::tty::new(&pty_config, window_size, id)?;
        let term_size =
            TermSize::new(settings.cols as usize, settings.rows as usize);
        let event_proxy = EventProxy(event_sender);
        let term = Arc::new(FairMutex::new(alacritty_terminal::Term::new(
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
            _id: id,
            term: term.clone(),
            notifier,
        })
    }

    pub fn is_mode(&self, mode: TermMode) -> bool {
        self.term.lock_unfair().mode().contains(mode)
    }

    pub fn mode(&self) -> TermMode {
        *self.term.lock_unfair().mode()
    }

    pub fn start_selection(
        &mut self,
        selection_type: SelectionType,
        x: f32,
        y: f32,
        cell_width: f32,
        cell_height: f32,
    ) {
        let mut term = self.term.lock_unfair();
        let col = x / cell_width;
        let row = y / cell_height;
        let location = viewport_to_point(term.grid().display_offset(), Point::new(
            row as usize,
            Column(col as usize),
        ));
        let side = if col.fract() < 0.5 {
            Side::Left
        } else {
            Side::Right
        };

        println!("start");
        term.selection = Some(Selection::new(selection_type, location, side))
    }

    pub fn update_selection(
        &mut self,
        x: f32,
        y: f32,
        cell_width: f32,
        cell_height: f32,
    ) {
        let mut term = self.term.lock_unfair();
        let display_offset = term.grid().display_offset();
        if let Some(ref mut selection) = term.selection {
            println!("update");
            let col = x / cell_width;
            let row = y / cell_height;
            let location = viewport_to_point(display_offset, Point::new(
                row as usize,
                Column(col as usize),
            ));
            let side = if col.fract() < 0.5 {
                Side::Left
            } else {
                Side::Right
            };
            selection.update(location, side);
        }
    }

    pub fn get_selection_range(&self) -> Option<SelectionRange> {
        let term = self.term.lock_unfair();
        if let Some(selection) = &term.selection {
            return selection.to_range(&term)
        }

        None
    }

    pub fn resize(
        &mut self,
        rows: u16,
        cols: u16,
        font_width: f32,
        font_height: f32,
    ) {
        if rows > 0 && cols > 0 {
            let size = WindowSize {
                cell_width: font_width as u16,
                cell_height: font_height as u16,
                num_cols: cols,
                num_lines: rows,
            };

            self.notifier.on_resize(size);
            self.term.lock_unfair().resize(TermSize::new(
                size.num_cols as usize,
                size.num_lines as usize,
            ));
        }
    }

    pub fn write_to_pty<I: Into<Cow<'static, [u8]>>>(&self, input: I) {
        self.notifier.notify(input);
        self.term.lock_unfair().scroll_display(Scroll::Bottom);
    }

    pub fn scroll(&mut self, delta_value: i32) {
        if delta_value != 0 {
            let scroll = Scroll::Delta(delta_value);
            self.term.lock_unfair().grid_mut().scroll_display(scroll);
        }
    }

    pub fn renderable_content(&self) -> Grid<Cell> {
        let term = self.term.lock_unfair();
        term.grid().clone()
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
