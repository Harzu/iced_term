use tokio::sync::mpsc;

const DEFAULT_SHELL: &str = "/bin/bash";
const DEFAULT_COLS_NUM: u16 = 50;
const DEFAULT_ROWS_NUM: u16 = 50;

#[derive(Debug, Clone)]
pub struct BackendSettings {
    pub shell: String,
    pub cols: u16,
    pub rows: u16,
}

impl Default for BackendSettings {
    fn default() -> Self {
        Self {
            shell: DEFAULT_SHELL.to_string(),
            cols: DEFAULT_COLS_NUM,
            rows: DEFAULT_ROWS_NUM,
        }
    }
}
