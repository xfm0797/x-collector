use rusqlite::{params, Connection, Row};
use serde::Serialize;

use super::now;

#[derive(Debug, Serialize, Clone)]
pub struct PublishRecord {
    pub id: i64,
    pub article_id: i64,
    pub article_title: Option<String>,
    pub cms_id: Option<i64>,
    pub cms_name: Option<String>,
    pub publish_type: String,
    pub status: String,
    pub remote_id: Option<String>,
    pub remote_url: Option<String>,
    pub error_message: Option<String>,
    pub published_at: Option<String>,
    pub created_at: Option<String>,
}

pub struct PublishRepo;

impl PublishRepo {
    fn from_row(row: &Row) -> rusqlite::Result<PublishRecord> {
        Ok(PublishRecord {
            id: row.get("id")?,
            article_id: row.get("article_id")?,
            article_title: row.get("article_title")?,
            cms_id: row.get("cms_id")?,
            cms_name: row.get("cms_name")?,
            publish_type: row.get("publish_type")?,
            status: row.get("status")?,
            remote_id: row.get("remote_id")?,
            remote_url: row.get("remote_url")?,
            error_message: row.get("error_message")?,
            published_at: row.get("published_at")?,
            created_at: row.get("created_at")?,
        })
    }

    pub fn list(conn: &Connection, limit: i64) -> rusqlite::Result<Vec<PublishRecord>> {
        let mut stmt = conn.prepare(
            "SELECT p.*, a.title AS article_title, c.name AS cms_name
             FROM publish_records p
             LEFT JOIN articles a ON a.id = p.article_id
             LEFT JOIN cms_connections c ON c.id = p.cms_id
             ORDER BY p.created_at DESC LIMIT ?1",
        )?;
        let records = stmt
            .query_map(params![limit], Self::from_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(records)
    }

    /// 按 id 查询单条发布记录（含文章标题与 CMS 名称）
    pub fn get(conn: &Connection, id: i64) -> rusqlite::Result<Option<PublishRecord>> {
        conn.query_row(
            "SELECT p.*, a.title AS article_title, c.name AS cms_name
             FROM publish_records p
             LEFT JOIN articles a ON a.id = p.article_id
             LEFT JOIN cms_connections c ON c.id = p.cms_id
             WHERE p.id = ?1",
            params![id],
            Self::from_row,
        )
        .map(Some)
        .or_else(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            e => Err(e),
        })
    }

    pub fn insert(
        conn: &Connection,
        article_id: i64,
        cms_id: Option<i64>,
        publish_type: &str,
        status: &str,
        remote_id: Option<&str>,
        remote_url: Option<&str>,
        error_message: Option<&str>,
    ) -> rusqlite::Result<i64> {
        conn.execute(
            "INSERT INTO publish_records (article_id, cms_id, publish_type, status, remote_id, remote_url, error_message, published_at, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)",
            params![
                article_id,
                cms_id,
                publish_type,
                status,
                remote_id,
                remote_url,
                error_message,
                if status == "success" { Some(now()) } else { None },
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }
}
