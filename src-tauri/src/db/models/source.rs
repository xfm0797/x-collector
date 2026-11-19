use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};

use super::{now, parse_json_list};

#[derive(Debug, Serialize, Clone)]
pub struct CollectSource {
    pub id: i64,
    pub name: String,
    pub url: String,
    pub source_type: String,
    pub selector_title: Option<String>,
    pub selector_content: Option<String>,
    pub selector_author: Option<String>,
    pub selector_date: Option<String>,
    pub selector_tags: Option<String>,
    pub selector_cover: Option<String>,
    pub remove_selectors: Option<Vec<String>>,
    pub group_name: String,
    pub enabled: i64,
    pub last_collected: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct CollectSourceInput {
    pub name: String,
    pub url: String,
    pub source_type: Option<String>,
    pub selector_title: Option<String>,
    pub selector_content: Option<String>,
    pub selector_author: Option<String>,
    pub selector_date: Option<String>,
    pub selector_tags: Option<String>,
    pub selector_cover: Option<String>,
    pub remove_selectors: Option<Vec<String>>,
    pub group_name: Option<String>,
    pub enabled: Option<i64>,
}

impl CollectSource {
    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        let remove_json: Option<String> = row.get("remove_selectors")?;
        Ok(CollectSource {
            id: row.get("id")?,
            name: row.get("name")?,
            url: row.get("url")?,
            source_type: row.get("source_type")?,
            selector_title: row.get("selector_title")?,
            selector_content: row.get("selector_content")?,
            selector_author: row.get("selector_author")?,
            selector_date: row.get("selector_date")?,
            selector_tags: row.get("selector_tags")?,
            selector_cover: row.get("selector_cover")?,
            remove_selectors: parse_json_list(remove_json),
            group_name: row.get("group_name")?,
            enabled: row.get("enabled")?,
            last_collected: row.get("last_collected")?,
            created_at: row.get("created_at")?,
        })
    }
}

pub struct SourceRepo;

impl SourceRepo {
    pub fn list(conn: &Connection) -> rusqlite::Result<Vec<CollectSource>> {
        let mut stmt = conn.prepare("SELECT * FROM collect_sources ORDER BY created_at DESC")?;
        let rows = stmt.query_map([], CollectSource::from_row)?;
        let sources = rows.collect::<Result<Vec<_>, _>>()?;
        Ok(sources)
    }

    pub fn get(conn: &Connection, id: i64) -> rusqlite::Result<CollectSource> {
        conn.query_row("SELECT * FROM collect_sources WHERE id = ?1", params![id], |r| {
            CollectSource::from_row(r)
        })
    }

    pub fn create(conn: &Connection, input: &CollectSourceInput) -> rusqlite::Result<CollectSource> {
        let remove_json = input
            .remove_selectors
            .as_ref()
            .map(|r| serde_json::to_string(r).unwrap_or_default());
        conn.execute(
            "INSERT INTO collect_sources (name, url, source_type, selector_title, selector_content, selector_author, selector_date, selector_tags, selector_cover, remove_selectors, group_name, enabled, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                input.name,
                input.url,
                input.source_type.as_deref().unwrap_or("rss"),
                input.selector_title,
                input.selector_content,
                input.selector_author,
                input.selector_date,
                input.selector_tags,
                input.selector_cover,
                remove_json,
                input.group_name.as_deref().unwrap_or("默认"),
                input.enabled.unwrap_or(1),
                now(),
            ],
        )?;
        Self::get(conn, conn.last_insert_rowid())
    }

    pub fn update(conn: &Connection, id: i64, input: &CollectSourceInput) -> rusqlite::Result<()> {
        let remove_json = input
            .remove_selectors
            .as_ref()
            .map(|r| serde_json::to_string(r).unwrap_or_default());
        conn.execute(
            "UPDATE collect_sources SET
             name = ?1, url = ?2, source_type = COALESCE(?3, source_type),
             selector_title = COALESCE(?4, selector_title), selector_content = COALESCE(?5, selector_content),
             selector_author = COALESCE(?6, selector_author), selector_date = COALESCE(?7, selector_date),
             selector_tags = COALESCE(?8, selector_tags), selector_cover = COALESCE(?9, selector_cover),
             remove_selectors = COALESCE(?10, remove_selectors), group_name = COALESCE(?11, group_name),
             enabled = COALESCE(?12, enabled)
             WHERE id = ?13",
            params![
                input.name,
                input.url,
                input.source_type,
                input.selector_title,
                input.selector_content,
                input.selector_author,
                input.selector_date,
                input.selector_tags,
                input.selector_cover,
                remove_json,
                input.group_name,
                input.enabled,
                id,
            ],
        )?;
        Ok(())
    }

    pub fn delete(conn: &Connection, ids: &[i64]) -> rusqlite::Result<()> {
        for id in ids {
            conn.execute("DELETE FROM collect_sources WHERE id = ?1", params![id])?;
        }
        Ok(())
    }

    pub fn set_last_collected(conn: &Connection, id: i64) -> rusqlite::Result<()> {
        conn.execute(
            "UPDATE collect_sources SET last_collected = ?1 WHERE id = ?2",
            params![now(), id],
        )?;
        Ok(())
    }
}
