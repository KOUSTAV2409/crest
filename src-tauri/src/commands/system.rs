use serde::{Deserialize, Serialize};
use std::sync::atomic::Ordering;

#[derive(Serialize, Deserialize)]
pub struct SystemAction {
    pub id: String,
    pub title: String,
    pub icon: String,
}

/// Shown once in the palette when Linux global shortcut is unreliable or failed to register.
#[derive(Serialize, Deserialize)]
pub struct ShortcutSetupHint {
    pub show_banner: bool,
    pub headline: String,
    pub detail: String,
}

#[tauri::command]
pub fn get_shortcut_setup_hint() -> ShortcutSetupHint {
    let wayland = crate::hotkey::detect_wayland_session();
    let grab_ok = crate::hotkey::GLOBAL_HOTKEY_GRAB_OK.load(Ordering::Relaxed);
    let shortcut = crate::config::load_app_config().global_shortcut;

    // Wayland: in-app grabs often succeed on paper but Win+Meta never fires; grab failed on X11 too.
    let show_banner = wayland || !grab_ok;

    if !show_banner {
        return ShortcutSetupHint {
            show_banner: false,
            headline: String::new(),
            detail: String::new(),
        };
    }

    let headline = if wayland {
        "Windows/Super + Space may not work on Wayland".to_string()
    } else {
        "Could not grab the global shortcut".to_string()
    };

    let mut detail = String::new();
    if wayland {
        detail.push_str(
            "Your session uses Wayland. Crest still uses X11-style key grabs which usually do not receive your keys. ",
        );
        detail.push_str(&format!(
            "Use System Settings → Keyboard → Custom shortcuts and bind your key (e.g. {}) to command: crest — each run shows or hides Crest. ",
            shortcut
        ));
        detail.push_str("Or switch to an “Xorg” login session.");
    }
    if !grab_ok {
        if !detail.is_empty() {
            detail.push(' ');
        }
        detail.push_str(&format!(
            "Registration failed ({shortcut}). Try `alt+Space` or `ctrl+Alt+Space` in ~/.config/crest/config.json, or check ~/.local/share/crest/hotkey.log.",
        ));
    }

    ShortcutSetupHint {
        show_banner: true,
        headline,
        detail,
    }
}

#[tauri::command]
pub async fn get_system_actions() -> Result<Vec<SystemAction>, String> {
    Ok(vec![
        SystemAction { id: "lock".into(), title: "Lock Screen".into(), icon: "Lock".into() },
        SystemAction { id: "shutdown".into(), title: "Shutdown".into(), icon: "Power".into() },
        SystemAction { id: "quit".into(), title: "Quit Crest".into(), icon: "X".into() },
    ])
}
