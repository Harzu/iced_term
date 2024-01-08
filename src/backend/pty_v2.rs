use crate::backend::BackendSettings;
use crate::backend::RenderableCell;
use alacritty_terminal::event::Notify;
use alacritty_terminal::event::{EventListener, OnResize, WindowSize};
use alacritty_terminal::event_loop::Notifier;
use alacritty_terminal::grid::Scroll;
use alacritty_terminal::sync::FairMutex;
use alacritty_terminal::term::{cell, test::TermSize};
use std::borrow::Cow;
use std::io::Result;
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct PtyV2 {
    _id: u64,
    term: Arc<FairMutex<alacritty_terminal::Term<EventProxy>>>,
    notifier: Notifier,
}

impl PtyV2 {
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
            self.term.lock().resize(TermSize::new(
                size.num_cols as usize,
                size.num_lines as usize,
            ));
        }
    }

    pub fn write_to_pty<I: Into<Cow<'static, [u8]>>>(&self, input: I) {
        self.notifier.notify(input);
        self.term.lock().scroll_display(Scroll::Bottom);
    }

    pub fn scroll(&mut self, delta_value: i32) {
        let scroll = Scroll::Delta(delta_value);
        self.term.lock().scroll_display(scroll);
    }

    pub fn cells(&self) -> Vec<RenderableCell> {
        let mut res = vec![];
        let term = self.term.lock_unfair();

        let content = term.renderable_content();
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

impl Drop for PtyV2 {
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
        //TODO: handle error
        let _ = self.0.blocking_send(event);
    }
}