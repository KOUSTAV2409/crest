use std::path::PathBuf;
use freedesktop_entry_parser::parse_entry;
use rusqlite::{Connection, Result as SqlResult};

pub fn init() {
    println!("Initializing App Indexer...");
    
    // Use system data directory to prevent Tauri watcher from triggering rebuilds
    let db_dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("/tmp")).join("crest");
    if let Err(e) = std::fs::create_dir_all(&db_dir) {
        eprintln!("App indexer: failed to create db dir {:?}: {}", db_dir, e);
        return;
    }
    let db_path = db_dir.join("crest_index.db");
    let conn = match Connection::open(db_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("App indexer: failed to open db: {}", e);
            return;
        }
    };
    
    if let Err(e) = setup_db(&conn) {
        eprintln!("App indexer: failed to setup DB: {}", e);
        return;
    }
    
    let app_dirs = vec![
        "/usr/share/applications",
        "/var/lib/flatpak/exports/share/applications",
    ];
    
    // Add ~/.local/share/applications if user home exists
    let mut dirs: Vec<PathBuf> = app_dirs.iter().map(PathBuf::from).collect();
    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join(".local/share/applications"));
        dirs.push(home.join(".local/share/flatpak/exports/share/applications"));
    }
    
    index_apps(&conn, dirs);
}

fn setup_db(conn: &Connection) -> SqlResult<()> {
    conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS apps USING fts5(
            id, name, exec, icon, comment, categories
        )",
        [],
    )?;
    Ok(())
}

fn index_apps(conn: &Connection, dirs: Vec<PathBuf>) {
    if let Err(e) = conn.execute("DELETE FROM apps", []) {
        eprintln!("App indexer: failed to clear apps table: {}", e);
    }
    
    for dir in dirs {
        if !dir.exists() { continue; }
        
        for entry in walkdir::WalkDir::new(&dir).into_iter().filter_map(|e| e.ok()) {
            if entry.path().extension().and_then(|e| e.to_str()) == Some("desktop") {
                if let Ok(parsed) = parse_entry(entry.path()) {
                    if let Some(section) = parsed.section("Desktop Entry") {
                        let name = section.attr("Name").first().map(|s| s.as_str()).unwrap_or("Unknown");
                        let exec = section.attr("Exec").first().map(|s| s.as_str()).unwrap_or("");
                        let icon = section.attr("Icon").first().map(|s| s.as_str()).unwrap_or("");
                        let comment = section.attr("Comment").first().map(|s| s.as_str()).unwrap_or("");
                        let categories = section.attr("Categories").first().map(|s| s.as_str()).unwrap_or("");
                        
                        let id = entry.path().to_string_lossy().to_string();
                        
                        let _ = conn.execute(
                            "INSERT INTO apps (id, name, exec, icon, comment, categories) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                            (id, name, exec, icon, comment, categories),
                        );
                    }
                }
            }
        }
    }
    println!("App indexing complete.");
}
