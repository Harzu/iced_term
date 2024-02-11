use std::ops::Index;

use crate::backend::BackendCommand;
use crate::bindings::{BindingAction, InputKind};
use crate::term::{Event, Term, ViewProxy};
use crate::Command;
use alacritty_terminal::index::Point as TerminalGridPoint;
use alacritty_terminal::selection::SelectionType;
use alacritty_terminal::term::{cell, TermMode};
use iced::alignment::{Horizontal, Vertical};
use iced::mouse::{Cursor, ScrollDelta};
use iced::widget::canvas::{Path, Text};
use iced::widget::container;
use iced::{Element, Length, Point, Rectangle, Size, Theme};
use iced_core::keyboard::Modifiers;
use iced_core::mouse::{self, Click};
use iced_core::widget::operation;
use iced_graphics::core::widget::{tree, Tree};
use iced_graphics::core::Widget;
use iced_graphics::geometry::{Renderer, Stroke};

pub struct TermView<'a> {
    term: &'a Term,
}

pub fn term_view(term: &Term) -> Element<'_, Event> {
    container(TermView::new(term))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(iced::theme::Container::Custom(Box::new(
            term.theme().clone(),
        )))
        .into()
}

impl<'a> TermView<'a> {
    fn new(term: &'a Term) -> Self {
        Self { term }
    }

    pub fn focus<Message: 'static>(
        id: iced::widget::text_input::Id,
    ) -> iced::Command<Message> {
        iced::widget::text_input::focus(id)
    }

    fn is_cursor_in_layout(
        &self,
        cursor: Cursor,
        layout: iced_graphics::core::Layout<'_>,
    ) -> bool {
        if let Some(cursor_position) = cursor.position() {
            let layout_position = layout.position();
            let layout_size = layout.bounds();
            let is_triggered = cursor_position.x >= layout_position.x
                && cursor_position.y >= layout_position.y
                && cursor_position.x < (layout_position.x + layout_size.width)
                && cursor_position.y < (layout_position.y + layout_size.height);

            return is_triggered;
        }

        false
    }

    fn handle_mouse_event(
        &self,
        state: &mut TermViewState,
        layout_position: Point,
        cursor_position: Point,
        event: iced::mouse::Event,
    ) -> Option<Command> {
        let mut cmd = None;
        if let Some(ref backend) = self.term.backend() {
            let terminal_content = backend.renderable_content();
            match event {
                iced_core::mouse::Event::ButtonPressed(
                    iced_core::mouse::Button::Left,
                ) => {
                    let current_click =
                        Click::new(cursor_position, state.last_click);
                    let selction_type = match current_click.kind() {
                        mouse::click::Kind::Single => SelectionType::Simple,
                        mouse::click::Kind::Double => SelectionType::Semantic,
                        mouse::click::Kind::Triple => SelectionType::Lines,
                    };
                    state.last_click = Some(current_click);
                    state.is_dragged = true;
                    cmd = Some(Command::ProcessBackendCommand(
                        BackendCommand::SelectStart(
                            selction_type,
                            (
                                cursor_position.x - layout_position.x,
                                cursor_position.y - layout_position.y,
                            ),
                        ),
                    ));
                },
                iced_core::mouse::Event::CursorMoved { position } => {
                    let cursor_x = position.x - layout_position.x;
                    let cursor_y = position.y - layout_position.y;
                    state.mouse_position_on_grid = backend.selection_point(
                        cursor_x,
                        cursor_y,
                        terminal_content.grid.display_offset()
                    );

                    if state.is_dragged {
                        cmd = Some(Command::ProcessBackendCommand(
                            BackendCommand::SelectUpdate((
                                cursor_x,
                                cursor_y,
                            )),
                        ));
                    }
                },
                iced_core::mouse::Event::ButtonReleased(
                    iced_core::mouse::Button::Left,
                ) => {
                    match self.term.bindings().get_action(
                        InputKind::Mouse(iced_core::mouse::Button::Left),
                        state.keyboard_modifiers,
                        terminal_content.terminal_mode,
                    ) {
                        BindingAction::LinkProcess => {
                            cmd = Some(Command::ProcessBackendCommand(
                                BackendCommand::MatchWord(state.mouse_position_on_grid),
                            ));
                        }
                        _ => {
                            state.is_dragged = false;
                        }
                    }
                },
                iced::mouse::Event::WheelScrolled { delta } => match delta {
                    ScrollDelta::Lines { x: _, y } => {
                        state.scroll_pixels = 0.0;
                        let lines = if y <= 0.0 { y.floor() } else { y.ceil() };
                        cmd = Some(Command::ProcessBackendCommand(
                            BackendCommand::Scroll(lines as i32),
                        ));
                    },
                    ScrollDelta::Pixels { x: _, y } => {
                        state.scroll_pixels -= y;
                        let mut lines = 0;
                        let line_height = self.term.font().measure().height;
                        while state.scroll_pixels <= -line_height {
                            lines -= 1;
                            state.scroll_pixels += line_height;
                        }
                        while state.scroll_pixels >= line_height {
                            lines += 1;
                            state.scroll_pixels -= line_height;
                        }
                        cmd = Some(Command::ProcessBackendCommand(
                            BackendCommand::Scroll(-lines),
                        ));
                    },
                },
                _ => {},
            }
        }

        cmd
    }

    fn handle_keyboard_event(
        &self,
        state: &mut TermViewState,
        clipboard: &mut dyn iced_graphics::core::Clipboard,
        event: iced::keyboard::Event,
    ) -> Option<Command> {
        if let Some(ref backend) = self.term.backend() {
            let mut binding_action = BindingAction::Ignore;
            let last_content = backend.renderable_content();
            match event {
                iced::keyboard::Event::ModifiersChanged(m) => {
                    state.keyboard_modifiers = m;
                },
                iced::keyboard::Event::CharacterReceived(c) => {
                    binding_action = self.term.bindings().get_action(
                        InputKind::Char(c.to_ascii_lowercase()),
                        state.keyboard_modifiers,
                        last_content.terminal_mode,
                    );

                    if binding_action == BindingAction::Ignore
                        && !c.is_control()
                    {
                        let mut buf = [0, 0, 0, 0];
                        let str = c.encode_utf8(&mut buf);
                        return Some(Command::ProcessBackendCommand(
                            BackendCommand::Write(str.as_bytes().to_vec()),
                        ));
                    }
                },
                iced::keyboard::Event::KeyPressed {
                    key_code,
                    modifiers,
                } => {
                    binding_action = self.term.bindings().get_action(
                        InputKind::KeyCode(key_code),
                        modifiers,
                        last_content.terminal_mode,
                    );
                },
                _ => {},
            }

            match binding_action {
                BindingAction::Char(c) => {
                    let mut buf = [0, 0, 0, 0];
                    let str = c.encode_utf8(&mut buf);
                    return Some(Command::ProcessBackendCommand(
                        BackendCommand::Write(str.as_bytes().to_vec()),
                    ));
                },
                BindingAction::Esc(seq) => {
                    return Some(Command::ProcessBackendCommand(
                        BackendCommand::Write(seq.as_bytes().to_vec()),
                    ));
                },
                BindingAction::Paste => {
                    if let Some(data) = clipboard.read() {
                        let input: Vec<u8> = data.bytes().collect();
                        return Some(Command::ProcessBackendCommand(
                            BackendCommand::Write(input),
                        ));
                    }
                },
                BindingAction::Copy => {
                    clipboard.write(backend.selectable_content());
                },
                _ => {},
            };
        }

        None
    }
}

