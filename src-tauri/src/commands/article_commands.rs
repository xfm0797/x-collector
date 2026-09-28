use serde::Serialize;
use tauri::State;

use crate::db::models::{ArticleInput, ArticleQuery, ArticleRepo};
use crate::AppState;

/// 文章分页列表
#[tauri::command]
pub fn list_articles(state: State<'_, AppState>, query: ArticleQuery) -> Result<PageData, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let (list, total) = ArticleRepo::list(&conn, &query).map_err(|e| e.to_string())?;
    Ok(PageData {
        list,
        total,
        page: query.page.unwrap_or(1),
        page_size: query.page_size.unwrap_or(10),
    })
}

#[derive(Serialize)]
pub struct PageData {
    pub list: Vec<crate::db::models::Article>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

/// 文章详情
#[tauri::command]
pub fn get_article(state: State<'_, AppState>, id: i64) -> Result<Article, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    ArticleRepo::get(&conn, id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "文章不存在".to_string())
}

type Article = crate::db::models::Article;

/// 新建文章
#[tauri::command]
pub fn create_article(state: State<'_, AppState>, input: ArticleInput) -> Result<Article, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    if input.title.trim().is_empty() {
        return Err("标题不能为空".to_string());
    }
    if input.url.trim().is_empty() {
        return Err("URL 不能为空".to_string());
    }
    ArticleRepo::create(&conn, &input).map_err(|e| e.to_string())
}

/// 更新文章
#[tauri::command]
pub fn update_article(state: State<'_, AppState>, id: i64, input: ArticleInput) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    ArticleRepo::update(&conn, id, &input).map_err(|e| e.to_string())
}

/// 更新文章状态
#[tauri::command]
pub fn update_article_status(state: State<'_, AppState>, id: i64, status: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    ArticleRepo::update_status(&conn, id, &status).map_err(|e| e.to_string())
}

/// 删除文章（单条/批量）
#[tauri::command]
pub fn delete_articles(state: State<'_, AppState>, ids: Vec<i64>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    ArticleRepo::delete(&conn, &ids).map_err(|e| e.to_string())
}

/// 仪表盘统计
#[tauri::command]
pub fn get_dashboard_stats(state: State<'_, AppState>) -> Result<DashboardStats, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    get_stats(&conn).map_err(|e| e.to_string())
}

#[derive(Serialize)]
pub struct DashboardStats {
    pub article_total: i64,
    pub today_collected: i64,
    pub pending_publish: i64,
    pub source_total: i64,
    pub keyword_total: i64,
    pub published_total: i64,
    pub publish_by_type: Vec<PublishTypeCount>,
    pub recent_articles: Vec<RecentArticle>,
}

#[derive(Serialize)]
pub struct PublishTypeCount {
    pub publish_type: String,
    pub count: i64,
}

#[derive(Serialize)]
pub struct RecentArticle {
    pub id: i64,
    pub title: String,
    pub source: Option<String>,
    pub collected_at: Option<String>,
    pub status: String,
}

fn get_stats(conn: &rusqlite::Connection) -> Result<DashboardStats, rusqlite::Error> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    let article_total: i64 = conn.query_row("SELECT COUNT(*) FROM articles", [], |r| r.get(0))?;
    let today_collected: i64 = conn.query_row(
        "SELECT COUNT(*) FROM articles WHERE collected_at LIKE ?1",
        rusqlite::params![format!("{}%", today)],
        |r| r.get(0),
    )?;
    let pending_publish: i64 = conn.query_row(
        "SELECT COUNT(*) FROM articles WHERE status IN ('collected', 'edited')",
        [],
        |r| r.get(0),
    )?;
    let source_total: i64 = conn.query_row("SELECT COUNT(*) FROM collect_sources", [], |r| r.get(0))?;
    let keyword_total: i64 = conn.query_row("SELECT COUNT(*) FROM keyword_tasks", [], |r| r.get(0))?;
    let published_total: i64 = conn.query_row(
        "SELECT COUNT(DISTINCT article_id) FROM publish_records WHERE status = 'success'",
        [],
        |r| r.get(0),
    )?;

    let mut stmt = conn.prepare(
        "SELECT publish_type, COUNT(*) as cnt FROM publish_records WHERE status = 'success'
         GROUP BY publish_type ORDER BY cnt DESC",
    )?;
    let publish_by_type = stmt
        .query_map([], |r| {
            Ok(PublishTypeCount {
                publish_type: r.get(0)?,
                count: r.get(1)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut stmt = conn.prepare(
        "SELECT id, title, source, collected_at, status FROM articles
         ORDER BY collected_at DESC LIMIT 5",
    )?;
    let recent_articles = stmt
        .query_map([], |r| {
            Ok(RecentArticle {
                id: r.get(0)?,
                title: r.get(1)?,
                source: r.get(2)?,
                collected_at: r.get(3)?,
                status: r.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(DashboardStats {
        article_total,
        today_collected,
        pending_publish,
        source_total,
        keyword_total,
        published_total,
        publish_by_type,
        recent_articles,
    })
}
