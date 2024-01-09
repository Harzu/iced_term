use iced::executor;
use iced::font::{Family, Stretch, Weight};
use iced::theme::{self, Theme};
use iced::widget::pane_grid::{self, PaneGrid};
use iced::widget::{button, container, responsive, row, text};
use iced::{alignment, Font};
use iced::{
    window, Application, Color, Command, Element, Length, Settings,
    Subscription,
};
use std::collections::HashMap;

const TERM_FONT_JET_BRAINS_BYTES: &[u8] =
    include_bytes!("../assets/fonts/JetBrains/JetBrainsMono-Bold.ttf");

pub fn main() -> iced::Result {
    Example::run(Settings {
        antialiasing: true,
        default_font: Font::MONOSPACE,
        window: window::Settings {
            size: (800, 600),
            ..window::Settings::default()
        },
        ..Settings::default()
    })
}

struct Example {
    panes: pane_grid::State<Pane>,
    tabs: HashMap<u64, iced_term::Term>,
    term_settings: iced_term::TermSettings,
    panes_created: usize,
    focus: Option<pane_grid::Pane>,
}

#[derive(Debug, Clone)]
enum Message {
    Split(pane_grid::Axis, pane_grid::Pane),
    Clicked(pane_grid::Pane),
    Resized(pane_grid::ResizeEvent),
    Close(pane_grid::Pane),
    TermEvent(iced_term::Event),
    FontLoaded(Result<(), iced::font::Error>),
}

impl Application for Example {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let initial_pane_id = 0;
        let (panes, _) = pane_grid::State::new(Pane::new(initial_pane_id));
        let term_settings = iced_term::TermSettings {
            font: iced_term::FontSettings {
                size: 14.0,
                font_type: Font {
                    weight: Weight::Bold,
                    family: Family::Name("JetBrains Mono"),
                    monospaced: false,
                    stretch: Stretch::Normal,
                },
                ..iced_term::FontSettings::default()
            },
            backend: iced_term::BackendSettings {
                shell: env!("SHELL").to_string(),
                ..iced_term::BackendSettings::default()
            },
        };
        let tab =
            iced_term::Term::new(initial_pane_id as u64, term_settings.clone());
        let mut tabs = HashMap::new();
        tabs.insert(initial_pane_id as u64, tab);

        (
            Example {
                panes,
                panes_created: 1,
                tabs,
                term_settings,
                focus: None,
            },
            Command::batch(vec![iced::font::load(TERM_FONT_JET_BRAINS_BYTES)
                .map(Message::FontLoaded)]),
        )
    }

    fn title(&self) -> String {
        String::from("Terminal with split panes")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::FontLoaded(_) => {},
            Message::Split(axis, pane) => {
                let result = self.panes.split(
                    axis,
                    &pane,
                    Pane::new(self.panes_created),
                );

                let tab = iced_term::Term::new(
                    self.panes_created as u64,
                    self.term_settings.clone(),
                );
                self.tabs.insert(self.panes_created as u64, tab);

                if let Some((pane, _)) = result {
                    let prev_focused_tab_id = (self.panes_created - 1) as u64;
                    if let Some(prev_focused_tab) =
                        self.tabs.get_mut(&prev_focused_tab_id)
                    {
                        prev_focused_tab.update(iced_term::Command::LostFocus);
                    }
                    self.focus = Some(pane);
                }

                self.panes_created += 1;
            },
            Message::Clicked(pane) => {
                if let Some(pane_id) = &self.focus {
                    let focused_pane = self.panes.get(pane_id).unwrap();
                    let prev_focused_tab =
                        self.tabs.get_mut(&(focused_pane.id as u64)).unwrap();
                    prev_focused_tab.update(iced_term::Command::LostFocus);
                }

                let new_focused_pane = self.panes.get(&pane).unwrap();
                let new_focused_tab =
                    self.tabs.get_mut(&(new_focused_pane.id as u64)).unwrap();
                new_focused_tab.update(iced_term::Command::Focus);

                self.focus = Some(pane);
            },
            Message::Resized(pane_grid::ResizeEvent { split, ratio }) => {
                self.panes.resize(&split, ratio);
            },
            Message::Close(pane) => {
                if let Some((closed_pane, sibling)) = self.panes.close(&pane) {
                    let tab_id = closed_pane.id as u64;
                    self.tabs.remove(&tab_id);
                    self.focus = Some(sibling);
                }
            },
            Message::TermEvent(event) => {
                match event {
                    iced_term::Event::InputReceived(id, data) => {
                        if let Some(tab) = self.tabs.get_mut(&id) {
                            tab.update(iced_term::Command::WriteToBackend(data))
                        }
                    },
                    iced_term::Event::Scrolled(id, delta) => {
                        if let Some(tab) = self.tabs.get_mut(&id) {
                            tab.update(iced_term::Command::Scroll(delta as i32))
                        }
                    },
                    iced_term::Event::Resized(id, size) => {
                        if let Some(tab) = self.tabs.get_mut(&id) {
                            tab.update(iced_term::Command::Resize(size));
                        }
                    },
                    iced_term::Event::BackendEventSenderReceived(id, tx) => {
                        if let Some(tab) = self.tabs.get_mut(&id) {
                            tab.update(iced_term::Command::InitBackend(tx));
                        }
                    },
                    iced_term::Event::BackendEventReceived(id, inner_event) => {
                        if let Some(tab) = self.tabs.get_mut(&id) {
                            tab.update(
                                iced_term::Command::ProcessBackendEvent(
                                    inner_event,
                                ),
                            );
                        }
                    },
                    _ => {},
                };
            },
        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let focus = self.focus;
        let total_panes = self.panes.len();

        let pane_grid = PaneGrid::new(&self.panes, |id, pane, _| {
            let is_focused = focus == Some(id);
            let title = row![
                "Pane",
                text(pane.id.to_string()).style(if is_focused {
                    PANE_ID_COLOR_FOCUSED
                } else {
                    PANE_ID_COLOR_UNFOCUSED
                }),
            ]
            .spacing(5);

            let title_bar = pane_grid::TitleBar::new(title)
                .controls(view_controls(id, total_panes, pane.is_pinned))
                .padding(10)
                .style(if is_focused {
                    style::title_bar_focused
                } else {
                    style::title_bar_active
                });

            let pane_id = pane.id as u64;
            pane_grid::Content::new(responsive(move |_| {
                view_content(pane_id, &self.tabs)
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
        .on_resize(10, Message::Resized);

        container(pane_grid)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut sb = vec![];
        for id in self.tabs.keys() {
            let tab = self.tabs.get(id).unwrap();
            let sub = tab.subscription().map(Message::TermEvent);
            sb.push(sub)
        }

        Subscription::batch(sb)
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

fn view_content(
    pane_id: u64,
    tabs: &HashMap<u64, iced_term::Term>,
) -> Element<'_, Message> {
    let tab = tabs.get(&pane_id).expect("tab with target id not found");
    let tab_view = tab.view().map(Message::TermEvent);

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
) -> Element<'a, Message> {
    let mut row = row![].spacing(5);
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

    row = row.push(button(
        "Split horizontally",
        Message::Split(pane_grid::Axis::Horizontal, pane),
    ));

    row = row.push(button(
        "Split vertically",
        Message::Split(pane_grid::Axis::Vertical, pane),
    ));

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
