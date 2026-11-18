//! Small preferences file stored next to the user configuration.

use std::env;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub last_source_dir: Option<PathBuf>,
    pub last_output_dir: Option<PathBuf>,
    pub overwrite: bool,
    pub open_after: bool,
    pub dark_theme: bool,
    pub window_size: Option<[f32; 2]>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            last_source_dir: None,
            last_output_dir: None,
            overwrite: false,
            open_after: true,
            dark_theme: true,
            window_size: None,
        }
    }
}

impl Settings {
    pub fn path() -> Option<PathBuf> {
        config_dir().map(|directory| directory.join("jconverter").join("settings.json"))
    }

    /// Load saved preferences; unreadable or corrupt files fall back to defaults.
    pub fn load() -> Self {
        let Some(path) = Self::path() else {
            return Self::default();
        };
        let Ok(text) = std::fs::read_to_string(&path) else {
            return Self::default();
        };
        serde_json::from_str(&text).unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), String> {
        let Some(path) = Self::path() else {
            return Err("no configuration directory is available".to_string());
        };
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("could not create {}: {error}", parent.display()))?;
        }
        let text = serde_json::to_string_pretty(self)
            .map_err(|error| format!("could not encode settings: {error}"))?;
        std::fs::write(&path, text)
            .map_err(|error| format!("could not write {}: {error}", path.display()))
    }
}

fn config_dir() -> Option<PathBuf> {
    if let Some(directory) = env::var_os("XDG_CONFIG_HOME").filter(|value| !value.is_empty()) {
        return Some(PathBuf::from(directory));
    }

    #[cfg(target_os = "windows")]
    if let Some(directory) = env::var_os("APPDATA").filter(|value| !value.is_empty()) {
        return Some(PathBuf::from(directory));
    }

    let home = env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .filter(|value| !value.is_empty())?;
    let home = PathBuf::from(home);

    #[cfg(target_os = "macos")]
    return Some(home.join("Library").join("Application Support"));

    #[cfg(not(target_os = "macos"))]
    Some(home.join(".config"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_user_friendly() {
        let settings = Settings::default();
        assert!(!settings.overwrite);
        assert!(settings.open_after);
        assert!(settings.dark_theme);
        assert!(settings.window_size.is_none());
    }

    #[test]
    fn corrupt_settings_fall_back_to_defaults() {
        let parsed: Settings = serde_json::from_str("{\"overwrite\": ").unwrap_or_default();
        assert!(!parsed.overwrite);
    }

    #[test]
    fn round_trips_through_json() {
        let settings = Settings {
            overwrite: true,
            window_size: Some([800.0, 600.0]),
            ..Settings::default()
        };
        let text = serde_json::to_string(&settings).unwrap();
        let parsed: Settings = serde_json::from_str(&text).unwrap();
        assert!(parsed.overwrite);
        assert_eq!(parsed.window_size, Some([800.0, 600.0]));
    }
}
