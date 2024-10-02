use crate::backend::{
    Backend, BackendCommand, LinkAction, MouseButton, MouseMode,
    RenderableContent,
};
use crate::bindings::{BindingAction, BindingsLayout, InputKind};
use crate::terminal::{Event, Command, Terminal, ViewProxy};
use alacritty_terminal::index::Point as TerminalGridPoint;
use alacritty_terminal::selection::SelectionType;
use alacritty_terminal::term::{cell, TermMode};
use iced::alignment::{Horizontal, Vertical};
use iced::mouse::{Cursor, ScrollDelta};
use iced::widget::canvas::{Path, Text};
use iced::widget::container;
use iced::{Element, Length, Point, Rectangle, Size, Theme};
use iced_core::clipboard::Kind as ClipboardKind;
use iced_core::keyboard::{Key, Modifiers};
use iced_core::mouse::{self, Click};
use iced_core::text::{LineHeight, Shaping};
use iced_core::widget::operation;
use iced_graphics::core::widget::{tree, Tree};
use iced_graphics::core::Widget;
use iced_graphics::geometry::Stroke;

pub struct TermView<'a> {
    term: &'a Terminal,
}

pub fn term_view(term: &Terminal) -> Element<'_, Event> {
    container(TermView::new(term))
        .width(Length::Fill)
        .height(Length::Fill)
        // .style(iced::theme)
        .into()
}

impl<'a> TermView<'a> {
    fn new(term: &'a Terminal) -> Self {
        Self { term }
    }

