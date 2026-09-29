//! 可视化选择器命令：页面 DOM 检查 + 选择器测试

use tauri::State;

use crate::collector::engine::Fetcher;
use crate::collector::selector_inspector;
use crate::AppState;

use super::collect_commands::load_fetch_settings;

/// 抓取页面并返回 DOM 结构树（用于可视化选择器）
#[tauri::command]
pub async fn inspect_page(
    state: State<'_, AppState>,
    url: String,
) -> Result<selector_inspector::PageInspect, String> {
    if !url.starts_with("http") {
        return Err("请输入以 http/https 开头的有效 URL".to_string());
    }
    let fetch_settings = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        load_fetch_settings(&conn)
    };
    let fetcher = Fetcher::new(fetch_settings)?;
    selector_inspector::inspect_page(&fetcher, &url).await
}

/// 在页面上测试 CSS 选择器，返回匹配数量与文本预览
#[tauri::command]
pub async fn test_selector(
    state: State<'_, AppState>,
    url: String,
    css: String,
) -> Result<selector_inspector::SelectorTestResult, String> {
    if !url.starts_with("http") {
        return Err("请输入以 http/https 开头的有效 URL".to_string());
    }
    if css.trim().is_empty() {
        return Err("请输入 CSS 选择器".to_string());
    }
    let fetch_settings = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        load_fetch_settings(&conn)
    };
    let fetcher = Fetcher::new(fetch_settings)?;
    selector_inspector::test_selector(&fetcher, &url, css.trim()).await
}
