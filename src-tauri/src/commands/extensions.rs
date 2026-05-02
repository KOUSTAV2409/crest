use serde_json::Value;
use crate::commands::search::{SearchResult, ResultIcon, Action, Preview};
use rusqlite::Connection;
use std::path::PathBuf;

#[tauri::command]
pub async fn run_extension(id: String, _action: String, _args: Value) -> Result<Value, String> {
    let plugins = crate::plugins::list_plugins();
    let plugin_id = id.replace("plugin-", "");
    
    if let Some(plugin) = plugins.into_iter().find(|p| p.name == plugin_id) {
        println!("Running plugin: {}", plugin.command);
        let results = crate::plugins::run_plugin(&plugin.command, "");
        return Ok(serde_json::to_value(results).map_err(|e| e.to_string())?);
    }
    
    Err("Plugin not found".into())
}

#[tauri::command]
pub async fn get_clipboard_history() -> Result<Vec<SearchResult>, String> {
    let db_dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("/tmp")).join("crest");
    let db_path = db_dir.join("crest_index.db");
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare(
        "SELECT content, timestamp FROM clipboard_history ORDER BY timestamp DESC LIMIT 100"
    ).map_err(|e| e.to_string())?;

    let history_iter = stmt.query_map([], |row| {
        let content: String = row.get(0)?;
        let timestamp: String = row.get(1)?;
        
        let display_title = if content.len() > 60 {
            format!("{}...", &content[..60])
        } else {
            content.clone()
        };

        Ok(SearchResult {
            id: format!("clip-{}", timestamp),
            title: display_title,
            subtitle: format!("Copied at {}", timestamp),
            icon: ResultIcon { kind: "emoji".into(), value: "📋".into() },
            category: "Clipboard".into(),
            score: 0.1,
            actions: vec![
                Action { id: "copy".into(), title: "Copy to Clipboard".into(), shortcut: Some("↵".into()) },
                Action { id: "open_url".into(), title: "Open in Browser".into(), shortcut: Some("⌘↵".into()) }
            ],
            preview: Some(Preview {
                title: "Clipboard History".into(),
                subtitle: Some(timestamp),
                description: Some(content),
            })
        })
    }).map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for item in history_iter {
        if let Ok(res) = item {
            results.push(res);
        }
    }

    Ok(results)
}
