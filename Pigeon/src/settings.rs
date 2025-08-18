use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AccessibilitySettings {
	pub reduce_motion: bool,
	pub high_contrast: bool,
	pub larger_text: bool,
}

fn file_path(data_dir: &Path) -> PathBuf {
	data_dir.join("a11y.toml")
}

pub fn load_accessibility_settings(data_dir: &Path) -> Result<AccessibilitySettings, std::io::Error> {
	let path = file_path(data_dir);
	if !path.exists() {
		return Ok(AccessibilitySettings::default());
	}
	let s = fs::read_to_string(&path)?;
	toml::from_str::<AccessibilitySettings>(&s).map_err(std::io::Error::other)
}

pub fn save_accessibility_settings(
	data_dir: &Path,
	settings: &AccessibilitySettings,
) -> Result<(), std::io::Error> {
	fs::create_dir_all(data_dir)?;
	let s = toml::to_string_pretty(settings).map_err(std::io::Error::other)?;
	fs::write(file_path(data_dir), s)
}


// Application state (e.g., onboarding status)

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AppState {
	pub onboarded: bool,
}

fn app_state_path(data_dir: &Path) -> PathBuf {
	data_dir.join("app_state.toml")
}

pub fn load_app_state(data_dir: &Path) -> Result<AppState, std::io::Error> {
	let path = app_state_path(data_dir);
	if !path.exists() {
		return Ok(AppState::default());
	}
	let s = fs::read_to_string(&path)?;
	toml::from_str::<AppState>(&s).map_err(std::io::Error::other)
}

pub fn save_app_state(data_dir: &Path, state: &AppState) -> Result<(), std::io::Error> {
	fs::create_dir_all(data_dir)?;
	let s = toml::to_string_pretty(state).map_err(std::io::Error::other)?;
	fs::write(app_state_path(data_dir), s)
}


