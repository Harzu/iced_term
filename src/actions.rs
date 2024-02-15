#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Redraw,
    Shutdown,
    ChangeTitle,
    Ignore,
}
