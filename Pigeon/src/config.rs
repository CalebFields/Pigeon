use std::path::PathBuf;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct AppConfig {
    pub data_dir: PathBuf,
    pub log_level: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            data_dir: default_data_dir(),
            log_level: "info".to_string(),
        }
    }
}

#[allow(dead_code)]
pub fn load() -> AppConfig {
    // Placeholder: M0-020 will implement TOML + env overrides
    AppConfig::default()
}

fn default_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("pigeon")
}


