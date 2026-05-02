pub mod commands;
pub mod indexer;
pub mod hotkey;
pub mod window;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            hotkey::init(app.handle());
            window::setup_window_events(app.handle());
            indexer::apps::init();
            indexer::files::init();
            
            // Initialize currency rates in background
            tauri::async_runtime::spawn(commands::search::fetch_exchange_rates());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::search::search,
            commands::search::launch_app,
            commands::search::calculate,
            commands::search::search_files,
            commands::search::open_file,
            commands::search::search_web,
            commands::search::fetch_web_results,
            commands::system::get_system_actions,
            commands::extensions::run_extension,
            commands::extensions::get_clipboard_history
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
