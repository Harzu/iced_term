use crate::backend::{BackendSettings, Pty};
use crate::bindings::{Binding, BindingAction, BindingsLayout, InputKind};
use crate::font::TermFont;
use crate::theme::TermTheme;
use crate::{ColorPalette, FontSettings};
use alacritty_terminal::index::{Column, Line, Point as TermPoint};
use alacritty_terminal::selection::{Selection, SelectionRange, SelectionType};
use alacritty_terminal::term::{cell, TermMode};
use iced::alignment::{Horizontal, Vertical};
use iced::futures::SinkExt;
use iced::mouse::{Cursor, ScrollDelta};
use iced::widget::canvas::{Cache, Path, Text};
use iced::widget::container;
use iced::{Element, Length, Point, Rectangle, Size, Subscription, Theme};
use iced_core::keyboard::Modifiers;
use iced_core::mouse::{self, Click};
use iced_core::widget::operation;
use iced_graphics::core::widget::{tree, Tree};
use iced_graphics::core::Widget;
use iced_graphics::geometry::Renderer;
use tokio::sync::mpsc::{self, Sender};

#[derive(Debug, Clone)]
pub enum Event {
    Scrolled(u64, f32),
    Resized(u64, Size<f32>),
    Ignored(u64),
    InputReceived(u64, Vec<u8>),
    SelectStarted(u64, SelectionType, (f32, f32)),
    SelectUpdated(u64, (f32, f32)),
    BackendEventSenderReceived(u64, Sender<alacritty_terminal::event::Event>),
    BackendEventReceived(u64, alacritty_terminal::event::Event),
}

#[derive(Debug, Clone)]
pub enum Command {
    InitBackend(Sender<alacritty_terminal::event::Event>),
    WriteToBackend(Vec<u8>),
    Scroll(i32),
    ChangeTheme(Box<ColorPalette>),
    AddBindings(Vec<(Binding<InputKind>, BindingAction)>),
    Resize(Size<f32>),
    SelectStart(SelectionType, (f32, f32)),
    SelectUpdate((f32, f32)),
    ProcessBackendEvent(alacritty_terminal::event::Event),
}

#[derive(Default, Clone)]
pub struct TermSettings {
    pub font: FontSettings,
    pub theme: ColorPalette,
    pub backend: BackendSettings,
}

pub struct Term {
    id: u64,
    font: TermFont,
    theme: TermTheme,
    cache: Cache,
    bindings: BindingsLayout,
    backend_settings: BackendSettings,
    backend: Option<Pty>,
}

impl Term {
    pub fn new(id: u64, settings: TermSettings) -> Self {
        Self {
            id,
            font: TermFont::new(settings.font),
            theme: TermTheme::new(Box::new(settings.theme)),
            bindings: BindingsLayout::default(),
            cache: Cache::default(),
            backend_settings: settings.backend,
            backend: None,
        }
    }

    pub fn widget_id(&self) -> iced::widget::text_input::Id {
        iced::widget::text_input::Id::new(self.id.to_string())
    }

    pub fn subscription(&self) -> Subscription<Event> {
        let id = self.id;
        iced::subscription::channel(id, 100, move |mut output| async move {
            let (event_tx, mut event_rx) = mpsc::channel(100);
            output
                .send(Event::BackendEventSenderReceived(id, event_tx))
                .await
                .unwrap_or_else(|_| {
                    panic!("ICED SUBSCRIPTION {}: sending BackendEventSenderReceived event is failed", id)
                });

            while let Some(event) = event_rx.recv().await {
                output
                    .send(Event::BackendEventReceived(id, event))
                    .await
                    .unwrap_or_else(|_| {
                        panic!("ICED SUBSCRIPTION {}: sending BackendEventReceived event is failed", id)
                    });
            }

            panic!("ICED SUBSCRIPTION {}: terminal event channel closed unexpected", id);
        })
    }

