use std::path::PathBuf;
use rusqlite::Connection;
use arboard::Clipboard;
use std::time::Duration;
use std::thread;

pub fn init() {
    println!("Initializing Clipboard Listener...");
    
    let db_dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("/tmp")).join("crest");
    std::fs::create_dir_all(&db_dir).unwrap();
    let db_path = db_dir.join("crest_index.db");
    
    // Setup table
    {
        let conn = Connection::open(&db_path).unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS clipboard_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content TEXT UNIQUE NOT NULL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        ).expect("Failed to create clipboard table");
    }

    // Start background thread
    thread::spawn(move || {
        let mut last_content = String::new();
        let mut clipboard = Clipboard::new().expect("Failed to initialize clipboard");
        
        loop {
            if let Ok(content) = clipboard.get_text() {
                let trimmed = content.trim();
                if !trimmed.is_empty() && trimmed != last_content {
                    last_content = trimmed.to_string();
                    save_to_db(&db_path, &last_content);
                }
            }
            thread::sleep(Duration::from_millis(1000));
        }
    });
}

fn save_to_db(db_path: &PathBuf, content: &str) {
    if let Ok(conn) = Connection::open(db_path) {
        // Insert or ignore if exists, then update timestamp if exists
        let _ = conn.execute(
            "INSERT INTO clipboard_history (content) VALUES (?1)
             ON CONFLICT(content) DO UPDATE SET timestamp = CURRENT_TIMESTAMP",
            [content],
        );

        // Keep only top 1000
        let _ = conn.execute(
            "DELETE FROM clipboard_history WHERE id NOT IN (
                SELECT id FROM clipboard_history ORDER BY timestamp DESC LIMIT 1000
            )",
            [],
        );
    }
}
