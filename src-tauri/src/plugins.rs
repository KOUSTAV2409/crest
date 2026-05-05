use serde::Deserialize;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde::Serialize;

static LOGGED_MISSING_MANIFEST: AtomicBool = AtomicBool::new(false);
static LOGGED_OPEN_POLICY_WARNING: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Serialize)]
pub struct Plugin {
    pub name: String,
    pub command: String,
    pub description: String,
    pub icon: String,
}

#[derive(Debug, Deserialize)]
struct PluginManifestFile {
    version: u32,
    entries: Vec<PluginManifestEntry>,
}

#[derive(Debug, Deserialize)]
struct PluginManifestEntry {
    /// Relative path inside the plugins directory (e.g. `notes.sh`).
    path: String,
    #[serde(default = "default_true")]
    enabled: bool,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    icon: Option<String>,
}

fn default_true() -> bool {
    true
}

pub fn get_plugins_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("crest")
        .join("plugins")
}

fn manifest_path() -> PathBuf {
    get_plugins_dir().join("manifest.json")
}

fn canonical_under_root(root: &Path, relative: &str) -> Option<PathBuf> {
    let _ = std::fs::create_dir_all(root);
    let root = root.canonicalize().ok()?;
    let candidate =
        PathBuf::from(relative.trim().trim_start_matches(['/', '\\']));
    if candidate
        .components()
        .any(|c| matches!(c, Component::ParentDir))
    {
        return None;
    }
    let joined = root.join(candidate);
    let joined = joined.canonicalize().ok()?;
    if joined.starts_with(&root) {
        Some(joined)
    } else {
        None
    }
}

fn load_manifest() -> Result<PluginManifestFile, String> {
    let path = manifest_path();
    let raw = std::fs::read_to_string(&path).map_err(|e| format!("read manifest: {}", e))?;
    serde_json::from_str(&raw).map_err(|e| format!("{}", e))
}

pub fn list_plugins_manifest_only() -> Vec<Plugin> {
    let plugins_dir = get_plugins_dir();
    if !manifest_path().exists() {
        if LOGGED_MISSING_MANIFEST
            .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
            .is_ok()
        {
            eprintln!(
                "Crest plugins: no manifest.json at {:?} — extensions disabled until you add one in the same folder. See configs/plugins.manifest.example.json in the Crest repo (use version 1 and an empty entries list if you don't use plugins).",
                manifest_path()
            );
        }
        return vec![];
    }

    match load_manifest() {
        Ok(m) => {
            if m.version != 1 {
                eprintln!("Crest plugins: unsupported manifest version {}", m.version);
                return vec![];
            }
            m.entries
                .into_iter()
                .filter(|e| e.enabled)
                .filter_map(|e| {
                    let trimmed = e.path.trim();
                    let abs = canonical_under_root(&plugins_dir, trimmed)?;
                    if !abs.is_file() {
                        eprintln!(
                            "Crest plugins: manifest entry {:?} is not a file; skipping.",
                            trimmed
                        );
                        return None;
                    }
                    let fname = Path::new(trimmed)
                        .file_stem()
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_else(|| trimmed.to_string());

                    Some(Plugin {
                        name: fname.clone(),
                        command: abs.to_string_lossy().into_owned(),
                        description: e
                            .description
                            .unwrap_or_else(|| format!("Extension: {}", fname)),
                        icon: e.icon.unwrap_or_else(|| "🔌".into()),
                    })
                })
                .collect()
        }
        Err(e) => {
            eprintln!("Crest plugins: invalid manifest: {}", e);
            vec![]
        }
    }
}

fn list_plugins_open_policy() -> Vec<Plugin> {
    let dir = get_plugins_dir();
    if !dir.exists() {
        let _ = std::fs::create_dir_all(&dir);
        return vec![];
    }

    let root_ok = dir.canonicalize().ok();

    let mut plugins = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.file_name().and_then(|n| n.to_str()) == Some("manifest.json") {
                continue;
            }
            if !path.is_file() {
                continue;
            }

            let canonical = match path.canonicalize() {
                Ok(p) => p,
                Err(_) => continue,
            };

            if let Some(ref root) = root_ok {
                if !canonical.starts_with(root) {
                    continue;
                }
            }

            let name = path.file_stem().unwrap().to_string_lossy().to_string();
            plugins.push(Plugin {
                name: name.clone(),
                command: canonical.to_string_lossy().into_owned(),
                description: format!("Extension: {}", name),
                icon: "🔌".into(),
            });
        }
    }
    plugins
}

pub fn list_plugins() -> Vec<Plugin> {
    match crate::config::load_app_config().plugin_policy {
        crate::config::PluginPolicy::Manifest => list_plugins_manifest_only(),
        crate::config::PluginPolicy::Open => {
            if LOGGED_OPEN_POLICY_WARNING
                .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                eprintln!(
                    "Crest plugins: plugin_policy=open trusts every executable/script in {:?}. Prefer \"manifest\" in config.",
                    get_plugins_dir()
                );
            }
            list_plugins_open_policy()
        }
    }
}

/// Run plugin script/binary with `query` as a single argv argument (no arbitrary shell interpretation).
pub fn run_plugin(command_abs: &str, query: &str) -> Vec<crate::commands::search::SearchResult> {
    let cmd_path = Path::new(command_abs);

    #[cfg(not(target_os = "windows"))]
    {
        let Ok(meta) = std::fs::metadata(cmd_path) else {
            return vec![];
        };
        if !meta.is_file() {
            return vec![];
        }

        let is_executable = (meta.permissions().mode() & 0o111) != 0;
        let mut cmd = if command_abs.ends_with(".sh") || !is_executable {
            let mut c = Command::new("/bin/sh");
            c.arg(command_abs);
            c
        } else {
            Command::new(command_abs)
        };
        cmd.arg(query)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let output = run_with_timeout(cmd, Duration::from_secs(2));
        return parse_plugin_output_limited(output, 1_000_000);
    }

    #[cfg(target_os = "windows")]
    {
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", command_abs, query])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        parse_plugin_output_limited(run_with_timeout(cmd, Duration::from_secs(2)), 1_000_000)
    }
}

fn run_with_timeout(
    mut command: Command,
    timeout: Duration,
) -> Result<std::process::Output, std::io::Error> {
    // Use a join timeout so plugins can’t hang the launcher.
    let child = command.spawn()?;
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let out = child.wait_with_output();
        let _ = tx.send(out);
    });
    match rx.recv_timeout(timeout) {
        Ok(out) => out,
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => Err(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            "plugin execution timed out",
        )),
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "plugin execution channel disconnected",
        )),
    }
}

fn parse_plugin_output_limited(
    output: Result<std::process::Output, std::io::Error>,
    max_stdout_bytes: usize,
) -> Vec<crate::commands::search::SearchResult> {
    let Ok(out) = output else {
        return vec![];
    };
    if out.stdout.len() > max_stdout_bytes {
        return vec![];
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    serde_json::from_str::<Vec<crate::commands::search::SearchResult>>(&stdout).unwrap_or_default()
}