    pub fn update(&mut self, cmd: Command) {
        match cmd {
            Command::InitBackend(sender) => {
                self.backend = Some(
                    Pty::new(
                        self.id,
                        sender,
                        self.backend_settings.clone(),
                        self.font.measure(),
                    )
                        .unwrap_or_else(|_| {
                            panic!("init pty with ID: {} is failed", self.id);
                        }),
                );
            },
            Command::ProcessBackendEvent(event) => {
                if let alacritty_terminal::event::Event::Wakeup = event {
                    self.cache.clear();
                }
            },
            Command::WriteToBackend(input) => {
                if let Some(ref mut backend) = self.backend {
                    backend.write_to_pty(input);
                }
            },
            Command::Scroll(delta) => {
                if let Some(ref mut backend) = self.backend {
                    backend.scroll(delta);
                    self.cache.clear();
                }
            },
            Command::Resize(size) => {
                if let Some(ref mut backend) = self.backend {
                    let rows = (size.height / backend.size().cell_height as f32)
                        .floor() as u16;
                    let cols = (size.width / backend.size().cell_width as f32)
                        .floor() as u16;
                    let font_size = self.font.measure();
                    backend.resize(
                        rows,
                        cols,
                        font_size.width,
                        font_size.height,
                    );
                    self.cache.clear();
                }
            },
            Command::ChangeTheme(palette) => {
                self.theme = TermTheme::new(palette);
                self.cache.clear();
            },
            Command::AddBindings(bindings) => {
                self.bindings.add_bindings(bindings);
            },
            Command::SelectStart(selection_type, (x, y)) => {
                if let Some(ref mut backend) = self.backend {
                    backend.start_selection(
                        selection_type,
                        x,
                        y,
                    );
                    self.cache.clear();
                }
            },
            Command::SelectUpdate((x, y)) => {
                if let Some(ref mut backend) = self.backend {
                    backend.update_selection(
                        x,
                        y,
                    );
                    self.cache.clear();
                }  
            }
        }
    }
}

pub struct TermView<'a> {
    term: &'a Term,
    bindings: &'a BindingsLayout,
}

pub fn term_view(term: &Term) -> Element<'_, Event> {
    container(TermView::new(term, &term.bindings))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(iced::theme::Container::Custom(Box::new(term.theme.clone())))
        .into()
}

impl<'a> TermView<'a> {
    fn new(term: &'a Term, bindings: &'a BindingsLayout) -> Self {
        Self { term, bindings }
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
    ) -> Event {
        let mut term_event = Event::Ignored(self.term.id);
        if let Some(ref backend) = self.term.backend {
            match event {
                iced_core::mouse::Event::ButtonPressed(iced_core::mouse::Button::Left) => {
                    state.last_click = Some(Click::new(cursor_position, state.last_click));
                    let selction_type = match state.last_click {
                        Some(click) => match click.kind() {
                            mouse::click::Kind::Single => SelectionType::Simple,
                            mouse::click::Kind::Double => SelectionType::Semantic,
                            mouse::click::Kind::Triple => SelectionType::Lines,
                        }
                        None => SelectionType::Simple,
                    };
    
                    state.is_dragged = true;
                    term_event = Event::SelectStarted(
                        self.term.id,
                        selction_type,
                        (
                            cursor_position.x - layout_position.x,
                            cursor_position.y - layout_position.y
                        )
                    )
                },
                iced_core::mouse::Event::CursorMoved { position: _ } => {
                    if state.is_dragged {
                        term_event = Event::SelectUpdated(
                            self.term.id,
                            (
                                cursor_position.x - layout_position.x,
                                cursor_position.y - layout_position.y
                            )
                        );
                    }
                },
                iced_core::mouse::Event::ButtonReleased(iced_core::mouse::Button::Left) => {
                    state.is_dragged = false;
                },
                iced::mouse::Event::WheelScrolled { delta } => match delta {
                    ScrollDelta::Lines { x: _, y } => {
                        state.scroll_pixels = 0.0;
                        let lines = if y <= 0.0 { y.floor() } else { y.ceil() };
                        term_event = Event::Scrolled(self.term.id, lines)
                    },
                    ScrollDelta::Pixels { x: _, y } => {
                        state.scroll_pixels -= y;
                        let mut lines = 0;
                        let line_height = self.term.font.measure().height;
                        while state.scroll_pixels <= -line_height {
                            lines -= 1;
                            state.scroll_pixels += line_height;
                        }
                        while state.scroll_pixels >= line_height {
                            lines += 1;
                            state.scroll_pixels -= line_height;
                        }
    
                        term_event = Event::Scrolled(self.term.id, -lines as f32)
                    },
                },
                _ => {},
            }
        }

        term_event
    }

