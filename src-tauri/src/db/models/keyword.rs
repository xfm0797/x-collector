use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};

use super::now;

#[derive(Debug, Serialize, Clone)]
pub struct KeywordTask {
    pub id: i64,
    pub keyword: String,
    pub group_name: String,
    pub search_engine: String,
    pub site_limit: Option<String>,
    pub match_mode: String,
    pub max_pages: i64,
    pub interval_minutes: i64,
    pub enabled: i64,
    pub last_run: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct KeywordTaskInput {
    pub keyword: String,
    pub group_name: Option<String>,
    pub search_engine: Option<String>,
    pub site_limit: Option<String>,
    pub match_mode: Option<String>,
    pub max_pages: Option<i64>,
    pub interval_minutes: Option<i64>,
    pub enabled: Option<i64>,
}

impl KeywordTask {
    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(KeywordTask {
            id: row.get("id")?,
            keyword: row.get("keyword")?,
            group_name: row.get("group_name")?,
            search_engine: row.get("search_engine")?,
            site_limit: row.get("site_limit")?,
            match_mode: row.get("match_mode")?,
            max_pages: row.get("max_pages")?,
            interval_minutes: row.get("interval_minutes")?,
            enabled: row.get("enabled")?,
            last_run: row.get("last_run")?,
            created_at: row.get("created_at")?,
        })
    }
}

pub struct KeywordRepo;

impl KeywordRepo {
    pub fn list(conn: &Connection) -> rusqlite::Result<Vec<KeywordTask>> {
        let mut stmt = conn.prepare("SELECT * FROM keyword_tasks ORDER BY created_at DESC")?;
        let rows = stmt.query_map([], KeywordTask::from_row)?;
        let tasks = rows.collect::<Result<Vec<_>, _>>()?;
        Ok(tasks)
    }

    pub fn get(conn: &Connection, id: i64) -> rusqlite::Result<KeywordTask> {
        conn.query_row("SELECT * FROM keyword_tasks WHERE id = ?1", params![id], |r| {
            KeywordTask::from_row(r)
        })
    }

    pub fn create(conn: &Connection, input: &KeywordTaskInput) -> rusqlite::Result<KeywordTask> {
        conn.execute(
            "INSERT INTO keyword_tasks (keyword, group_name, search_engine, site_limit, match_mode, max_pages, interval_minutes, enabled, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                input.keyword,
                input.group_name.as_deref().unwrap_or("默认"),
                input.search_engine.as_deref().unwrap_or("baidu"),
                input.site_limit,
                input.match_mode.as_deref().unwrap_or("title"),
                input.max_pages.unwrap_or(3).clamp(1, 10),
                input.interval_minutes.unwrap_or(0),
                input.enabled.unwrap_or(1),
                now(),
            ],
        )?;
        Self::get(conn, conn.last_insert_rowid())
    }

    pub fn update(conn: &Connection, id: i64, input: &KeywordTaskInput) -> rusqlite::Result<()> {
        conn.execute(
            "UPDATE keyword_tasks SET
             keyword = ?1,
             group_name = COALESCE(?2, group_name),
             search_engine = COALESCE(?3, search_engine),
             site_limit = COALESCE(?4, site_limit),
             match_mode = COALESCE(?5, match_mode),
             max_pages = COALESCE(?6, max_pages),
             interval_minutes = COALESCE(?7, interval_minutes),
             enabled = COALESCE(?8, enabled)
             WHERE id = ?9",
            params![
                input.keyword,
                input.group_name,
                input.search_engine,
                input.site_limit,
                input.match_mode,
                input.max_pages,
                input.interval_minutes,
                input.enabled,
                id,
            ],
        )?;
        Ok(())
    }

    pub fn delete(conn: &Connection, ids: &[i64]) -> rusqlite::Result<()> {
        for id in ids {
            conn.execute("DELETE FROM keyword_tasks WHERE id = ?1", params![id])?;
        }
        Ok(())
    }

    pub fn set_last_run(conn: &Connection, id: i64) -> rusqlite::Result<()> {
        conn.execute(
            "UPDATE keyword_tasks SET last_run = ?1 WHERE id = ?2",
            params![now(), id],
        )?;
        Ok(())
    }
}
