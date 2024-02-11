pub mod settings;

use alacritty_terminal::event::{
    Event, EventListener, Notify, OnResize, WindowSize,
};
use alacritty_terminal::event_loop::{EventLoop, Msg, Notifier};
use alacritty_terminal::grid::{Dimensions, Scroll};
use alacritty_terminal::index::{Column, Line, Point, Side};
use alacritty_terminal::selection::{Selection, SelectionRange, SelectionType};
use alacritty_terminal::sync::FairMutex;
use alacritty_terminal::term::{
    self, cell::Cell, test::TermSize, viewport_to_point, Term, TermMode,
};
use alacritty_terminal::{tty, Grid};
use iced_core::Size;
use settings::BackendSettings;
use std::borrow::Cow;
use std::cmp::min;
use std::io::Result;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::actions::Action;

#[derive(Debug, Clone)]
pub enum BackendCommand {
    Write(Vec<u8>),
    Scroll(i32),
    Resize(Size<f32>),
    SelectStart(SelectionType, (f32, f32)),
    SelectUpdate((f32, f32)),
    ProcessAlacrittyEvent(Event),
}

#[derive(Clone, Copy, Debug)]
pub struct TerminalSize {
    pub cell_width: u16,
    pub cell_height: u16,
    num_cols: u16,
    num_lines: u16,
}

impl Default for TerminalSize {
    fn default() -> Self {
        Self {
            cell_height: 0,
            cell_width: 0,
            num_cols: 80,
            num_lines: 50,
        }
    }
}

impl Dimensions for TerminalSize {
    fn total_lines(&self) -> usize {
        self.screen_lines()
    }

    fn columns(&self) -> usize {
        self.num_cols as usize
    }

    fn last_column(&self) -> Column {
        Column(self.num_cols as usize - 1)
    }

    fn bottommost_line(&self) -> Line {
        Line(self.num_lines as i32 - 1)
    }

    fn screen_lines(&self) -> usize {
        self.num_lines as usize
    }
}

impl From<TerminalSize> for WindowSize {
    fn from(size: TerminalSize) -> Self {
        Self {
            num_lines: size.num_lines,
            num_cols: size.num_cols,
            cell_width: size.cell_width,
            cell_height: size.cell_height,
        }
    }
}

pub struct Backend {
    term: Arc<FairMutex<Term<EventProxy>>>,
    size: TerminalSize,
    notifier: Notifier,
    last_content: RenderableContent,
}

impl Backend {
    pub fn new(
        id: u64,
        event_sender: mpsc::Sender<Event>,
        settings: BackendSettings,
        font_size: Size<f32>,
    ) -> Result<Self> {
        let pty_config = tty::Options {
            shell: Some(tty::Shell::new(settings.shell, vec![])),
            ..tty::Options::default()
        };
        let config = term::Config::default();
        let terminal_size = TerminalSize {
            cell_width: font_size.width as u16,
            cell_height: font_size.height as u16,
            ..TerminalSize::default()
        };

        let pty = tty::new(&pty_config, terminal_size.into(), id)?;
        let event_proxy = EventProxy(event_sender);

        let mut term = Term::new(config, &terminal_size, event_proxy.clone());
        let cursor = term.grid_mut().cursor_cell().clone();
        let initial_content = RenderableContent {
            grid: term.grid().clone(),
            selectable_range: None,
            terminal_mode: *term.mode(),
            terminal_size,
            cursor: cursor.clone(),
        };

        let term = Arc::new(FairMutex::new(term));
        let pty_event_loop =
            EventLoop::new(term.clone(), event_proxy, pty, false, false);
        let notifier = Notifier(pty_event_loop.channel());
        let _pty_join_handle = pty_event_loop.spawn();

        Ok(Self {
            term: term.clone(),
            size: terminal_size,
            notifier,
            last_content: initial_content,
        })
    }

    pub fn process_command(&mut self, cmd: BackendCommand) -> Action {
        let mut action = Action::Ignore;
        let term = self.term.clone();
        let mut term = term.lock();
        match cmd {
            BackendCommand::ProcessAlacrittyEvent(event) => {
                match event {
                    Event::Wakeup => {
                        self.internal_sync(&mut term);
                        action = Action::Redraw;
                    },
                    Event::Exit => {
                        action = Action::Shutdown;
                    },
                    _ => {},
                };
            },
            BackendCommand::Write(input) => {
                self.write(input);
                term.scroll_display(Scroll::Bottom);
            },
            BackendCommand::Scroll(delta) => {
                self.scroll(&mut term, delta);
                action = Action::Redraw;
            },
            BackendCommand::Resize(size) => {
                self.resize(
                    &mut term,
                    size.width,
                    size.height,
                    self.size.cell_width,
                    self.size.cell_height,
                );
            },
            BackendCommand::SelectStart(selection_type, (x, y)) => {
                self.start_selection(&mut term, selection_type, x, y);
                action = Action::Redraw;
            },
            BackendCommand::SelectUpdate((x, y)) => {
                self.update_selection(&mut term, x, y);
                action = Action::Redraw;
            },
        };

        action
    }

