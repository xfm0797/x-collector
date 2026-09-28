use tauri::State;

use crate::db::models::{KeywordRepo, KeywordTask, KeywordTaskInput, LogRepo};
use crate::AppState;

/// 关键词任务列表
#[tauri::command]
pub fn list_keyword_tasks(state: State<'_, AppState>) -> Result<Vec<KeywordTask>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    KeywordRepo::list(&conn).map_err(|e| e.to_string())
}

/// 新建关键词任务
#[tauri::command]
pub fn create_keyword_task(state: State<'_, AppState>, input: KeywordTaskInput) -> Result<KeywordTask, String> {
    if input.keyword.trim().is_empty() {
        return Err("关键词不能为空".to_string());
    }
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    KeywordRepo::create(&conn, &input).map_err(|e| e.to_string())
}

/// 更新关键词任务
#[tauri::command]
pub fn update_keyword_task(state: State<'_, AppState>, id: i64, input: KeywordTaskInput) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    KeywordRepo::update(&conn, id, &input).map_err(|e| e.to_string())
}

/// 删除关键词任务（单条/批量）
#[tauri::command]
pub fn delete_keyword_tasks(state: State<'_, AppState>, ids: Vec<i64>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    KeywordRepo::delete(&conn, &ids).map_err(|e| e.to_string())
}

/// 采集日志列表
#[tauri::command]
pub fn list_collect_logs(state: State<'_, AppState>, limit: Option<i64>) -> Result<Vec<crate::db::models::CollectLog>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    LogRepo::list(&conn, limit.unwrap_or(100)).map_err(|e| e.to_string())
}
