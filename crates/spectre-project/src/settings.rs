// =============================================================================
// File: crates/spectre-project/src/settings.rs
// Layer: project persistence
// Purpose: Global application settings as human-readable TOML
// Status: Implemented; TOML round-trip with defaulted missing keys.
// Notes: Settings are app-global, not per-project. serde(default) on the struct
//        lets an older or partial file load by filling absent keys.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::serialize::ProjectError;

// Out-of-box audio and UI defaults
const DEFAULT_SAMPLE_RATE_HZ: u32 = 48_000;
const DEFAULT_BUFFER_SIZE: u32 = 512;
const DEFAULT_THEME: &str = "dark";
const MAX_RECENT_PROJECTS: usize = 16;

// Persistent global preferences shown in the app's settings panel
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    // Preferred output device name; None means the system default
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_device: Option<String>,
    pub sample_rate_hz: u32,
    pub buffer_size: u32,
    pub theme: String,
    pub recent_projects: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            audio_device: None,
            sample_rate_hz: DEFAULT_SAMPLE_RATE_HZ,
            buffer_size: DEFAULT_BUFFER_SIZE,
            theme: DEFAULT_THEME.to_string(),
            recent_projects: Vec::new(),
        }
    }
}

impl Settings {
    // Encode the settings to a pretty TOML string
    pub fn to_toml(&self) -> Result<String, ProjectError> {
        toml::to_string_pretty(self).map_err(|e| ProjectError::Encode(e.to_string()))
    }

    // Decode settings from a TOML string, filling any missing keys
    pub fn from_toml(text: &str) -> Result<Self, ProjectError> {
        toml::from_str(text).map_err(|e| ProjectError::Decode(e.to_string()))
    }

    // Write the settings to disk as TOML
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), ProjectError> {
        std::fs::write(path, self.to_toml()?)?;
        Ok(())
    }

    // Read settings from a TOML file
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ProjectError> {
        let text = std::fs::read_to_string(path)?;
        Self::from_toml(&text)
    }

    // Record a freshly opened project, most-recent first, de-duplicated
    pub fn push_recent(&mut self, path: impl Into<String>) {
        let path = path.into();
        self.recent_projects.retain(|p| p != &path);
        self.recent_projects.insert(0, path);
        self.recent_projects.truncate(MAX_RECENT_PROJECTS);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path() -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("spectre_settings_{nanos}.toml"))
    }

    #[test]
    fn defaults_are_sensible() {
        let s = Settings::default();
        assert_eq!(s.sample_rate_hz, 48_000);
        assert_eq!(s.buffer_size, 512);
        assert_eq!(s.theme, "dark");
        assert!(s.audio_device.is_none());
    }

    #[test]
    fn toml_round_trip_matches() {
        let mut s = Settings {
            audio_device: Some("BlackHole 2ch".into()),
            theme: "light".into(),
            ..Settings::default()
        };
        s.push_recent("/songs/a.geist");
        let text = s.to_toml().unwrap();
        let back = Settings::from_toml(&text).unwrap();
        assert_eq!(s, back);
    }

    #[test]
    fn partial_toml_fills_defaults() {
        let s = Settings::from_toml("theme = \"contrast\"").unwrap();
        assert_eq!(s.theme, "contrast");
        assert_eq!(s.sample_rate_hz, 48_000); // defaulted
        assert_eq!(s.buffer_size, 512); // defaulted
    }

    #[test]
    fn path_round_trip_matches() {
        let s = Settings::default();
        let path = temp_path();
        s.save(&path).unwrap();
        let loaded = Settings::load(&path).unwrap();
        std::fs::remove_file(&path).ok();
        assert_eq!(s, loaded);
    }

    #[test]
    fn push_recent_dedups_and_orders_most_recent_first() {
        let mut s = Settings::default();
        s.push_recent("a");
        s.push_recent("b");
        s.push_recent("a"); // re-opening a moves it back to the front
        assert_eq!(s.recent_projects, vec!["a".to_string(), "b".to_string()]);
    }
}
