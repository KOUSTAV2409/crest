use global_hotkey::{GlobalHotKeyManager, hotkey::{HotKey, Modifiers, Code}, GlobalHotKeyEvent};
use tauri::{AppHandle, Manager};
use once_cell::sync::Lazy;

// Keep the manager globally static to ensure it never drops
static MANAGER: Lazy<GlobalHotKeyManager> = Lazy::new(|| GlobalHotKeyManager::new().unwrap());

pub fn init(app: &AppHandle) {
    let hotkey = HotKey::new(Some(Modifiers::SUPER), Code::Space);
    
    // Register the hotkey
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
                    // Move window interaction to the main thread
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
