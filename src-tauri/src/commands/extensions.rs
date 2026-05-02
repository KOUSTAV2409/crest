use serde_json::Value;

#[tauri::command]
pub async fn run_extension(id: String, action: String, args: Value) -> Result<Value, String> {
    println!("Running extension: {} - action: {} - args: {:?}", id, action, args);
    Ok(serde_json::json!({ "status": "success" }))
}

#[tauri::command]
pub async fn get_clipboard_history() -> Result<Vec<Value>, String> {
    Ok(vec![])
}
