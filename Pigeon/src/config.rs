use std::{env, fs, path::PathBuf};

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
    let mut cfg = AppConfig::default();

    // Load from config file if present
    if let Some(file_cfg) = load_from_file() {
        cfg = file_cfg;
    }

    // Apply env overrides
    if let Ok(dir) = env::var("PIGEON_DATA_DIR") {
        cfg.data_dir = PathBuf::from(dir);
    }
    if let Ok(level) = env::var("PIGEON_LOG_LEVEL") {
        cfg.log_level = level;
    }

    cfg
}

fn load_from_file() -> Option<AppConfig> {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    let path = base.join("pigeon").join("config.toml");
    let contents = fs::read_to_string(&path).ok()?;
    toml::from_str::<FileConfig>(&contents)
        .ok()
        .map(|f| f.into_app_config())
}

#[derive(serde::Deserialize, Debug, Default)]
struct FileConfig {
    data_dir: Option<PathBuf>,
    log_level: Option<String>,
}

impl FileConfig {
    fn into_app_config(self) -> AppConfig {
        let mut cfg = AppConfig::default();
        if let Some(d) = self.data_dir { cfg.data_dir = d; }
        if let Some(l) = self.log_level { cfg.log_level = l; }
        cfg
    }
}

fn default_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("pigeon")
}


