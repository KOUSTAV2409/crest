use notify::{Config, RecursiveMode, RecommendedWatcher, Watcher};
use rusqlite::Connection;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::time::Duration;
use walkdir::WalkDir;

pub fn init() {
    tauri::async_runtime::spawn(async move {
        if let Err(e) = sync_file_index_blocking() {
            eprintln!("File indexing error: {}", e);
        }

        spawn_debounced_watcher(base_index_dirs());
    });
}

pub fn base_index_dirs() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = vec![];
    for d in [dirs::document_dir(), dirs::download_dir(), dirs::desktop_dir()] {
        if let Some(p) = d {
            dirs.push(p);
        }
    }
    dirs
}

fn sync_file_index_blocking() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let db_dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("/tmp")).join("crest");
    std::fs::create_dir_all(&db_dir)?;
    let db_path = db_dir.join("crest_index.db");
    let conn = Connection::open(db_path)?;
    ensure_files_schema(&conn)?;
    conn.execute_batch(
        r#"
        PRAGMA journal_mode=WAL;
        PRAGMA foreign_keys=ON;
        "#,
    )?;
    sync_files_with_generation(&conn)?;
    Ok(())
}

fn ensure_files_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS files (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            path TEXT NOT NULL,
            extension TEXT NOT NULL,
            mtime_ms INTEGER NOT NULL DEFAULT 0,
            size_bytes INTEGER NOT NULL DEFAULT 0,
            gen INTEGER NOT NULL DEFAULT 0
        )",
        [],
    )?;

    for col in ["mtime_ms", "size_bytes", "gen"] {
        if !column_exists(conn, col)? {
            conn.execute(
                &format!("ALTER TABLE files ADD COLUMN {col} INTEGER NOT NULL DEFAULT 0"),
                [],
            )?;
        }
    }
    Ok(())
}

fn column_exists(conn: &Connection, column: &str) -> rusqlite::Result<bool> {
    let mut stmt = conn.prepare("PRAGMA table_info(files)")?;
    let names = stmt.query_map([], |row| row.get::<_, String>(1))?;
    for n in names {
        if n? == column {
            return Ok(true);
        }
    }
    Ok(false)
}

fn next_generation(conn: &Connection) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COALESCE(MAX(gen), 0) + 1 FROM files",
        [],
        |row| row.get(0),
    )
}

fn file_mtime_ms(meta: &std::fs::Metadata) -> u64 {
    meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn stable_id_for_path(full_path_str: &str) -> String {
    if full_path_str.len() <= 2048 {
        full_path_str.to_string()
    } else {
        let mut h = DefaultHasher::new();
        full_path_str.hash(&mut h);
        format!("{:016x}", h.finish())
    }
}

pub fn sync_files_with_generation(conn: &Connection) -> rusqlite::Result<()> {
    let gen = next_generation(conn)?;
    let tx = conn.unchecked_transaction()?;

    for dir in base_index_dirs().into_iter() {
        if !dir.exists() {
            continue;
        }
        for entry in WalkDir::new(&dir).into_iter().filter_entry(|e| !is_hidden_or_excluded(e)) {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path();
            let Ok(meta) = std::fs::metadata(path) else {
                continue;
            };
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let ext = path
                .extension()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let full_path = path.to_string_lossy().to_string();
            let id = stable_id_for_path(&full_path);
            let mtime_ms = file_mtime_ms(&meta) as i64;
            let size_bytes = meta.len() as i64;

            let unchanged: bool = tx
                .query_row(
                    "SELECT mtime_ms, size_bytes FROM files WHERE id = ?1",
                    [&id],
                    |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
                )
                .map(|(om, os)| om == mtime_ms && os == size_bytes)
                .unwrap_or(false);

            if unchanged {
                tx.execute(
                    "UPDATE files SET gen = ?1 WHERE id = ?2",
                    rusqlite::params![gen, id],
                )?;
            } else {
                tx.execute(
                    "INSERT INTO files (id, name, path, extension, mtime_ms, size_bytes, gen)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                     ON CONFLICT(id) DO UPDATE SET
                       name = excluded.name,
                       path = excluded.path,
                       extension = excluded.extension,
                       mtime_ms = excluded.mtime_ms,
                       size_bytes = excluded.size_bytes,
                       gen = excluded.gen",
                    rusqlite::params![id, name, full_path, ext, mtime_ms, size_bytes, gen],
                )?;
            }
        }
    }

    tx.execute("DELETE FROM files WHERE gen != ?1", [gen])?;
    tx.commit()
}

fn is_hidden_or_excluded(entry: &walkdir::DirEntry) -> bool {
    let name = entry.file_name().to_string_lossy();
    if name.starts_with('.') && name != "." && name != ".." {
        return true;
    }
    let excluded = ["node_modules", "target", "build", "dist", "vendor", "__pycache__"];
    excluded.contains(&name.as_ref())
}

fn spawn_debounced_watcher(paths: Vec<PathBuf>) {
    let (notify_tx, notify_rx) = std::sync::mpsc::channel();
    let mut watcher = match RecommendedWatcher::new(notify_tx, Config::default()) {
        Ok(w) => w,
        Err(e) => {
            eprintln!(
                "Crest: file watcher unavailable ({}); incremental updates disabled",
                e
            );
            return;
        }
    };

    let mut watched = false;
    for p in &paths {
        if p.exists() && watcher.watch(p, RecursiveMode::Recursive).is_ok() {
            watched = true;
        }
    }

    if !watched {
        return;
    }

    std::thread::spawn(move || loop {
        if notify_rx.recv().is_err() {
            break;
        }
        std::thread::sleep(Duration::from_millis(750));
        while notify_rx.recv_timeout(Duration::from_millis(50)).is_ok() {}

        if let Err(e) = sync_file_index_blocking() {
            eprintln!("Crest incremental file reindex failed: {}", e);
        }
    });
}
