use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager,
    hotkey::{Code, HotKey, Modifiers},
};
use once_cell::sync::Lazy;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Manager};

/// Set to `true` only after [`GlobalHotKeyManager::register`] succeeds (startup).
pub static GLOBAL_HOTKEY_GRAB_OK: AtomicBool = AtomicBool::new(false);

// Keep the manager globally static to ensure it never drops.
static MANAGER: Lazy<Result<GlobalHotKeyManager, String>> =
    Lazy::new(|| GlobalHotKeyManager::new().map_err(|e| format!("{}", e)));

fn append_hotkey_log(message: &str) {
    let dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("crest");
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    let path = dir.join("hotkey.log");
    let ts = format!("{:?}", std::time::SystemTime::now());
    if let Ok(mut f) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let _ = writeln!(f, "[{}] {}", ts, message);
    }
}

/// True when `WAYLAND_DISPLAY` is set (typical GNOME/KDE defaults).
pub fn detect_wayland_session() -> bool {
    std::env::var("WAYLAND_DISPLAY")
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false)
}

pub fn init(app: &AppHandle) {
    let manager = match &*MANAGER {
        Ok(m) => m,
        Err(e) => {
            let msg = format!(
                "Failed to initialize GlobalHotKeyManager: {}. Hotkeys disabled; use a system shortcut to run `crest` (single-instance toggle).",
                e
            );
            eprintln!("Crest: {}", msg);
            append_hotkey_log(&msg);
            return;
        }
    };

    let cfg = crate::config::load_app_config();
    let hotkey_raw = cfg.global_shortcut.trim();
    let hotkey: HotKey = match hotkey_raw.parse::<HotKey>() {
        Ok(hk) => hk,
        Err(e) => {
            let msg = format!(
                "Invalid global_shortcut {:?} in config: {}. Falling back to super+Space.",
                hotkey_raw, e
            );
            eprintln!("Crest: {}", msg);
            append_hotkey_log(&msg);
            HotKey::new(Some(Modifiers::SUPER), Code::Space)
        }
    };

    match manager.register(hotkey) {
        Ok(()) => {
            GLOBAL_HOTKEY_GRAB_OK.store(true, Ordering::Relaxed);
        }
        Err(e) => {
            let msg = format!(
                "Failed to register global shortcut {:?} -> {}: {}. \
If you use Wayland (default on many distros), X11-style global grabs often do not work. \
Fix: open System Settings → Keyboard → Custom Shortcuts and assign Super+Space (or another key) to run the `crest` command; a second press will hide the window. \
You can also try `alt+Space` in ~/.config/crest/config.json. \
This message is saved to your data directory as crest/hotkey.log.",
                hotkey_raw,
                hotkey,
                e
            );
            eprintln!("Crest: {}", msg);
            append_hotkey_log(&msg);
        }
    }

    let app_handle = app.clone();

    std::thread::spawn(move || {
        let receiver = GlobalHotKeyEvent::receiver();
        loop {
            if let Ok(event) = receiver.recv() {
                if event.id == hotkey.id() {
                    let handle = app_handle.clone();
                    let handle_clone = handle.clone();
                    let _ = handle.run_on_main_thread(move || {
                        if let Some(window) = handle_clone.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    });
                }
            }
        }
    });
}