    pub fn start_selection(
        &mut self,
        terminal: &mut Term<EventProxy>,
        selection_type: SelectionType,
        x: f32,
        y: f32,
    ) {
        let location =
            self.selection_point(x, y, terminal.grid().display_offset());
        terminal.selection = Some(Selection::new(
            selection_type,
            location,
            self.selection_side(x),
        ));
        self.internal_sync(terminal);
    }

    pub fn update_selection(
        &mut self,
        terminal: &mut Term<EventProxy>,
        x: f32,
        y: f32,
    ) {
        let display_offset = terminal.grid().display_offset();
        if let Some(ref mut selection) = terminal.selection {
            let location = self.selection_point(x, y, display_offset);
            selection.update(location, self.selection_side(x));
            self.internal_sync(terminal);
        }
    }

    fn selection_point(&self, x: f32, y: f32, display_offset: usize) -> Point {
        let col = (x as usize) / (self.size.cell_width as usize);
        let col = min(Column(col), Column(self.size.num_cols as usize - 1));

        let line = (y as usize) / (self.size.cell_height as usize);
        let line = min(line, self.size.num_lines as usize - 1);

        viewport_to_point(display_offset, Point::new(line, col))
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
        terminal: &mut Term<EventProxy>,
        layout_width: f32,
        layout_height: f32,
        cell_width: u16,
        cell_height: u16,
    ) {
        let rows = (layout_height / cell_height as f32).floor() as u16;
        let cols = (layout_width / cell_width as f32).floor() as u16;
        if rows > 0 && cols > 0 {
            self.size = TerminalSize {
                cell_width,
                cell_height,
                num_cols: cols,
                num_lines: rows,
            };

            self.notifier.on_resize(self.size.into());
            terminal.resize(TermSize::new(
                self.size.num_cols as usize,
                self.size.num_lines as usize,
            ));
        }
    }

    pub fn write<I: Into<Cow<'static, [u8]>>>(&self, input: I) {
        self.notifier.notify(input);
    }

    pub fn scroll(
        &mut self,
        terminal: &mut Term<EventProxy>,
        delta_value: i32,
    ) {
        if delta_value != 0 {
            let scroll = Scroll::Delta(delta_value);
            if terminal
                .mode()
                .contains(TermMode::ALTERNATE_SCROLL | TermMode::ALT_SCREEN)
            {
                let line_cmd = if delta_value > 0 { b'A' } else { b'B' };
                let mut content = vec![];

                for _ in 0..delta_value.abs() {
                    content.push(0x1b);
                    content.push(b'O');
                    content.push(line_cmd);
                }

                self.notifier.notify(content);
            } else {
                terminal.grid_mut().scroll_display(scroll);
            }
            self.internal_sync(terminal);
        }
    }

    pub fn selectable_content(&self) -> String {
        let content = self.renderable_content();
        let mut result = String::new();
        if let Some(range) = content.selectable_range {
            for indexed in content.grid.display_iter() {
                if range.contains(indexed.point) {
                    result.push(indexed.c);
                }
            }
        }
        result
    }

    pub fn sync(&mut self) {
        let term = self.term.clone();
        let mut term = term.lock();
        self.internal_sync(&mut term);
    }

    fn internal_sync(&mut self, terminal: &mut Term<EventProxy>) {
        let selectable_range = match &terminal.selection {
            Some(s) => s.to_range(terminal),
            None => None,
        };

        let cursor = terminal.grid_mut().cursor_cell().clone();
        self.last_content = RenderableContent {
            grid: terminal.grid().clone(),
            selectable_range,
            cursor: cursor.clone(),
            terminal_mode: *terminal.mode(),
            terminal_size: self.size,
        }
    }

    pub fn renderable_content(&self) -> &RenderableContent {
        &self.last_content
    }
}

pub struct RenderableContent {
    pub grid: Grid<Cell>,
    pub selectable_range: Option<SelectionRange>,
    pub cursor: Cell,
    pub terminal_mode: TermMode,
    pub terminal_size: TerminalSize,
}

impl Drop for Backend {
    fn drop(&mut self) {
        let _ = self.notifier.0.send(Msg::Shutdown);
    }
}

#[derive(Clone)]
pub struct EventProxy(mpsc::Sender<Event>);

impl EventListener for EventProxy {
    fn send_event(&self, event: Event) {
        let _ = self.0.blocking_send(event);
    }
}
