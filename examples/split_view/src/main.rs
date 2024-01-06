use std::collections::HashMap;

use iced::alignment::{self, Alignment};
use iced::executor;
use iced::keyboard;
use iced::theme::{self, Theme};
use iced::widget::pane_grid::{self, PaneGrid};
use iced::widget::{
    button, column, container, responsive, row, scrollable, text,
};
use iced::{
    window,
    Application, Color, Command, Element, Length, Settings, Size, Subscription,
};
use iced_term::Pty;
use iced_term::{self, Event, Term};

pub fn main() -> iced::Result {
    Example::run(Settings {
        antialiasing: true,
        window: window::Settings {
            size: (800, 600),
            ..window::Settings::default()
        },
        ..Settings::default()
    })
}

struct Example {
    panes: pane_grid::State<Pane>,
    tabs: HashMap<u64, Term>,
    panes_created: usize,
    focus: Option<pane_grid::Pane>,
}

#[derive(Debug, Clone)]
enum Message {
    Split(pane_grid::Axis, pane_grid::Pane),
    SplitFocused(pane_grid::Axis),
    FocusAdjacent(pane_grid::Direction),
    Clicked(pane_grid::Pane),
    Dragged(pane_grid::DragEvent),
    Resized(pane_grid::ResizeEvent),
    TogglePin(pane_grid::Pane),
    Maximize(pane_grid::Pane),
    Restore,
    Close(pane_grid::Pane),
    CloseFocused,
    TermMessage(Event),
    // GlobalEvent(iced::Event),
}

impl Application for Example {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let initial_pane_id = 0;
        let (panes, _) = pane_grid::State::new(Pane::new(initial_pane_id));
        let tab = iced_term::Term::new(initial_pane_id as u64, 10.0);
        let mut tabs = HashMap::new();
        tabs.insert(initial_pane_id as u64, tab);

