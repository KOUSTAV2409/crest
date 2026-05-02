use tauri::{WindowEvent, AppHandle, Manager};

pub fn setup_window_events(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        window.on_window_event(|event| {
            if let WindowEvent::Focused(focused) = event {
                if !focused {
                    // This is a common pattern to hide the window instead of closing it
                }
            }
        });
    }
}