    pub fn focus<Message: 'static>(
        id: iced::widget::text_input::Id,
    ) -> iced::Task<Message> {
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
        let mut commands = Vec::new();
        if let Some(backend) = self.term.backend() {
            let terminal_content = backend.renderable_content();
            let terminal_mode = terminal_content.terminal_mode;

            match event {
                iced_core::mouse::Event::ButtonPressed(
                    iced_core::mouse::Button::Left,
                ) => {
                    Self::handle_left_button_pressed(
                        state,
                        &terminal_mode,
                        cursor_position,
                        layout_position,
                        &mut commands,
                    );
                },
                iced_core::mouse::Event::CursorMoved { position } => {
                    Self::handle_cursor_moved(
                        state,
                        backend.renderable_content(),
                        position,
                        layout_position,
                        &mut commands,
                    );
                },
                iced_core::mouse::Event::ButtonReleased(
                    iced_core::mouse::Button::Left,
                ) => {
                    Self::handle_button_released(
                        state,
                        &terminal_mode,
                        self.term.bindings(),
                        &mut commands,
                    );
                },
                iced::mouse::Event::WheelScrolled { delta } => {
                    Self::handle_wheel_scrolled(
                        state,
                        delta,
                        &self.term.font().measure(),
                        &mut commands,
                    );
                },
                _ => {},
            }
        }

        commands
    }

    fn handle_left_button_pressed(
        state: &mut TermViewState,
        terminal_mode: &TermMode,
        cursor_position: Point,
        layout_position: Point,
        commands: &mut Vec<Command>,
    ) {
        let cmd = if terminal_mode.contains(TermMode::SGR_MOUSE)
            && state.keyboard_modifiers.is_empty()
        {
            Command::ProcessBackendCommand(BackendCommand::MouseReport(
                MouseMode::Sgr,
                MouseButton::LeftButton,
                state.mouse_position_on_grid,
                true,
            ))
        } else {
            let current_click = Click::new(cursor_position, mouse::Button::Left, state.last_click);
            let selection_type = match current_click.kind() {
                mouse::click::Kind::Single => SelectionType::Simple,
                mouse::click::Kind::Double => SelectionType::Semantic,
                mouse::click::Kind::Triple => SelectionType::Lines,
            };
            state.last_click = Some(current_click);
            Command::ProcessBackendCommand(BackendCommand::SelectStart(
                selection_type,
                (
                    cursor_position.x - layout_position.x,
                    cursor_position.y - layout_position.y,
                ),
            ))
        };
        commands.push(cmd);
        state.is_dragged = true;
    }

    fn handle_cursor_moved(
        state: &mut TermViewState,
        terminal_content: &RenderableContent,
        position: Point,
        layout_position: Point,
        commands: &mut Vec<Command>,
    ) {
        let cursor_x = position.x - layout_position.x;
        let cursor_y = position.y - layout_position.y;
        state.mouse_position_on_grid = Backend::selection_point(
            cursor_x,
            cursor_y,
            &terminal_content.terminal_size,
            terminal_content.grid.display_offset(),
        );

        // Handle command or selection update based on terminal mode and modifiers
        if state.is_dragged {
            let terminal_mode = terminal_content.terminal_mode;
            let cmd = if terminal_mode.contains(TermMode::SGR_MOUSE)
                && state.keyboard_modifiers.is_empty()
            {
                Command::ProcessBackendCommand(BackendCommand::MouseReport(
                    MouseMode::Sgr,
                    MouseButton::LeftMove,
                    state.mouse_position_on_grid,
                    true,
                ))
            } else {
                Command::ProcessBackendCommand(BackendCommand::SelectUpdate((
                    cursor_x, cursor_y,
                )))
            };
            commands.push(cmd);
        }

        // Handle link hover if applicable
        if state.keyboard_modifiers == Modifiers::COMMAND {
            commands.push(Command::ProcessBackendCommand(
                BackendCommand::ProcessLink(
                    LinkAction::Hover,
                    state.mouse_position_on_grid,
                ),
            ));
        }
    }

    fn handle_button_released(
        state: &mut TermViewState,
        terminal_mode: &TermMode,
        bindings: &BindingsLayout, // Use the actual type of your bindings here
        commands: &mut Vec<Command>,
    ) {
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

        if bindings.get_action(
            InputKind::Mouse(iced_core::mouse::Button::Left),
            state.keyboard_modifiers,
            *terminal_mode,
        ) == BindingAction::LinkOpen
        {
            commands.push(Command::ProcessBackendCommand(
                BackendCommand::ProcessLink(
                    LinkAction::Open,
                    state.mouse_position_on_grid,
                ),
            ));
        }
    }

    fn handle_wheel_scrolled(
        state: &mut TermViewState,
        delta: ScrollDelta,
        font_measure: &Size<f32>,
        commands: &mut Vec<Command>,
    ) {
        match delta {
            ScrollDelta::Lines { y, .. } => {
                let lines = y.signum() * y.abs().round();
                commands.push(Command::ProcessBackendCommand(
                    BackendCommand::Scroll(lines as i32),
                ));
            },
            ScrollDelta::Pixels { y, .. } => {
                state.scroll_pixels -= y;
                let line_height = font_measure.height; // Assume this method exists and gives the height of a line
                let lines = (state.scroll_pixels / line_height).trunc();
                state.scroll_pixels %= line_height;
                if lines != 0.0 {
                    commands.push(Command::ProcessBackendCommand(
                        BackendCommand::Scroll(lines as i32),
                    ));
                }
            },
        }
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
                    let action =
                        if state.keyboard_modifiers == Modifiers::COMMAND {
                            LinkAction::Hover
                        } else {
                            LinkAction::Clear
                        };
                    return Some(Command::ProcessBackendCommand(
                        BackendCommand::ProcessLink(
                            action,
                            state.mouse_position_on_grid,
                        ),
                    ));
                },
                iced::keyboard::Event::KeyPressed {
                    key,
                    modifiers,
                    ..
                } => match key {
                    Key::Character(c) => {
                        binding_action = self.term.bindings().get_action(
                            InputKind::Char(c.to_ascii_lowercase()),
                            state.keyboard_modifiers,
                            last_content.terminal_mode,
                        );

                        if binding_action == BindingAction::Ignore {
                            return Some(Command::ProcessBackendCommand(
                                BackendCommand::Write(c.as_bytes().to_vec()),
                            ));
                        }
                    },
                    Key::Named(code) => {
                        binding_action = self.term.bindings().get_action(
                            InputKind::KeyCode(code),
                            modifiers,
                            last_content.terminal_mode,
                        );
                    },
                    _ => {},
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
                    if let Some(data) = clipboard.read(ClipboardKind::Standard)
                    {
                        let input: Vec<u8> = data.bytes().collect();
                        return Some(Command::ProcessBackendCommand(
                            BackendCommand::Write(input),
                        ));
                    }
                },
                BindingAction::Copy => {
                    clipboard.write(
                        ClipboardKind::Standard,
                        backend.selectable_content(),
                    );
                },
                _ => {},
            };
        }

        None
    }
}

