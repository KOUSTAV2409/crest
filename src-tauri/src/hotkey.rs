use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager,
    hotkey::{HotKey, Modifiers, Code},
};
use once_cell::sync::Lazy;
use tauri::{AppHandle, Manager};

// Keep the manager globally static to ensure it never drops.
static MANAGER: Lazy<GlobalHotKeyManager> = Lazy::new(|| GlobalHotKeyManager::new().unwrap());

pub fn init(app: &AppHandle) {
    let cfg = crate::config::load_app_config();
    let hotkey_raw = cfg.global_shortcut.trim();
    let hotkey: HotKey = match hotkey_raw.parse::<HotKey>() {
        Ok(hk) => hk,
        Err(e) => {
            eprintln!(
                "Crest hotkey {:?} invalid ({}); using super+Space",
                hotkey_raw, e
            );
            HotKey::new(Some(Modifiers::SUPER), Code::Space)
        }
    };

    if let Err(e) = MANAGER.register(hotkey) {
        eprintln!("Failed to register hotkey: {}", e);
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
