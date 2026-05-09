use tauri::{WindowEvent, AppHandle, Manager};

pub fn setup_window_events(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        // Terax Wayland/Mutter Fix: Re-asserting false after the window is realized 
        // makes mutter respect it and drops the forced CSD title bar.
        #[cfg(target_os = "linux")]
        {
            let _ = window.set_decorations(false);
        }

        window.on_window_event(|event| {
            if let WindowEvent::Focused(focused) = event {
                if !focused {
                    // This is a common pattern to hide the window instead of closing it
                }
            }
        });
    }
}
