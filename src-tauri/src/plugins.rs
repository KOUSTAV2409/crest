use std::path::PathBuf;
use std::process::Command;
use serde::{Serialize, Deserialize};
use crate::commands::search::SearchResult;

#[derive(Serialize, Deserialize)]
pub struct Plugin {
    pub name: String,
    pub command: String,
    pub description: String,
    pub icon: String,
}

pub fn get_plugins_dir() -> PathBuf {
    dirs::config_dir().unwrap_or_else(|| PathBuf::from("/tmp")).join("crest").join("plugins")
}

pub fn list_plugins() -> Vec<Plugin> {
    let dir = get_plugins_dir();
    if !dir.exists() {
        let _ = std::fs::create_dir_all(&dir);
        return vec![];
    }

    let mut plugins = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let name = path.file_stem().unwrap().to_string_lossy().to_string();
                plugins.push(Plugin {
                    name: name.clone(),
                    command: path.to_string_lossy().to_string(),
                    description: format!("Extension: {}", name),
                    icon: "🔌".into(),
                });
            }
        }
    }
    plugins
}

pub fn run_plugin(command: &str, query: &str) -> Vec<SearchResult> {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", command, query]).output()
    } else {
        Command::new("sh").args(["-c", &format!("{} \"{}\"", command, query)]).output()
    };

    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        if let Ok(results) = serde_json::from_str::<Vec<SearchResult>>(&stdout) {
            return results;
        }
    }
    vec![]
}