    fn handle_keyboard_event(
        &self,
        state: &mut TermViewState,
        clipboard: &mut dyn iced_graphics::core::Clipboard,
        event: iced::keyboard::Event,
    ) -> Event {
        if let Some(ref backend) = self.term.backend {
            let mut binding_action = BindingAction::Ignore;

            match event {
                iced::keyboard::Event::ModifiersChanged(m) => {
                    state.keyboard_modifiers = m;
                },
                iced::keyboard::Event::CharacterReceived(c) => {
                    binding_action = self.bindings.get_action(
                        InputKind::Char(c.to_ascii_lowercase()),
                        state.keyboard_modifiers,
                        backend.mode(),
                    );

                    if binding_action == BindingAction::Ignore
                        && !c.is_control()
                    {
                        let mut buf = [0, 0, 0, 0];
                        let str = c.encode_utf8(&mut buf);
                        return Event::InputReceived(
                            self.term.id,
                            str.as_bytes().to_vec(),
                        );
                    }
                },
                iced::keyboard::Event::KeyPressed {
                    key_code,
                    modifiers,
                } => {
                    binding_action = self.bindings.get_action(
                        InputKind::KeyCode(key_code),
                        modifiers,
                        backend.mode(),
                    );
                },
                _ => {},
            }

            match binding_action {
                BindingAction::Char(c) => {
                    let mut buf = [0, 0, 0, 0];
                    let str = c.encode_utf8(&mut buf);
                    return Event::InputReceived(
                        self.term.id,
                        str.as_bytes().to_vec(),
                    );
                },
                BindingAction::Esc(seq) => {
                    return Event::InputReceived(
                        self.term.id,
                        seq.as_bytes().to_vec(),
                    )
                },
                BindingAction::Paste => {
                    if let Some(data) = clipboard.read() {
                        let input: Vec<u8> = data.bytes().collect();
                        return Event::InputReceived(self.term.id, input);
                    }
                },
                _ => {},
            };
        }

        Event::Ignored(self.term.id)
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
        _tree: &Tree,
        renderer: &mut iced::Renderer<Theme>,
        _theme: &Theme,
        _style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout,
        _cursor: Cursor,
        viewport: &Rectangle,
    ) {
        let geom = self.term.cache.draw(renderer, viewport.size(), |frame| {
            if let Some(ref backend) = self.term.backend {
                let (content, selectable_range, term_mode, size) = backend.renderable_content();
                for indexed in content.display_iter() {
                    let cell_width = size.cell_width as f64;
                    let cell_height = size.cell_height as f64;
                    let x = indexed.point.column.0 as f64 * cell_width;
                    let y = (indexed.point.line.0 as f64
                        + content.display_offset() as f64)
                        * cell_height;

                    let mut fg = self.term.theme.get_color(indexed.fg);
                    let mut bg = self.term.theme.get_color(indexed.bg);

                    if indexed.cell.flags.contains(cell::Flags::INVERSE) {
                        std::mem::swap(&mut fg, &mut bg);
                    }

                    if let Some(range) = selectable_range {
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

                    if content.cursor.point == indexed.point {
                        let cursor_rect = Path::rectangle(
                            Point {
                                x: layout.position().x
                                    + content.cursor.point.column.0 as f32
                                        * cell_width as f32,
                                y: layout.position().y
                                    + (content.cursor.point.line.0
                                        + content.display_offset() as i32)
                                        as f32
                                        * cell_height as f32,
                            },
                            Size::new(cell_width as f32, cell_height as f32),
                        );

                        if !term_mode.contains(TermMode::ALT_SCREEN) {
                            frame.fill(&cursor_rect, fg);
                        }
                    }

                    if indexed.c != ' ' && indexed.c != '\t' {
                        if content.cursor.point == indexed.point {
                            std::mem::swap(&mut fg, &mut bg);
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
                            font: self.term.font.font_type(),
                            size: self.term.font.size(),
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
        if state.size != layout_size && self.term.backend.is_some() {
            state.size = layout_size;
            shell.publish(Event::Resized(self.term.id, layout_size));
        }

        if !state.is_focused {
            return iced::event::Status::Ignored;
        }

        let mut term_event = Event::Ignored(self.term.id);
        match event {
            iced::Event::Mouse(mouse_event) => {
                if self.is_cursor_in_layout(cursor, layout) {
                    let cursor_position = cursor
                        .position()
                        .unwrap();
                    term_event = self.handle_mouse_event(
                        state, 
                        layout.position(), 
                        cursor_position,
                        mouse_event
                    )
                }
            },
            iced::Event::Keyboard(keyboard_event) => {
                term_event = self.handle_keyboard_event(state, clipboard, keyboard_event)
            },
            _ => {}
        };

        match term_event {
            Event::Ignored(_) => iced::event::Status::Ignored,
            e => {
                shell.publish(e);
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
