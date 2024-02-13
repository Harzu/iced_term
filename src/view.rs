use crate::backend::{BackendCommand, LinkAction, MouseButton, MouseMode};
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
use iced_core::text::Shaping;
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

    fn is_cursor_hovered_hyperlink(&self, state: &TermViewState) -> bool {
        if let Some(ref backend) = self.term.backend() {
            let content = backend.renderable_content();
            if let Some(hyperlink_range) = &content.hovered_hyperlink {
                return hyperlink_range.contains(&state.mouse_position_on_grid);
            }
        }

        false
    }

    fn handle_mouse_event(
        &self,
        state: &mut TermViewState,
        layout_position: Point,
        cursor_position: Point,
        event: iced::mouse::Event,
    ) -> Vec<Command> {
        let mut commands = vec![];
        if let Some(ref backend) = self.term.backend() {
            let terminal_content = backend.renderable_content();
            let terminal_mode = backend.renderable_content().terminal_mode;
            match event {
                iced_core::mouse::Event::ButtonPressed(
                    iced_core::mouse::Button::Left,
                ) => {
                    state.is_dragged = true;
                    if terminal_mode.contains(TermMode::SGR_MOUSE) && state.keyboard_modifiers.is_empty() {
                        commands.push(Command::ProcessBackendCommand(
                            BackendCommand::MouseReport(
                                MouseMode::Sgr,
                                MouseButton::LeftButton,
                                state.mouse_position_on_grid,
                                true,
                            ),
                        ));
                    } else {
                        let current_click =
                            Click::new(cursor_position, state.last_click);
                        let selction_type = match current_click.kind() {
                            mouse::click::Kind::Single => SelectionType::Simple,
                            mouse::click::Kind::Double => {
                                SelectionType::Semantic
                            },
                            mouse::click::Kind::Triple => SelectionType::Lines,
                        };
                        state.last_click = Some(current_click);
                        commands.push(Command::ProcessBackendCommand(
                            BackendCommand::SelectStart(
                                selction_type,
                                (
                                    cursor_position.x - layout_position.x,
                                    cursor_position.y - layout_position.y,
                                ),
                            ),
                        ));
                    }
                },
                iced_core::mouse::Event::CursorMoved { position } => {
                    let cursor_x = position.x - layout_position.x;
                    let cursor_y = position.y - layout_position.y;
                    state.mouse_position_on_grid = backend.selection_point(
                        cursor_x,
                        cursor_y,
                        terminal_content.grid.display_offset(),
                    );

                    if state.keyboard_modifiers == Modifiers::COMMAND {
                        commands.push(Command::ProcessBackendCommand(
                            BackendCommand::FindLink(
                                LinkAction::Hover,
                                state.mouse_position_on_grid,
                            ),
                        ));
                    }

                    if state.is_dragged {
                        if terminal_mode.contains(TermMode::SGR_MOUSE) && state.keyboard_modifiers.is_empty() {
                            commands.push(Command::ProcessBackendCommand(
                                BackendCommand::MouseReport(
                                    MouseMode::Sgr,
                                    MouseButton::LeftMove,
                                    state.mouse_position_on_grid,
                                    true,
                                ),
                            ));
                        } else {
                            commands.push(Command::ProcessBackendCommand(
                                BackendCommand::SelectUpdate((
                                    cursor_x, cursor_y,
                                )),
                            ));
                        }
                    }
                },
                iced_core::mouse::Event::ButtonReleased(
                    iced_core::mouse::Button::Left,
                ) => {
                    if self.term.bindings().get_action(
                        InputKind::Mouse(iced_core::mouse::Button::Left),
                        state.keyboard_modifiers,
                        terminal_content.terminal_mode,
                    ) == BindingAction::LinkOpen
                    {
                        commands.push(Command::ProcessBackendCommand(
                            BackendCommand::FindLink(
                                LinkAction::Open,
                                state.mouse_position_on_grid,
                            ),
                        ));
                    };

                    state.is_dragged = false;
                    if terminal_mode.contains(TermMode::SGR_MOUSE) {
                        commands.push(Command::ProcessBackendCommand(
                            BackendCommand::MouseReport(
                                MouseMode::Sgr,
                                MouseButton::LeftButton,
                                state.mouse_position_on_grid,
                                false,
                            ),
                        ));
                    }
                },
                iced::mouse::Event::WheelScrolled { delta } => match delta {
                    ScrollDelta::Lines { x: _, y } => {
                        state.scroll_pixels = 0.0;
                        let lines = if y <= 0.0 { y.floor() } else { y.ceil() };
                        commands.push(Command::ProcessBackendCommand(
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
                        commands.push(Command::ProcessBackendCommand(
                            BackendCommand::Scroll(-lines),
                        ));
                    },
                },
                _ => {},
            }
        }

        commands
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
                    if state.keyboard_modifiers == Modifiers::COMMAND {
                        return Some(Command::ProcessBackendCommand(
                            BackendCommand::FindLink(
                                LinkAction::Hover,
                                state.mouse_position_on_grid,
                            ),
                        ));
                    } else {
                        return Some(Command::ProcessBackendCommand(
                            BackendCommand::FindLink(
                                LinkAction::Clear,
                                state.mouse_position_on_grid,
                            ),
                        ));
                    }
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

                    let cell_size =
                        Size::new(cell_width as f32, cell_height as f32);
                    let background = Path::rectangle(
                        Point {
                            x: layout.position().x + x as f32,
                            y: layout.position().y + y as f32,
                        },
                        cell_size,
                    );
                    frame.fill(&background, bg);

                    if let Some(range) = &content.hovered_hyperlink {
                        if range.contains(&indexed.point)
                            && range.contains(&state.mouse_position_on_grid)
                        {
                            let underline = Path::line(
                                Point {
                                    x: layout.position().x + x as f32,
                                    y: layout.position().y
                                        + y as f32
                                        + cell_size.height,
                                },
                                Point {
                                    x: layout.position().x
                                        + x as f32
                                        + cell_size.width,
                                    y: layout.position().y
                                        + y as f32
                                        + cell_size.height,
                                },
                            );

                            frame.stroke(
                                &underline,
                                Stroke::default()
                                    .with_width(self.term.font().size() * 0.15)
                                    .with_color(fg),
                            )
                        }
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
                                    + cell_size.width / 2.0,
                                y: layout.position().y
                                    + y as f32
                                    + cell_size.height / 2.0,
                            },
                            font: self.term.font().font_type(),
                            size: self.term.font().size(),
                            color: fg,
                            horizontal_alignment: Horizontal::Center,
                            vertical_alignment: Vertical::Center,
                            shaping: Shaping::Advanced,
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

        let mut commands = vec![];
        match event {
            iced::Event::Mouse(mouse_event) => {
                if self.is_cursor_in_layout(cursor, layout) {
                    let cursor_position = cursor.position().unwrap();
                    commands = self.handle_mouse_event(
                        state,
                        layout.position(),
                        cursor_position,
                        mouse_event,
                    )
                }
            },
            iced::Event::Keyboard(keyboard_event) => {
                if let Some(cmd) =
                    self.handle_keyboard_event(state, clipboard, keyboard_event)
                {
                    commands.push(cmd);
                }
            },
            _ => {},
        };

        if !commands.is_empty() {
            for cmd in commands {
                shell.publish(Event::CommandReceived(self.term.id(), cmd));
            }
            iced::event::Status::Captured
        } else {
            iced::event::Status::Ignored
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: iced_core::Layout<'_>,
        cursor: iced_core::mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &iced::Renderer<Theme>,
    ) -> iced_core::mouse::Interaction {
        let state = tree.state.downcast_ref::<TermViewState>();
        let mut cursor_mode = iced_core::mouse::Interaction::Idle;
        let mut terminal_mode = TermMode::empty();
        if let Some(ref backend) = self.term.backend() {
            terminal_mode = backend.renderable_content().terminal_mode;
        }
        if self.is_cursor_in_layout(cursor, layout)
            && !terminal_mode.contains(TermMode::SGR_MOUSE)
        {
            cursor_mode = iced_core::mouse::Interaction::Text;
        }

        if self.is_cursor_hovered_hyperlink(state) {
            cursor_mode = iced_core::mouse::Interaction::Pointer;
        }

        cursor_mode
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
    mouse_position_on_grid: TerminalGridPoint,
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
