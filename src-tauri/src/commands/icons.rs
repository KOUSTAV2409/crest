//! Freedesktop icon resolution with minimal blocking work per lookup.
//! Paths are canonicalised only—no synchronous SVG/GPU-heavy rasterisation on the IPC path.

use freedesktop_icons::default_theme_gtk;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Theme list evaluated once per process (`gsettings` + env are not hot‑polled).
static THEMES_LOOKUP_ORDER: Lazy<Vec<String>> = Lazy::new(theme_lookup_chain);

/// Per‑stem memo (theme order is fixed → key is stable).
static STEM_ICON_CACHE: Lazy<Mutex<HashMap<String, Option<PathBuf>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[tauri::command]
pub async fn resolve_desktop_icon_path(name: String) -> Option<String> {
    let name = name.trim().to_owned();
    if name.is_empty() {
        return None;
    }
    tokio::task::spawn_blocking(move || resolve_desktop_icon_path_inner(&name))
        .await
        .ok()
        .flatten()
}

fn resolve_desktop_icon_path_inner(name: &str) -> Option<String> {
    if name.is_empty() {
        return None;
    }

    let trimmed = name.trim();
    let path_in = Path::new(trimmed);

    if path_in.is_absolute() && path_in.exists() && path_in.is_file() {
        return normalize_fast(path_in.to_path_buf());
    }

    let icon_leaf = Path::new(trimmed)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(trimmed);
    let themed_token = Path::new(icon_leaf)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(icon_leaf);

    if let Some(path) = themed_lookup_memo(themed_token) {
        return normalize_fast(path);
    }

    if trimmed.contains('/') && !path_in.is_absolute() {
        let rel = Path::new(trimmed.trim_start_matches("./"));
        if rel.is_absolute() && rel.exists() && rel.is_file() {
            return normalize_fast(rel.to_path_buf());
        }
        for base in icon_base_roots() {
            let cand = base.join(rel);
            if cand.exists() && cand.is_file() {
                return normalize_fast(cand);
            }
        }
        if let Some(hit) = try_pixmaps_path(icon_leaf, themed_token) {
            return normalize_fast(hit);
        }
    }

    try_pixmaps_path(icon_leaf, themed_token).and_then(normalize_fast)
}

fn theme_lookup_chain() -> Vec<String> {
    let mut out: Vec<String> = Vec::new();

    push_unique(&mut out, split_theme_token(&env_clean("ICON_THEME")));
    push_unique(&mut out, split_theme_token(&env_clean("GTK_ICON_THEME")));
    push_unique(&mut out, gtk_to_icon_hint(&env_clean("GTK_THEME")));
    push_unique(&mut out, filter_empty(default_theme_gtk()));

    for t in [
        "Adwaita",
        "Yaru",
        "elementary",
        "Papirus",
        "Papirus-Dark",
        "breeze-dark",
        "breeze",
        "hicolor",
    ] {
        push_unique_literal(&mut out, t);
    }

    out
}

fn themed_lookup_memo(stem: &str) -> Option<PathBuf> {
    let stem_key = stem.to_owned();

    if let Ok(guard) = STEM_ICON_CACHE.try_lock() {
        if let Some(hit) = guard.get(&stem_key) {
            return hit.clone();
        }
    }

    let resolved = themed_lookup_icon(stem);

    let _ = STEM_ICON_CACHE.try_lock().map(|mut guard| {
        if guard.len() > 1024 {
            guard.clear();
        }
        guard.insert(stem_key, resolved.clone());
    });

    resolved
}

/// Prefer bitmaps first across all themes/sizes, then SVG (symbolics) once.
fn themed_lookup_icon(stem: &str) -> Option<PathBuf> {
    let themes = THEMES_LOOKUP_ORDER.as_slice();

    for theme in themes {
        for size in ICON_SIZES {
            let hit = freedesktop_icons::lookup(stem)
                .with_theme(theme)
                .with_scale(1)
                .with_size(*size)
                .with_cache()
                .find();
            if hit.is_some() {
                return hit;
            }
        }
    }

    for theme in themes {
        for size in ICON_SIZES {
            let hit = freedesktop_icons::lookup(stem)
                .with_theme(theme)
                .with_scale(1)
                .with_size(*size)
                .with_cache()
                .force_svg()
                .find();
            if hit.is_some() {
                return hit;
            }
        }
    }

    None
}

/// Fewer probes than scanning every legacy size tier—latency wins over microscopic mismatches.
const ICON_SIZES: &[u16] = &[128, 96, 64, 48];

fn push_unique(acc: &mut Vec<String>, s: Option<String>) {
    if let Some(v) = s {
        let t = v.trim();
        if t.is_empty() {
            return;
        }
        if !acc.iter().any(|x| x == t) {
            acc.push(t.to_owned());
        }
    }
}

fn push_unique_literal(acc: &mut Vec<String>, t: &'static str) {
    push_unique(acc, Some(t.into()));
}

fn env_clean(var: &'static str) -> Option<String> {
    match std::env::var(var) {
        Ok(s) => {
            let t = s.trim().to_owned();
            (!t.is_empty()).then_some(t)
        }
        Err(_) => None,
    }
}

fn filter_empty(s: Option<String>) -> Option<String> {
    s.filter(|x| !x.trim().is_empty())
}

fn split_theme_token(raw: &Option<String>) -> Option<String> {
    Some(
        raw.as_ref()?
            .split(':')
            .next()
            .unwrap_or_default()
            .split('@')
            .next()
            .unwrap_or_default()
            .trim()
            .to_owned(),
    )
    .filter(|t| !t.is_empty())
}

fn gtk_to_icon_hint(raw: &Option<String>) -> Option<String> {
    split_theme_token(raw)
}

fn icon_base_roots() -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Some(h) = dirs::home_dir() {
        v.push(h.join(".local/share/icons"));
        v.push(h.join(".icons"));
    }
    if let Ok(xdg_data) = std::env::var("XDG_DATA_DIRS") {
        for part in xdg_data.split(':') {
            let p = Path::new(part.trim());
            if !p.as_os_str().is_empty() {
                v.push(p.join("icons"));
            }
        }
    }
    v.push(PathBuf::from("/usr/local/share/icons"));
    v.push(PathBuf::from("/usr/share/icons"));
    v.into_iter().filter(|p| p.is_dir()).collect()
}

fn try_pixmaps_path(simple_path_str: &str, stem_fallback: &str) -> Option<PathBuf> {
    let mut bases: Vec<PathBuf> = vec![PathBuf::from("/usr/share/pixmaps")];
    if let Some(h) = dirs::home_dir() {
        bases.push(h.join(".local/share/pixmaps"));
    }

    let exts = ["png", "svg", "webp", "jpg", "jpeg"];

    let try_names: &[&str] = if Path::new(simple_path_str)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ex| !ex.is_empty())
    {
        &[simple_path_str]
    } else {
        &[stem_fallback, simple_path_str]
    };

    for base in bases.into_iter() {
        if !base.is_dir() {
            continue;
        }
        for n in try_names.iter().copied() {
            let direct = base.join(n);
            if direct.is_file() {
                return Some(direct);
            }
            let name_stem = Path::new(n)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(stem_fallback);
            for ext in exts {
                let p = base.join(format!("{name_stem}.{ext}"));
                if p.is_file() {
                    return Some(p);
                }
            }
        }
    }

    None
}

fn normalize_fast(path: PathBuf) -> Option<String> {
    let path = fs::canonicalize(&path).unwrap_or(path);
    Some(path.to_string_lossy().into_owned())
}
