use serde::Serialize;
use tauri::State;

use crate::collector::rewriter::{self, RewriteOptions};
use crate::collector::synonym_dict;
use crate::db::models::{ArticleRepo, SettingsRepo};
use crate::AppState;

/// 获取伪原创选项
#[tauri::command]
pub fn get_rewrite_options(state: State<'_, AppState>) -> Result<RewriteOptions, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    Ok(load_options(&conn))
}

/// 保存伪原创选项
#[tauri::command]
pub fn save_rewrite_options(state: State<'_, AppState>, options: RewriteOptions) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let pairs = [
        ("rewrite_enabled", if options.enabled { "true" } else { "false" }.to_string()),
        ("rewrite_intensity", options.intensity.clone()),
        ("rewrite_synonym_ratio", options.synonym_ratio.to_string()),
        ("rewrite_sentence_ratio", options.sentence_ratio.to_string()),
        ("rewrite_paragraph_shuffle", if options.paragraph_shuffle { "true" } else { "false" }.to_string()),
        ("rewrite_rewrite_ends", if options.rewrite_ends { "true" } else { "false" }.to_string()),
        ("rewrite_keywords", options.keywords.join(",")),
    ];
    for (k, v) in pairs {
        SettingsRepo::set(&conn, k, &v).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 预览伪原创效果（不落库）
#[tauri::command]
pub fn preview_rewrite(state: State<'_, AppState>, article_id: i64) -> Result<RewritePreview, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let article = ArticleRepo::get(&conn, article_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "文章不存在".to_string())?;
    let original = article.content_md.clone().unwrap_or_default();

    let options = load_options(&conn);
    let custom = load_custom_dict(&conn);
    // 预览时强制启用
    let mut preview_opts = options.clone();
    preview_opts.enabled = true;
    let rewritten = rewriter::rewrite_markdown(&original, &preview_opts, &custom);

    Ok(RewritePreview {
        original,
        rewritten,
        options,
    })
}

#[derive(Serialize)]
pub struct RewritePreview {
    pub original: String,
    pub rewritten: String,
    pub options: RewriteOptions,
}

/// 应用伪原创到文章
#[tauri::command]
pub fn apply_rewrite(state: State<'_, AppState>, article_id: i64) -> Result<crate::db::models::Article, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let article = ArticleRepo::get(&conn, article_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "文章不存在".to_string())?;
    let original = article.content_md.clone().unwrap_or_default();

    let options = load_options(&conn);
    let custom = load_custom_dict(&conn);
    let rewritten = rewriter::rewrite_markdown(&original, &options, &custom);
    ArticleRepo::update_content_md(&conn, article_id, &rewritten).map_err(|e| e.to_string())?;

    ArticleRepo::get(&conn, article_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "文章不存在".to_string())
}

/// 导入自定义同义词词库
#[tauri::command]
pub fn import_custom_dict(state: State<'_, AppState>, content: String) -> Result<OpResult, String> {
    let groups = synonym_dict::parse_custom_dict(&content)?;
    if groups.is_empty() {
        return Ok(OpResult {
            success: false,
            message: "词库为空或格式不正确：需要至少一组包含 2 个词的同义词组".to_string(),
        });
    }
    let count = groups.len();
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    SettingsRepo::set(&conn, "rewrite_custom_dict", &content).map_err(|e| e.to_string())?;
    Ok(OpResult {
        success: true,
        message: format!("导入成功：新增 {} 组同义词", count),
    })
}

#[derive(Serialize)]
pub struct OpResult {
    pub success: bool,
    pub message: String,
}

/// 重置为默认词库
#[tauri::command]
pub fn reset_custom_dict(state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    SettingsRepo::set(&conn, "rewrite_custom_dict", "").map_err(|e| e.to_string())?;
    Ok(())
}

/// 词库统计
#[tauri::command]
pub fn get_dict_stats(state: State<'_, AppState>) -> Result<DictStats, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let custom = load_custom_dict(&conn);
    let (builtin, custom_count) = synonym_dict::dict_stats(&custom);
    Ok(DictStats {
        builtin_groups: builtin,
        custom_groups: custom_count,
    })
}

#[derive(Serialize)]
pub struct DictStats {
    pub builtin_groups: usize,
    pub custom_groups: usize,
}

fn load_options(conn: &rusqlite::Connection) -> RewriteOptions {
    let get = |k: &str| SettingsRepo::get(conn, k);
    RewriteOptions {
        enabled: get("rewrite_enabled").map(|v| v == "true").unwrap_or(false),
        intensity: get("rewrite_intensity").unwrap_or_else(|| "medium".to_string()),
        synonym_ratio: get("rewrite_synonym_ratio").and_then(|v| v.parse().ok()).unwrap_or(30),
        sentence_ratio: get("rewrite_sentence_ratio").and_then(|v| v.parse().ok()).unwrap_or(20),
        paragraph_shuffle: get("rewrite_paragraph_shuffle").map(|v| v == "true").unwrap_or(false),
        rewrite_ends: get("rewrite_rewrite_ends").map(|v| v == "true").unwrap_or(false),
        keywords: get("rewrite_keywords")
            .map(|v| {
                v.split(|c: char| matches!(c, ',' | '，') || c.is_whitespace())
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default(),
    }
}

fn load_custom_dict(conn: &rusqlite::Connection) -> Vec<Vec<String>> {
    SettingsRepo::get(conn, "rewrite_custom_dict")
        .and_then(|v| synonym_dict::parse_custom_dict(&v).ok())
        .unwrap_or_default()
}
