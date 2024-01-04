const DEFAULT_LINUX_SHELL: &str = "/bin/bash";
const DEFAULT_COLS_NUM: u16 = 50;
const DEFAULT_ROWS_NUM: u16 = 50;

pub struct Settings {
    pub shell: String,
    pub cols: u16,
    pub rows: u16,
}

impl Default for  Settings {
    fn default() -> Self {
        Self {
            shell: DEFAULT_LINUX_SHELL.to_string(),
            cols: DEFAULT_COLS_NUM,
            rows: DEFAULT_ROWS_NUM,
        }
    }
}

impl Settings {
    pub fn set_size(&mut self, cols: u16, rows: u16) {
        self.rows = rows;
        self.cols = cols;
    }

    pub fn set_shell(&mut self, shell_path: &str) {
        self.shell = shell_path.to_string();
    }
}