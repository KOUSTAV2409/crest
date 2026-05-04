use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const DEFAULT_HOTKEY: &str = "super+Space";

/// How extensions in the plugins directory are discovered and executed.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PluginPolicy {
    /// Only entries listed in `plugins/manifest.json` may execute (recommended).
    #[default]
    Manifest,
    /// Legacy: execute any executable file placed in the plugins directory (full trust).
    Open,
}

/// User-editable Crest settings at ~/.config/crest/config.json (or XDG equivalent).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    /// Global launcher shortcut parsed by [`global_hotkey`](https://docs.rs/global-hotkey) (modifiers first), e.g. `super+Space`, `alt+Esc`.
    pub global_shortcut: String,
    /// Extension loading policy (see README for plugins).
    #[serde(default)]
    pub plugin_policy: PluginPolicy,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            global_shortcut: DEFAULT_HOTKEY.to_owned(),
            plugin_policy: PluginPolicy::default(),
        }
    }
}

pub fn crest_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("crest")
}

pub fn crest_config_file() -> PathBuf {
    crest_config_dir().join("config.json")
}

pub fn load_app_config() -> AppConfig {
    let path = crest_config_file();
    if !path.exists() {
        if let Err(e) = ensure_config_skeleton(&path) {
            eprintln!("Crest: could not write default config {:?}: {}", path, e);
        }
        return AppConfig::default();
    }
    match std::fs::read_to_string(&path) {
        Ok(s) => match serde_json::from_str::<AppConfig>(&s) {
            Ok(c) => c,
            Err(e) => {
                eprintln!(
                    "Crest: invalid config at {:?} ({}); using defaults",
                    path, e
                );
                AppConfig::default()
            }
        },
        Err(e) => {
            eprintln!("Crest: could not read {:?} ({}); using defaults", path, e);
            AppConfig::default()
        }
    }
}

fn ensure_config_skeleton(path: &std::path::Path) -> std::io::Result<()> {
    if let Some(p) = path.parent() {
        std::fs::create_dir_all(p)?;
    }
    let cfg = AppConfig::default();
    std::fs::write(path, serde_json::to_vec_pretty(&cfg).unwrap_or_default())
}
