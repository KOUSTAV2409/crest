use rusqlite::Connection;
use std::path::PathBuf;
use walkdir::WalkDir;

pub fn init() {
    println!("Initializing Files Indexer...");
    
    // Spawn background task so we don't block app startup
    tauri::async_runtime::spawn(async move {
        if let Err(e) = index_files() {
            eprintln!("File indexing error: {}", e);
        }
    });
}

fn index_files() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let db_dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("/tmp")).join("crest");
    let db_path = db_dir.join("crest_index.db");
    let mut conn = Connection::open(db_path)?;
    
    // Create files table if it doesn't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS files (
            id TEXT PRIMARY KEY,
            name TEXT,
            path TEXT,
            extension TEXT
        )",
        [],
    )?;
    
    // Clear old index for a fresh start (for simplicity initially)
    conn.execute("DELETE FROM files", [])?;
    
    let tx = conn.transaction()?;
    
    let mut count = 0;
    
    let base_dirs = vec![
        dirs::document_dir(),
        dirs::download_dir(),
        dirs::desktop_dir(),
    ];
    
    for dir in base_dirs.into_iter().flatten() {
        for entry in WalkDir::new(&dir).into_iter().filter_entry(|e| !is_hidden_or_excluded(e)) {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            
            if entry.file_type().is_file() {
                let path = entry.path();
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                let ext = path.extension().unwrap_or_default().to_string_lossy();
                let full_path = path.to_string_lossy();
                
                tx.execute(
                    "INSERT INTO files (id, name, path, extension) VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![full_path, name, full_path, ext],
                )?;
                count += 1;
            }
        }
    }
    
    tx.commit()?;
    
    println!("File indexing complete. Indexed {} files.", count);
    
    Ok(())
}

fn is_hidden_or_excluded(entry: &walkdir::DirEntry) -> bool {
    let name = entry.file_name().to_string_lossy();
    if name.starts_with('.') && name != "." && name != ".." {
        return true;
    }
    let excluded = ["node_modules", "target", "build", "dist", "vendor", "__pycache__"];
    if excluded.contains(&name.as_ref()) {
        return true;
    }
    false
}