impl<'a> Widget<Event, iced::Renderer<Theme>> for TermView<'a> {
    fn width(&self) -> Length {
        Length::Fill
    }

    fn height(&self) -> Length {
        Length::Fill
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<TermViewState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(TermViewState::new())
    }

    fn layout(
        &self,
        _renderer: &iced::Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        let size = limits
            .width(Length::Fill)
            .height(Length::Fill)
            .resolve(Size::ZERO);

        iced::advanced::layout::Node::new(size)
    }

    fn operate(
        &self,
        tree: &mut Tree,
        _layout: iced_core::Layout<'_>,
        _renderer: &iced::Renderer<Theme>,
        operation: &mut dyn operation::Operation<Event>,
    ) {
        let state = tree.state.downcast_mut::<TermViewState>();
        let wid = iced_core::widget::Id::from(self.term.widget_id());
        operation.focusable(state, Some(&wid));
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer<Theme>,
        _theme: &Theme,
        _style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout,
        _cursor: Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<TermViewState>();
        let geom = self.term.cache().draw(renderer, viewport.size(), |frame| {
            if let Some(ref backend) = self.term.backend() {
                let content = backend.renderable_content();
                for indexed in content.grid.display_iter() {
                    let cell_width = content.terminal_size.cell_width as f64;
                    let cell_height = content.terminal_size.cell_height as f64;
                    let x = indexed.point.column.0 as f64 * cell_width;
                    let y = (indexed.point.line.0 as f64
                        + content.grid.display_offset() as f64)
                        * cell_height;

                    let mut fg = self.term.theme().get_color(indexed.fg);
                    let mut bg = self.term.theme().get_color(indexed.bg);

                    if indexed.cell.flags.intersects(cell::Flags::DIM)
                        || indexed.cell.flags.intersects(cell::Flags::DIM_BOLD)
                    {
                        fg.a *= 0.7;
                    }

                    if indexed.cell.flags.contains(cell::Flags::INVERSE) {
                        std::mem::swap(&mut fg, &mut bg);
                    }

                    if let Some(range) = content.selectable_range {
                        if range.contains(indexed.point) {
                            std::mem::swap(&mut fg, &mut bg);
                        }
                    }

                    let size = Size::new(cell_width as f32, cell_height as f32);
                    let background = Path::rectangle(
                        Point {
                            x: layout.position().x + x as f32,
                            y: layout.position().y + y as f32,
                        },
                        size,
                    );
                    frame.fill(&background, bg);

                    if state.mouse_position_on_grid == indexed.point {
                        // println!("Hyperlink");
                        frame.stroke(
                            &background, 
                            Stroke::default()
                                .with_color(fg)
                                .with_width(2.0)
                        )
                    }

                    if content.grid.cursor.point == indexed.point {
                        let cursor_rect = Path::rectangle(
                            Point {
                                x: layout.position().x
                                    + content.grid.cursor.point.column.0 as f32
                                        * cell_width as f32,
                                y: layout.position().y
                                    + (content.grid.cursor.point.line.0
                                        + content.grid.display_offset() as i32)
                                        as f32
                                        * cell_height as f32,
                            },
                            Size::new(cell_width as f32, cell_height as f32),
                        );

                        let cursor_color =
                            self.term.theme().get_color(content.cursor.fg);
                        frame.fill(&cursor_rect, cursor_color);
                    }

                    if indexed.c != ' ' && indexed.c != '\t' {
                        if content.grid.cursor.point == indexed.point
                            && content
                                .terminal_mode
                                .contains(TermMode::APP_CURSOR)
                        {
                            fg = bg;
                        }

                        let text = Text {
                            content: indexed.c.to_string(),
                            position: Point {
                                x: layout.position().x
                                    + x as f32
                                    + size.width / 2.0,
                                y: layout.position().y
                                    + y as f32
                                    + size.height / 2.0,
                            },
                            font: self.term.font().font_type(),
                            size: self.term.font().size(),
                            color: fg,
                            horizontal_alignment: Horizontal::Center,
                            vertical_alignment: Vertical::Center,
                            ..Text::default()
                        };

                        frame.fill_text(text);
                    }
                }
            }
        });

        renderer.draw(vec![geom]);
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: iced::Event,
        layout: iced_graphics::core::Layout<'_>,
        cursor: Cursor,
        _renderer: &iced::Renderer<Theme>,
        clipboard: &mut dyn iced_graphics::core::Clipboard,
        shell: &mut iced_graphics::core::Shell<'_, Event>,
        _viewport: &Rectangle,
    ) -> iced::event::Status {
        let state = tree.state.downcast_mut::<TermViewState>();
        let layout_size = layout.bounds().size();
        if state.size != layout_size && self.term.backend().is_some() {
            state.size = layout_size;
            let cmd = Command::ProcessBackendCommand(BackendCommand::Resize(
                layout_size,
            ));
            shell.publish(Event::CommandReceived(self.term.id(), cmd));
        }

        if !state.is_focused {
            return iced::event::Status::Ignored;
        }

        let mut cmd = None;
        match event {
            iced::Event::Mouse(mouse_event) => {
                if self.is_cursor_in_layout(cursor, layout) {
                    let cursor_position = cursor.position().unwrap();
                    cmd = self.handle_mouse_event(
                        state,
                        layout.position(),
                        cursor_position,
                        mouse_event,
                    )
                }
            },
            iced::Event::Keyboard(keyboard_event) => {
                cmd =
                    self.handle_keyboard_event(state, clipboard, keyboard_event)
            },
            _ => {},
        };

        match cmd {
            None => iced::event::Status::Ignored,
            Some(c) => {
                shell.publish(Event::CommandReceived(self.term.id(), c));
                iced::event::Status::Captured
            },
        }
    }