impl<'a> Widget<Event, Theme, iced::Renderer> for TermView<'a> {
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fill,
            height: Length::Fill,
        }
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<TermViewState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(TermViewState::new())
    }

    fn layout(
        &self,
        _tree: &mut Tree,
        _renderer: &iced::Renderer,
        limits: &iced_core::layout::Limits,
    ) -> iced_core::layout::Node {
        let size = limits.resolve(Length::Fill, Length::Fill, Size::ZERO);
        iced::advanced::layout::Node::new(size)
    }

    fn operate(
        &self,
        tree: &mut Tree,
        _layout: iced_core::Layout<'_>,
        _renderer: &iced::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        let state = tree.state.downcast_mut::<TermViewState>();
        let wid = iced_core::widget::Id::from(self.term.widget_id());
        operation.focusable(state, Some(&wid));   
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        _theme: &Theme,
        _style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout,
        _cursor: Cursor,
        viewport: &Rectangle,
    ) {
        if let Some(ref backend) = self.term.backend() {
            let state = tree.state.downcast_ref::<TermViewState>();
            let content = backend.renderable_content();
            let term_size = content.terminal_size;
            let cell_width = term_size.cell_width as f32;
            let cell_height = term_size.cell_height as f32;
            let font_size = self.term.font().size();
            let font_scale_factor = self.term.font().scale_factor();
            let layout_offset_x = layout.position().x;
            let layout_offset_y = layout.position().y;

            let geom =
                self.term.cache().draw(renderer, viewport.size(), |frame| {
                    for indexed in content.grid.display_iter() {
                        let x = layout_offset_x
                            + (indexed.point.column.0 as f32 * cell_width);
                        let y = layout_offset_y
                            + ((indexed.point.line.0 as f32
                                + content.grid.display_offset() as f32)
                                * cell_height);

                        let mut fg = self.term.theme().get_color(indexed.fg);
                        let mut bg = self.term.theme().get_color(indexed.bg);

                        // Handle dim, inverse, and selected text
                        if indexed.cell.flags.intersects(
                            cell::Flags::DIM | cell::Flags::DIM_BOLD,
                        ) {
                            fg.a *= 0.7;
                        }
                        if indexed.cell.flags.contains(cell::Flags::INVERSE)
                            || content
                                .selectable_range
                                .map_or(false, |r| r.contains(indexed.point))
                        {
                            std::mem::swap(&mut fg, &mut bg);
                        }

                        let cell_size = Size::new(cell_width, cell_height);

                        // Draw cell background
                        let background =
                            Path::rectangle(Point::new(x, y), cell_size);
                        frame.fill(&background, bg);

                        // Draw hovered hyperlink underline
                        if content.hovered_hyperlink.as_ref().map_or(
                            false,
                            |range| {
                                range.contains(&indexed.point)
                                    && range
                                        .contains(&state.mouse_position_on_grid)
                            },
                        ) {
                            let underline_height = y + cell_size.height;
                            let underline = Path::line(
                                Point::new(x, underline_height),
                                Point::new(
                                    x + cell_size.width,
                                    underline_height,
                                ),
                            );
                            frame.stroke(
                                &underline,
                                Stroke::default()
                                    .with_width(font_size * 0.15)
                                    .with_color(fg),
                            );
                        }

                        // Handle cursor rendering
                        if content.grid.cursor.point == indexed.point {
                            let cursor_color =
                                self.term.theme().get_color(content.cursor.fg);
                            let cursor_rect =
                                Path::rectangle(Point::new(x, y), cell_size);
                            frame.fill(&cursor_rect, cursor_color);
                        }

                        // Draw text
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
                                position: Point::new(
                                    x + (cell_size.width / 2.0),
                                    y + (cell_size.height / 2.0),
                                ),
                                font: self.term.font().font_type(),
                                size: iced_core::Pixels(font_size),
                                color: fg,
                                horizontal_alignment: Horizontal::Center,
                                vertical_alignment: Vertical::Center,
                                shaping: Shaping::Advanced,
                                line_height: LineHeight::Relative(
                                    font_scale_factor,
                                ),
                            };
                            frame.fill_text(text);
                        }
                    }
                });

            use iced::advanced::graphics::geometry::Renderer as _;
            renderer.draw_geometry(geom);
        }
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: iced::Event,
        layout: iced_graphics::core::Layout<'_>,
        cursor: Cursor,
        _renderer: &iced::Renderer,
        clipboard: &mut dyn iced_graphics::core::Clipboard,
        shell: &mut iced_graphics::core::Shell<'_, Event>,
        _viewport: &Rectangle,
    ) -> iced::event::Status {
        let state = tree.state.downcast_mut::<TermViewState>();
        let layout_size = layout.bounds().size();
        if state.size != layout_size && self.term.backend().is_some() {
            state.size = layout_size;
            let cmd = Command::ProcessBackendCommand(BackendCommand::Resize(
                Some(layout_size),
                None,
            ));
            shell.publish(Event::CommandReceived(self.term.id(), cmd));
        }

        if !state.is_focused {
            return iced::event::Status::Ignored;
        }

        let commands = match event {
            iced::Event::Mouse(mouse_event)
                if self.is_cursor_in_layout(cursor, layout) =>
            {
                self.handle_mouse_event(
                    state,
                    layout.position(),
                    cursor.position().unwrap(), // Assuming cursor position is always available here.
                    mouse_event,
                )
            },
            iced::Event::Keyboard(keyboard_event) => {
                self.handle_keyboard_event(state, clipboard, keyboard_event)
                    .into_iter() // Convert Option to iterator (0 or 1 element)
                    .collect()
            },
            _ => Vec::new(), // No commands for other events.
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
        _renderer: &iced::Renderer,
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

impl<'a> From<TermView<'a>> for Element<'a, Event, Theme, iced::Renderer> {
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

#[cfg(test)]
mod tests {
    use super::*;

    mod handle_left_button_pressed_tests {
        use super::*;
        use alacritty_terminal::index::{Column, Line};

        #[test]
        fn handles_sgr_mouse_mode_with_left_click() {
            let mut state = TermViewState::new();
            let terminal_mode = TermMode::SGR_MOUSE;
            let layout_position = Point { x: 5.0, y: 5.0 };
            let cursor_position = Point { x: 100.0, y: 150.0 };
            let mut commands = Vec::new();

            TermView::handle_left_button_pressed(
                &mut state,
                &terminal_mode,
                cursor_position,
                layout_position,
                &mut commands,
            );

            assert_eq!(commands.len(), 1);
            assert!(matches!(
                commands[0],
                Command::ProcessBackendCommand(BackendCommand::MouseReport(
                    MouseMode::Sgr,
                    MouseButton::LeftButton,
                    TerminalGridPoint {
                        line: Line(0),
                        column: Column(0),
                    },
                    true,
                ))
            ));
            assert!(state.is_dragged);
        }

        #[test]
        fn starts_simple_selection_with_left_click() {
            let terminal_mode = TermMode::SGR_MOUSE;
            let cursor_position = Point { x: 200.0, y: 150.0 };
            let layout_position = Point { x: 50.0, y: 50.0 };

            let cases = vec![
                SelectionType::Simple,
                SelectionType::Semantic,
                SelectionType::Lines,
            ];

            for _selection_type in cases {
                let mut state = TermViewState::new();
                state.keyboard_modifiers = Modifiers::SHIFT;
                let mut commands = Vec::new();

                TermView::handle_left_button_pressed(
                    &mut state,
                    &terminal_mode,
                    cursor_position,
                    layout_position,
                    &mut commands,
                );

                assert_eq!(commands.len(), 1);
                assert!(matches!(
                    commands[0],
                    Command::ProcessBackendCommand(
                        BackendCommand::SelectStart(
                            _selection_type,
                            (150.0, 100.0)
                        )
                    ),
                ));
                assert!(state.is_dragged);
            }
        }
    }

    mod handle_cursor_moved_tests {
        use alacritty_terminal::index::{Column, Line};

        use super::*;

        #[test]
        fn updates_mouse_position_on_grid() {
            let mut state = TermViewState::new();
            let terminal_content = RenderableContent::default();
            let mut commands = Vec::new();
            let cases = vec![
                (
                    Point { x: 0.0, y: 0.0 },
                    Point { x: 1.0, y: 1.0 },
                    TerminalGridPoint {
                        line: Line(1),
                        column: Column(1),
                    },
                ),
                (
                    Point { x: 0.0, y: 0.0 },
                    Point { x: 2.0, y: 2.0 },
                    TerminalGridPoint {
                        line: Line(2),
                        column: Column(2),
                    },
                ),
                (
                    Point { x: 0.0, y: 0.0 },
                    Point { x: 30.0, y: 2.0 },
                    TerminalGridPoint {
                        line: Line(2),
                        column: Column(30),
                    },
                ),
                (
                    Point { x: 10.0, y: 0.0 },
                    Point { x: 30.0, y: 2.0 },
                    TerminalGridPoint {
                        line: Line(2),
                        column: Column(20),
                    },
                ),
                (
                    Point { x: 10.0, y: 10.0 },
                    Point { x: 30.0, y: 2.0 },
                    TerminalGridPoint {
                        line: Line(0),
                        column: Column(20),
                    },
                ),
            ];

            for (layout_position, cursor_position, expected) in cases {
                TermView::handle_cursor_moved(
                    &mut state,
                    &terminal_content,
                    cursor_position,
                    layout_position,
                    &mut commands,
                );

                assert_eq!(state.mouse_position_on_grid, expected);
            }
        }

        #[test]
        fn generates_drag_update_command_when_dragged() {
            let mut state = TermViewState::new();
            state.is_dragged = true; // Simulate an ongoing drag operation
            let terminal_content = RenderableContent::default();
            let layout_position = Point { x: 5.0, y: 5.0 };
            let cursor_position = Point { x: 100.0, y: 150.0 };
            let mut commands = Vec::new();

            TermView::handle_cursor_moved(
                &mut state,
                &terminal_content,
                cursor_position,
                layout_position,
                &mut commands,
            );

            assert_eq!(commands.len(), 1);
            assert!(matches!(
                commands[0],
                Command::ProcessBackendCommand(BackendCommand::SelectUpdate((
                    95.0, 145.0
                )))
            ));
        }

        #[test]
        fn generates_drag_update_command_when_dragged_in_srg_mode() {
            let mut state = TermViewState::new();
            state.is_dragged = true; // Simulate an ongoing drag operation
            let mut terminal_content = RenderableContent::default();
            terminal_content.terminal_mode = TermMode::SGR_MOUSE;
            let layout_position = Point { x: 5.0, y: 5.0 };
            let cursor_position = Point { x: 100.0, y: 150.0 };
            let mut commands = Vec::new();

            TermView::handle_cursor_moved(
                &mut state,
                &terminal_content,
                cursor_position,
                layout_position,
                &mut commands,
            );

            assert_eq!(commands.len(), 1);
            assert!(matches!(
                commands[0],
                Command::ProcessBackendCommand(BackendCommand::MouseReport(
                    MouseMode::Sgr,
                    MouseButton::LeftMove,
                    TerminalGridPoint {
                        line: Line(49),
                        column: Column(79),
                    },
                    true,
                ))
            ));
        }

        #[test]
        fn generates_drag_update_command_when_dragged_in_srg_mode_with_key_mods(
        ) {
            let mut state = TermViewState::new();
            state.keyboard_modifiers = Modifiers::SHIFT;
            state.is_dragged = true; // Simulate an ongoing drag operation
            let mut terminal_content = RenderableContent::default();
            terminal_content.terminal_mode = TermMode::SGR_MOUSE;
            let layout_position = Point { x: 5.0, y: 5.0 };
            let cursor_position = Point { x: 100.0, y: 150.0 };
            let mut commands = Vec::new();

            TermView::handle_cursor_moved(
                &mut state,
                &terminal_content,
                cursor_position,
                layout_position,
                &mut commands,
            );

            assert_eq!(commands.len(), 1);
            assert!(matches!(
                commands[0],
                Command::ProcessBackendCommand(BackendCommand::SelectUpdate((
                    95.0, 145.0
                )))
            ));
        }

        #[test]
        fn generates_drag_update_and_link_open() {
            let mut state = TermViewState::new();
            state.keyboard_modifiers = Modifiers::COMMAND;
            state.is_dragged = true; // Simulate an ongoing drag operation
            let mut terminal_content = RenderableContent::default();
            terminal_content.terminal_mode = TermMode::SGR_MOUSE;
            let layout_position = Point { x: 5.0, y: 5.0 };
            let cursor_position = Point { x: 100.0, y: 150.0 };
            let mut commands = Vec::new();

            TermView::handle_cursor_moved(
                &mut state,
                &terminal_content,
                cursor_position,
                layout_position,
                &mut commands,
            );

            assert_eq!(commands.len(), 2);
            assert!(matches!(
                commands[0],
                Command::ProcessBackendCommand(BackendCommand::SelectUpdate((
                    95.0, 145.0
                )))
            ));
            assert!(matches!(
                commands[1],
                Command::ProcessBackendCommand(BackendCommand::ProcessLink(
                    LinkAction::Hover,
                    TerminalGridPoint {
                        line: Line(49),
                        column: Column(79),
                    },
                ))
            ));
        }
    }

    mod handle_button_released_tests {
        use super::*;
        use alacritty_terminal::index::{Column, Line};

        #[test]
        fn sgr_mouse_mode_activated() {
            let mut state = TermViewState::new();
            let terminal_mode = TermMode::SGR_MOUSE;
            let bindings = BindingsLayout::new();
            let mut commands = Vec::new();

            TermView::handle_button_released(
                &mut state,
                &terminal_mode,
                &bindings,
                &mut commands,
            );

            assert_eq!(commands.len(), 1);
            assert!(matches!(
                commands[0],
                Command::ProcessBackendCommand(BackendCommand::MouseReport(
                    MouseMode::Sgr,
                    MouseButton::LeftButton,
                    TerminalGridPoint {
                        line: Line(0),
                        column: Column(0)
                    },
                    false
                )),
            ));
        }

        #[test]
        fn link_open_on_button_release() {
            let mut state = TermViewState::new();
            state.keyboard_modifiers = Modifiers::COMMAND;
            let terminal_mode = TermMode::SGR_MOUSE; // Assume SGR_MOUSE mode doesn't affect link opening
            let bindings = BindingsLayout::new();
            let mut commands = Vec::new();

            TermView::handle_button_released(
                &mut state,
                &terminal_mode,
                &bindings,
                &mut commands,
            );

            assert_eq!(commands.len(), 2);
            assert!(matches!(
                commands[0],
                Command::ProcessBackendCommand(BackendCommand::MouseReport(
                    MouseMode::Sgr,
                    MouseButton::LeftButton,
                    TerminalGridPoint {
                        line: Line(0),
                        column: Column(0)
                    },
                    false
                )),
            ));
            assert!(matches!(
                commands[1],
                Command::ProcessBackendCommand(BackendCommand::ProcessLink(
                    LinkAction::Open,
                    TerminalGridPoint {
                        line: Line(0),
                        column: Column(0)
                    }
                )),
            ));
        }

        #[test]
        fn link_open_on_button_release_in_non_srg_mode() {
            let mut state = TermViewState::new();
            state.keyboard_modifiers = Modifiers::COMMAND;
            state.mouse_position_on_grid = TerminalGridPoint {
                line: Line(4),
                column: Column(10),
            };
            let terminal_mode = TermMode::empty(); // Assume SGR_MOUSE mode doesn't affect link opening
            let bindings = BindingsLayout::new();
            let mut commands = Vec::new();

            TermView::handle_button_released(
                &mut state,
                &terminal_mode,
                &bindings,
                &mut commands,
            );

            assert_eq!(commands.len(), 1);
            assert!(matches!(
                commands[0],
                Command::ProcessBackendCommand(BackendCommand::ProcessLink(
                    LinkAction::Open,
                    TerminalGridPoint {
                        line: Line(4),
                        column: Column(10)
                    }
                )),
            ));
        }
    }

    mod handle_wheel_scrolled_tests {
        use super::*;
        use crate::font::TermFont;
        use crate::settings::FontSettings;

        #[test]
        fn scroll_with_lines_downward() {
            let mut state = TermViewState::new();
            let font = TermFont::new(FontSettings::default());
            let mut commands = Vec::new();

            TermView::handle_wheel_scrolled(
                &mut state,
                ScrollDelta::Lines { y: 3.0, x: 0.0 }, // Scroll down 3 lines
                &font.measure(),
                &mut commands,
            );

            assert_eq!(commands.len(), 1);
            assert!(matches!(
                commands[0],
                Command::ProcessBackendCommand(BackendCommand::Scroll(3))
            ));
        }

        #[test]
        fn scroll_with_lines_upward() {
            let mut state = TermViewState::new();
            let font = TermFont::new(FontSettings::default());
            let mut commands = Vec::new();

            TermView::handle_wheel_scrolled(
                &mut state,
                ScrollDelta::Lines { y: -2.0, x: 0.0 },
                &font.measure(),
                &mut commands,
            );

            assert_eq!(commands.len(), 1);
            assert!(matches!(
                commands[0],
                Command::ProcessBackendCommand(BackendCommand::Scroll(-2))
            ));
        }

        #[test]
        fn scroll_with_pixels_accumulating_downward() {
            let mut state = TermViewState::new();
            let font = TermFont::new(FontSettings::default());
            let mut commands = Vec::new();

            TermView::handle_wheel_scrolled(
                &mut state,
                ScrollDelta::Pixels { y: 45.0, x: 0.0 },
                &font.measure(),
                &mut commands,
            );

            assert_eq!(commands.len(), 1);
            assert!(matches!(
                commands[0],
                Command::ProcessBackendCommand(BackendCommand::Scroll(-2))
            ));
            assert_eq!(state.scroll_pixels, -8.600002);
        }

        #[test]
        fn scroll_with_pixels_accumulating_upward() {
            let mut state = TermViewState::new();
            let font = TermFont::new(FontSettings::default());
            let mut commands = Vec::new();

            TermView::handle_wheel_scrolled(
                &mut state,
                ScrollDelta::Pixels { y: -60.0, x: 0.0 },
                &font.measure(),
                &mut commands,
            );

            assert_eq!(commands.len(), 1);
            assert!(matches!(
                commands[0],
                Command::ProcessBackendCommand(BackendCommand::Scroll(3))
            ));
            assert_eq!(state.scroll_pixels, 5.4000034);
        }
    }
}
