#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Redraw,
    Shutdown,
    Ignore,
    ChangeTitle,
}
