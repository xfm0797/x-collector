use std::collections::HashMap;
use tauri::State;

use crate::app::config::CollectSettings;
use crate::db::models::SettingsRepo;
use crate::AppState;

/// 读取全部设置
#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<HashMap<String, String>, String> {
    let conn = state.conn()?;
    SettingsRepo::get_all(&conn).map_err(|e| e.to_string())
}

/// 批量保存设置
#[tauri::command]
pub fn save_settings(state: State<'_, AppState>, settings: HashMap<String, String>) -> Result<(), String> {
    let conn = state.conn()?;
    for (k, v) in settings {
        SettingsRepo::set(&conn, &k, &v).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 读取采集相关设置
#[tauri::command]
pub fn get_collect_settings(state: State<'_, AppState>) -> Result<CollectSettings, String> {
    let conn = state.conn()?;
    Ok(CollectSettings::load(&conn))
}
