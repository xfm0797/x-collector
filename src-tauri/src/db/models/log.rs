use rusqlite::{params, Connection};
use serde::Serialize;

use super::now;

#[derive(Debug, Serialize, Clone)]
pub struct CollectLog {
    pub id: i64,
    pub task_id: Option<i64>,
    pub task_type: Option<String>,
    pub keyword: Option<String>,
    pub url: Option<String>,
    pub status: Option<String>,
    pub message: Option<String>,
    pub created_at: Option<String>,
}

pub struct LogRepo;

impl LogRepo {
    pub fn list(conn: &Connection, limit: i64) -> rusqlite::Result<Vec<CollectLog>> {
        let mut stmt =
            conn.prepare("SELECT * FROM collect_logs ORDER BY created_at DESC LIMIT ?1")?;
        let logs = stmt
            .query_map(params![limit], |row| {
                Ok(CollectLog {
                    id: row.get("id")?,
                    task_id: row.get("task_id")?,
                    task_type: row.get("task_type")?,
                    keyword: row.get("keyword")?,
                    url: row.get("url")?,
                    status: row.get("status")?,
                    message: row.get("message")?,
                    created_at: row.get("created_at")?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(logs)
    }

    pub fn insert(
        conn: &Connection,
        task_id: Option<i64>,
        task_type: &str,
        keyword: Option<&str>,
        url: Option<&str>,
        status: &str,
        message: Option<&str>,
    ) -> rusqlite::Result<()> {
        conn.execute(
            "INSERT INTO collect_logs (task_id, task_type, keyword, url, status, message, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![task_id, task_type, keyword, url, status, message, now()],
        )?;
        Ok(())
    }
}
