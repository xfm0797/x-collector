use tauri::State;

use crate::db::models::{CollectSource, CollectSourceInput, SourceRepo};
use crate::AppState;

/// 采集源列表
#[tauri::command]
pub fn list_sources(state: State<'_, AppState>) -> Result<Vec<CollectSource>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    SourceRepo::list(&conn).map_err(|e| e.to_string())
}

/// 新建采集源
#[tauri::command]
pub fn create_source(state: State<'_, AppState>, input: CollectSourceInput) -> Result<CollectSource, String> {
    if input.name.trim().is_empty() || input.url.trim().is_empty() {
        return Err("名称与 URL 不能为空".to_string());
    }
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    SourceRepo::create(&conn, &input).map_err(|e| e.to_string())
}

/// 更新采集源
#[tauri::command]
pub fn update_source(state: State<'_, AppState>, id: i64, input: CollectSourceInput) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    SourceRepo::update(&conn, id, &input).map_err(|e| e.to_string())
}

/// 删除采集源（单条/批量）
#[tauri::command]
pub fn delete_sources(state: State<'_, AppState>, ids: Vec<i64>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    SourceRepo::delete(&conn, &ids).map_err(|e| e.to_string())
}
