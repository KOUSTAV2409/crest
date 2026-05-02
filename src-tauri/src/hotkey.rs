use global_hotkey::{GlobalHotKeyManager, hotkey::{HotKey, Modifiers, Code}, GlobalHotKeyEvent};
use tauri::{AppHandle, Manager};

pub fn init(app: &AppHandle) {
    let manager = GlobalHotKeyManager::new().unwrap();
    let hotkey = HotKey::new(Some(Modifiers::CONTROL), Code::Space);
    
    manager.register(hotkey).unwrap();
    
    let app_handle = app.clone();
    
    std::thread::spawn(move || {
        // Keep the manager alive for the lifetime of the application
        let _keep_alive = manager;
        
        let receiver = GlobalHotKeyEvent::receiver();
        loop {
            if let Ok(event) = receiver.recv() {
                if event.id == hotkey.id() {
                    if let Some(window) = app_handle.get_webview_window("main") {
                        if window.is_visible().unwrap_or(false) {
                            let _ = window.hide();
                        } else {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                }
            }
        }
    });
}
