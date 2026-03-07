use crate::theme::Theme;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const MAX_RECENT_FILES: usize = 10;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub theme: String,
    #[serde(default)]
    pub recent_files: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            recent_files: Vec::new(),
        }
    }
}

fn config_path() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    Some(PathBuf::from(home).join(".config/mdreader/config.json"))
}

fn load_config() -> Config {
    let Some(path) = config_path() else {
        return Config::default();
    };
    let Ok(data) = std::fs::read_to_string(&path) else {
        return Config::default();
    };
    serde_json::from_str::<Config>(&data).unwrap_or_default()
}

fn save_config(config: &Config) {
    let Some(path) = config_path() else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(config) {
        let _ = std::fs::write(&path, json);
    }
}

pub fn load_theme() -> Theme {
    let config = load_config();
    match config.theme.as_str() {
        "light" => Theme::Light,
        _ => Theme::Dark,
    }
}

pub fn save_theme(theme: Theme) {
    let mut config = load_config();
    config.theme = match theme {
        Theme::Light => "light",
        Theme::Dark => "dark",
    }
    .to_string();
    save_config(&config);
}

pub fn load_recent_files() -> Vec<PathBuf> {
    let config = load_config();
    config
        .recent_files
        .iter()
        .map(PathBuf::from)
        .filter(|p| p.exists())
        .collect()
}

pub fn add_recent_file(path: &std::path::Path) {
    let mut config = load_config();
    let path_str = path.to_string_lossy().to_string();

    // Remove if already present, then push to front
    config.recent_files.retain(|p| p != &path_str);
    config.recent_files.insert(0, path_str);
    config.recent_files.truncate(MAX_RECENT_FILES);

    save_config(&config);
}
