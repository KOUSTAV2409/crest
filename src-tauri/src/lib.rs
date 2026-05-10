pub mod commands;
pub mod config;
pub mod indexer;
pub mod hotkey;
pub mod window;
pub mod clipboard;
pub mod plugins;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use tauri::Manager;

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let visible = w.is_visible().unwrap_or(false);
                if visible {
                    let _ = w.hide();
                } else {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let start = std::time::Instant::now();
            println!("[STARTUP] Setup hook started");

            window::setup_window_events(app.handle());
            println!("[STARTUP] setup_window_events took {:?}", start.elapsed());
            
            // Spawn background initializers to prevent blocking startup
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let s = std::time::Instant::now();
                hotkey::init(&handle);
                println!("[STARTUP_BG] hotkey::init took {:?}", s.elapsed());
            });
            tauri::async_runtime::spawn(async move {
                let s = std::time::Instant::now();
                indexer::apps::init();
                println!("[STARTUP_BG] indexer::apps::init took {:?}", s.elapsed());
            });
            tauri::async_runtime::spawn(async move {
                let s = std::time::Instant::now();
                indexer::files::init();
                println!("[STARTUP_BG] indexer::files::init took {:?}", s.elapsed());
            });
            tauri::async_runtime::spawn(async move {
                let s = std::time::Instant::now();
                clipboard::init();
                println!("[STARTUP_BG] clipboard::init took {:?}", s.elapsed());
            });
            
            // Initialize currency rates in background
            tauri::async_runtime::spawn(commands::search::fetch_exchange_rates());
            
            println!("[STARTUP] Setup hook completed in {:?}", start.elapsed());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::search::search,
            commands::search::launch_app,
            commands::search::calculate,
            commands::search::search_files,
            commands::search::open_file,
            commands::search::open_url,
            commands::search::search_web,
            commands::search::fetch_web_results,
            commands::search::quit_app,
            commands::system::get_system_actions,
            commands::system::get_shortcut_setup_hint,
            commands::extensions::run_extension,
            commands::extensions::get_clipboard_history,
            commands::icons::resolve_desktop_icon_path,
            commands::pty::spawn_pty,
            commands::pty::write_pty,
            commands::pty::resize_pty,
            commands::pty::kill_pty
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
