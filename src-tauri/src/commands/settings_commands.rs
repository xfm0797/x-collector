use serde::Serialize;
use std::collections::HashMap;
use tauri::State;

use crate::db::models::SettingsRepo;
use crate::AppState;

/// 读取全部设置
#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<HashMap<String, String>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    SettingsRepo::get_all(&conn).map_err(|e| e.to_string())
}

/// 批量保存设置
#[tauri::command]
pub fn save_settings(state: State<'_, AppState>, settings: HashMap<String, String>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    for (k, v) in settings {
        SettingsRepo::set(&conn, &k, &v).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 读取采集相关设置
#[tauri::command]
pub fn get_collect_settings(state: State<'_, AppState>) -> Result<CollectSettingsVo, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let get = |k: &str| SettingsRepo::get(&conn, k);
    Ok(CollectSettingsVo {
        timeout: get("collect_timeout").and_then(|v| v.parse().ok()).unwrap_or(30),
        concurrency: get("collect_concurrency").and_then(|v| v.parse().ok()).unwrap_or(3),
        interval_min: get("collect_interval_min").and_then(|v| v.parse().ok()).unwrap_or(2),
        interval_max: get("collect_interval_max").and_then(|v| v.parse().ok()).unwrap_or(5),
        retries: get("collect_retries").and_then(|v| v.parse().ok()).unwrap_or(3),
        user_agent: get("collect_user_agent").unwrap_or_default(),
        proxy_url: get("proxy_url").unwrap_or_default(),
    })
}

#[derive(Serialize)]
pub struct CollectSettingsVo {
    pub timeout: u64,
    pub concurrency: u64,
    pub interval_min: u64,
    pub interval_max: u64,
    pub retries: u32,
    pub user_agent: String,
    pub proxy_url: String,
}