    fn mouse_interaction(
        &self,
        _state: &Tree,
        layout: iced_core::Layout<'_>,
        cursor: iced_core::mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &iced::Renderer<Theme>,
    ) -> iced_core::mouse::Interaction {
        if self.is_cursor_in_layout(cursor, layout) {
            return iced_core::mouse::Interaction::Text;
        }

        iced_core::mouse::Interaction::Idle
    }
}

impl<'a> From<TermView<'a>> for Element<'a, Event, iced::Renderer<Theme>> {
    fn from(widget: TermView<'a>) -> Self {
        Self::new(widget)
    }
}

#[derive(Debug, Clone)]
pub struct TermViewState {
    is_focused: bool,
    is_dragged: bool,
    last_click: Option<mouse::Click>,
    scroll_pixels: f32,
    keyboard_modifiers: Modifiers,
    size: Size<f32>,
    mouse_position_on_grid: TerminalGridPoint
}

impl TermViewState {
    pub fn new() -> Self {
        Self {
            is_focused: true,
            is_dragged: false,
            last_click: None,
            scroll_pixels: 0.0,
            keyboard_modifiers: Modifiers::empty(),
            size: Size::from([0.0, 0.0]),
            mouse_position_on_grid: TerminalGridPoint::default(),
        }
    }
}

impl Default for TermViewState {
    fn default() -> Self {
        Self::new()
    }
}

impl operation::Focusable for TermViewState {
    fn is_focused(&self) -> bool {
        self.is_focused
    }

    fn focus(&mut self) {
        self.is_focused = true;
    }

    fn unfocus(&mut self) {
        self.is_focused = false;
    }
}