        (
            Example {
                panes,
                panes_created: 1,
                tabs: tabs,
                focus: None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Pane grid - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Split(axis, pane) => {
                let result =
                    self.panes.split(axis, &pane, Pane::new(self.panes_created));

                let tab = Term::new(self.panes_created as u64, 14.0);
                self.tabs.insert(self.panes_created as u64, tab);

                if let Some((pane, _)) = result {
                    let prev_focused_tab_id = (self.panes_created - 1) as u64;
                    let prev_focused_tab = self.tabs.get_mut(&prev_focused_tab_id).expect("init pty is failed");
                    prev_focused_tab.update(iced_term::Command::LostFocus);
                    self.focus = Some(pane);
                }

                self.panes_created += 1;
            }
            Message::SplitFocused(axis) => {
                if let Some(pane) = self.focus {
                    let result = self.panes.split(
                        axis,
                        &pane,
                        Pane::new(self.panes_created),
                    );

                    if let Some((pane, _)) = result {
                        self.focus = Some(pane);
                    }

                    self.panes_created += 1;
                }
            }
            Message::FocusAdjacent(direction) => {
                if let Some(pane) = self.focus {
                    if let Some(adjacent) = self.panes.adjacent(&pane, direction)
                    {
                        self.focus = Some(adjacent);
                    }
                }
            }
            Message::Clicked(pane) => {
                if let Some(pane_id) = &self.focus {
                    let focused_pane = self.panes.get(pane_id).unwrap();
                    let prev_focused_tab = self.tabs.get_mut(&(focused_pane.id as u64)).unwrap();
                    prev_focused_tab.update(iced_term::Command::LostFocus);
                }

                let new_focused_pane = self.panes.get(&pane).unwrap();
                let new_focused_tab = self.tabs.get_mut(&(new_focused_pane.id as u64)).unwrap();
                new_focused_tab.update(iced_term::Command::Focus);

                self.focus = Some(pane);
            }
            Message::Resized(pane_grid::ResizeEvent { split, ratio }) => {
                println!("{:?}, {}", split, ratio);
                // self.panes.iter().for_each(|p| {
                //     // p.0.
                // });
                // self.panes
                self.panes.resize(&split, ratio);
            }
            Message::Dragged(pane_grid::DragEvent::Dropped {
                pane,
                target,
            }) => {
                self.panes.drop(&pane, target);
            }
            Message::Dragged(_) => {}
            Message::TogglePin(pane) => {
                if let Some(Pane { is_pinned, .. }) = self.panes.get_mut(&pane) {
                    *is_pinned = !*is_pinned;
                }
            }
            Message::Maximize(pane) => self.panes.maximize(&pane),
            Message::Restore => {
                self.panes.restore();
            }
            Message::Close(pane) => {
                if let Some((_, sibling)) = self.panes.close(&pane) {
                    self.focus = Some(sibling);
                }
            }
            Message::CloseFocused => {
                if let Some(pane) = self.focus {
                    if let Some(Pane { is_pinned, .. }) = self.panes.get(&pane) {
                        if !is_pinned {
                            if let Some((_, sibling)) = self.panes.close(&pane) {
                                self.focus = Some(sibling);
                            }
                        }
                    }
                }
            },
            Message::TermMessage(m) => {
                match m {
                    Event::InputReceived(id, c) => {
                        let tab = self.tabs.get_mut(&id).expect("tab with target id not found");
                        tab.update(iced_term::Command::WriteToPTY(c))
                    },
                    Event::DataUpdated(id, data) => {
                        let tab = self.tabs.get_mut(&id).expect("tab with target id not found");
                        tab.update(iced_term::Command::RenderData(data))
                    },
                    Event::ContainerScrolled(id, delta) => {
                        let tab = self.tabs.get_mut(&id).expect("tab with target id not found");
                        tab.update(iced_term::Command::Scroll(delta.1 as i32))
                    }
                    Event::Resized(id, size) => {
                        let tab = self.tabs.get_mut(&id).expect("tab with target id not found");
                        tab.update(iced_term::Command::Resize(size));
                    }
                    _ => {}
                    _ => {}
                };
            },
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut sb = vec![];
        for id in self.tabs.keys() {
            let tab = self.tabs.get(id).unwrap();
            let sub = iced_term::data_received_subscription(id.clone(), tab.pty_data_reader())
                .map(|e| Message::TermMessage(e));

            sb.push(sub)
        }

        Subscription::batch(sb)
    }

    fn view(&self) -> Element<Message> {
        let focus = self.focus;
        let total_panes = self.panes.len();

        let pane_grid = PaneGrid::new(&self.panes, |id, pane, is_maximized| {
            let is_focused = focus == Some(id);

            let pin_button = button(
                text(if pane.is_pinned { "Unpin" } else { "Pin" }).size(14),
            )
            .on_press(Message::TogglePin(id))
            .padding(3);

            let title = row![
                pin_button,
                "Pane",
                text(pane.id.to_string()).style(if is_focused {
                    PANE_ID_COLOR_FOCUSED
                } else {
                    PANE_ID_COLOR_UNFOCUSED
                }),
            ]
            .spacing(5);

            let title_bar = pane_grid::TitleBar::new(title)
                .controls(view_controls(
                    id,
                    total_panes,
                    pane.is_pinned,
                    is_maximized,
                ))
                .padding(10)
                .style(if is_focused {
                    style::title_bar_focused
                } else {
                    style::title_bar_active
                });

            let pane_id = pane.id.clone() as u64;
            // let tab = self.tabs.get(&pane_id).expect("tab with target id not found");
            pane_grid::Content::new(responsive(move |size| {
                view_content(id, pane_id, &self.tabs, total_panes, pane.is_pinned, size)
            }))
            .title_bar(title_bar)
            .style(if is_focused {
                style::pane_focused
            } else {
                style::pane_active
            })
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(10)
        .on_click(Message::Clicked)
        .on_drag(Message::Dragged)
        .on_resize(10, Message::Resized);

        container(pane_grid)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
            .into()
    }
}

const PANE_ID_COLOR_UNFOCUSED: Color = Color::from_rgb(
    0xFF as f32 / 255.0,
    0xC7 as f32 / 255.0,
    0xC7 as f32 / 255.0,
);
const PANE_ID_COLOR_FOCUSED: Color = Color::from_rgb(
    0xFF as f32 / 255.0,
    0x47 as f32 / 255.0,
    0x47 as f32 / 255.0,
);

#[derive(Clone, Copy)]
struct Pane {
    pub id: usize,
    pub is_pinned: bool,
}

impl Pane {
    fn new(id: usize) -> Self {
        Self {
            id,
            is_pinned: false,
        }
    }
}

fn view_content<'a>(
    pane: pane_grid::Pane,
    // tab: &'a Term,
    pane_id: u64,
    tabs: &'a HashMap<u64, Term>,
    total_panes: usize,
    is_pinned: bool,
    size: Size,
) -> Element<'a, Message> {
    let tab = tabs.get(&pane_id).expect("tab with target id not found");
    let tab_view = tab.view()
        .map(move |e| Message::TermMessage(e));

    container(tab_view)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(5)
        .into()
}

fn view_controls<'a>(
    pane: pane_grid::Pane,
    total_panes: usize,
    is_pinned: bool,
    is_maximized: bool,
) -> Element<'a, Message> {
    let mut row = row![].spacing(5);

    if total_panes > 1 {
        let toggle = {
            let (content, message) = if is_maximized {
                ("Restore", Message::Restore)
            } else {
                ("Maximize", Message::Maximize(pane))
            };
            button(text(content).size(14))
                .style(theme::Button::Secondary)
                .padding(3)
                .on_press(message)
        };

        row = row.push(toggle);
    }

    let mut close = button(text("Close").size(14))
        .style(theme::Button::Destructive)
        .padding(3);

    if total_panes > 1 && !is_pinned {
        close = close.on_press(Message::Close(pane));
    }

    let button = |label, message| {
        button(
            text(label)
                .width(Length::Fill)
                .horizontal_alignment(alignment::Horizontal::Center)
                .size(16),
        )
        .width(Length::Fill)
        .padding(8)
        .on_press(message)
    };

    row = row.push(
        button(
            "Split horizontally",
            Message::Split(pane_grid::Axis::Horizontal, pane),
        )
    );

    row = row.push(
        button(
            "Split vertically",
            Message::Split(pane_grid::Axis::Vertical, pane),
        )
    );

    row.push(close).into()
}

mod style {
    use iced::widget::container;
    use iced::Theme;

    pub fn title_bar_active(theme: &Theme) -> container::Appearance {
        let palette = theme.extended_palette();

        container::Appearance {
            text_color: Some(palette.background.strong.text),
            background: Some(palette.background.strong.color.into()),
            ..Default::default()
        }
    }

    pub fn title_bar_focused(theme: &Theme) -> container::Appearance {
        let palette = theme.extended_palette();

        container::Appearance {
            text_color: Some(palette.primary.strong.text),
            background: Some(palette.primary.strong.color.into()),
            ..Default::default()
        }
    }

    pub fn pane_active(theme: &Theme) -> container::Appearance {
        let palette = theme.extended_palette();

        container::Appearance {
            background: Some(palette.background.weak.color.into()),
            border_width: 2.0,
            border_color: palette.background.strong.color,
            ..Default::default()
        }
    }

    pub fn pane_focused(theme: &Theme) -> container::Appearance {
        let palette = theme.extended_palette();

        container::Appearance {
            background: Some(palette.background.weak.color.into()),
            border_width: 2.0,
            border_color: palette.primary.strong.color,
            ..Default::default()
        }
    }
}