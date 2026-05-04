use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct SystemAction {
    pub id: String,
    pub title: String,
    pub icon: String,
}

#[tauri::command]
pub async fn get_system_actions() -> Result<Vec<SystemAction>, String> {
    Ok(vec![
        SystemAction { id: "lock".into(), title: "Lock Screen".into(), icon: "Lock".into() },
        SystemAction { id: "shutdown".into(), title: "Shutdown".into(), icon: "Power".into() },
        SystemAction { id: "quit".into(), title: "Quit Crest".into(), icon: "X".into() },
    ])
}
