use crate::theme::Theme;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub theme: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
        }
    }
}

fn config_path() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    Some(PathBuf::from(home).join(".config/mdreader/config.json"))
}

pub fn load_theme() -> Theme {
    let Some(path) = config_path() else {
        return Theme::default();
    };
    let Ok(data) = std::fs::read_to_string(&path) else {
        return Theme::default();
    };
    let Ok(config) = serde_json::from_str::<Config>(&data) else {
        return Theme::default();
    };
    match config.theme.as_str() {
        "light" => Theme::Light,
        _ => Theme::Dark,
    }
}

pub fn save_theme(theme: Theme) {
    let Some(path) = config_path() else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let config = Config {
        theme: match theme {
            Theme::Light => "light",
            Theme::Dark => "dark",
        }
        .to_string(),
    };
    if let Ok(json) = serde_json::to_string_pretty(&config) {
        let _ = std::fs::write(&path, json);
    }
}
