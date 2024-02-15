const DEFAULT_SHELL: &str = "/bin/bash";

#[derive(Debug, Clone)]
pub struct BackendSettings {
    pub shell: String,
}

impl Default for BackendSettings {
    fn default() -> Self {
        Self {
            shell: DEFAULT_SHELL.to_string(),
        }
    }
}
