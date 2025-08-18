use std::{env, fs, path::PathBuf};
use serde::Deserialize;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct AppConfig {
    pub data_dir: PathBuf,
    pub log_level: String,
    #[cfg(feature = "network")]
    pub listen_addr: Option<String>,
    #[cfg(feature = "network")]
    pub enable_mdns: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            data_dir: default_data_dir(),
            log_level: "info".to_string(),
            #[cfg(feature = "network")]
            listen_addr: None,
            #[cfg(feature = "network")]
            enable_mdns: false,
        }
    }
}

#[allow(dead_code)]
pub fn load() -> AppConfig {
    let mut cfg = AppConfig::default();

    // Ensure default config file exists with sections
    ensure_default_file();

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
    #[cfg(feature = "network")]
    {
        if let Ok(addr) = env::var("PIGEON_LISTEN_ADDR") {
            cfg.listen_addr = Some(addr);
        }
        if let Ok(v) = env::var("PIGEON_ENABLE_MDNS") {
            let v = v.to_ascii_lowercase();
            cfg.enable_mdns = v == "1" || v == "true" || v == "yes";
        }
    }

    cfg
}

fn load_from_file() -> Option<AppConfig> {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    let path = base.join("pigeon").join("config.toml");
    let contents = fs::read_to_string(&path).ok()?;
    toml::from_str::<RootConfig>(&contents)
        .ok()
        .map(|f| f.into_app_config())
}

#[derive(Deserialize, Debug, Default)]
#[allow(dead_code)]
struct RootConfig {
    // Back-compat flat keys
    data_dir: Option<PathBuf>,
    log_level: Option<String>,
    #[cfg(feature = "network")]
    listen_addr: Option<String>,
    #[cfg(feature = "network")]
    enable_mdns: Option<bool>,
    // Sectioned config
    storage: Option<StorageSection>,
    network: Option<NetworkSection>,
    security: Option<SecuritySection>,
}

#[derive(Deserialize, Debug, Default)]
struct StorageSection { data_dir: Option<PathBuf> }

#[derive(Deserialize, Debug, Default)]
#[allow(dead_code)]
struct NetworkSection {
    #[cfg(feature = "network")]
    listen_addr: Option<String>,
    #[cfg(feature = "network")]
    enable_mdns: Option<bool>,
}

#[derive(Deserialize, Debug, Default)]
#[allow(dead_code)]
struct SecuritySection {
    // Placeholder for future options (e.g., encrypt_at_rest)
}

impl RootConfig {
    fn into_app_config(self) -> AppConfig {
        let mut cfg = AppConfig::default();
        // storage section or flat
        if let Some(st) = self.storage {
            if let Some(d) = st.data_dir { cfg.data_dir = d; }
        }
        if let Some(d) = self.data_dir { cfg.data_dir = d; }
        if let Some(l) = self.log_level { cfg.log_level = l; }
        #[cfg(feature = "network")]
        {
            if let Some(net) = self.network {
                if let Some(a) = net.listen_addr { cfg.listen_addr = Some(a); }
                if let Some(m) = net.enable_mdns { cfg.enable_mdns = m; }
            }
            if let Some(a) = self.listen_addr { cfg.listen_addr = Some(a); }
            if let Some(m) = self.enable_mdns { cfg.enable_mdns = m; }
        }
        cfg
    }
}

fn default_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("pigeon")
}

fn ensure_default_file() {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    let dir = base.join("pigeon");
    let path = dir.join("config.toml");
    if path.exists() { return; }
    let _ = fs::create_dir_all(&dir);
    let data_dir = default_data_dir();
    let tmpl = format!(
        "# Pigeon config\n\n[storage]\n# Where to store app data (dbs, keys, inbox)\n# data_dir can be overridden by PIGEON_DATA_DIR\n# Default below is the OS data dir\n# On Windows: %APPDATA%/pigeon\n# On Linux: ~/.local/share/pigeon\n# On macOS: ~/Library/Application Support/pigeon\n#\n# data_dir = \"{data}\"\n\n[network]\n# listen_addr example: \"/ip4/0.0.0.0/tcp/4001\"\n# enable_mdns = false\n\n[security]\n# Reserved for future options (e.g., encrypt_at_rest)\n",
        data = data_dir.display()
    );
    let _ = fs::write(path, tmpl);
}


