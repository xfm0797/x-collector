use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};

use super::{now, parse_json_list};

pub use crate::domain::article::ArticleInput;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Article {
    pub id: i64,
    pub title: String,
    pub url: String,
    pub source: Option<String>,
    pub author: Option<String>,
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    pub content_html: Option<String>,
    pub content_md: Option<String>,
    pub excerpt: Option<String>,
    pub tags: Option<Vec<String>>,
    pub category: Option<String>,
    pub cover_image: Option<String>,
    pub status: String,
    pub is_draft: i64,
    pub collected_at: Option<String>,
    pub updated_at: Option<String>,
    pub published_at: Option<String>,
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default)]
pub struct ArticleQuery {
    pub keyword: Option<String>,
    pub status: Option<String>,
    pub source: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub sort: Option<String>,
    pub order: Option<String>,
}

/// 列表查询列（不加载正文 HTML，保持返回结构不变）
const LIST_COLUMNS: &str = "id, title, url, source, author, content_md, excerpt, tags, category, cover_image, status, is_draft, collected_at, updated_at, published_at";

impl Article {
    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        let tags_json: Option<String> = row.get("tags")?;
        Ok(Article {
            id: row.get("id")?,
            title: row.get("title")?,
            url: row.get("url")?,
            source: row.get("source")?,
            author: row.get("author")?,
            content_html: row.get("content_html")?,
            content_md: row.get("content_md")?,
            excerpt: row.get("excerpt")?,
            tags: parse_json_list(tags_json),
            category: row.get("category")?,
            cover_image: row.get("cover_image")?,
            status: row.get("status")?,
            is_draft: row.get("is_draft")?,
            collected_at: row.get("collected_at")?,
            updated_at: row.get("updated_at")?,
            published_at: row.get("published_at")?,
        })
    }

    fn from_row_lean(row: &Row) -> rusqlite::Result<Self> {
        let tags_json: Option<String> = row.get("tags")?;
        Ok(Article {
            id: row.get("id")?,
            title: row.get("title")?,
            url: row.get("url")?,
            source: row.get("source")?,
            author: row.get("author")?,
            content_html: None,
            content_md: row.get("content_md")?,
            excerpt: row.get("excerpt")?,
            tags: parse_json_list(tags_json),
            category: row.get("category")?,
            cover_image: row.get("cover_image")?,
            status: row.get("status")?,
            is_draft: row.get("is_draft")?,
            collected_at: row.get("collected_at")?,
            updated_at: row.get("updated_at")?,
            published_at: row.get("published_at")?,
        })
    }
}

pub struct ArticleRepo;

impl ArticleRepo {
    pub fn list(conn: &Connection, q: &ArticleQuery) -> rusqlite::Result<(Vec<Article>, i64)> {
        let page = q.page.unwrap_or(1).max(1);
        let page_size = q.page_size.unwrap_or(10).clamp(1, 100);
        let sort_col = match q.sort.as_deref() {
            Some("title") => "title",
            Some("status") => "status",
            _ => "collected_at",
        };
        let order = if q.order.as_deref() == Some("asc") { "ASC" } else { "DESC" };

        let mut where_clause = String::from("WHERE 1=1");
        let mut params_vec: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        if let Some(kw) = q.keyword.as_deref().filter(|s| !s.trim().is_empty()) {
            where_clause.push_str(" AND (title LIKE ?1 OR content_md LIKE ?1 OR tags LIKE ?1 OR source LIKE ?1)");
            params_vec.push(Box::new(format!("%{}%", kw.trim())));
        }
        if let Some(st) = q.status.as_deref().filter(|s| !s.is_empty()) {
            where_clause.push_str(&format!(" AND status = ?{}", params_vec.len() + 1));
            params_vec.push(Box::new(st.to_string()));
        }
        if let Some(src) = q.source.as_deref().filter(|s| !s.is_empty()) {
            where_clause.push_str(&format!(" AND source = ?{}", params_vec.len() + 1));
            params_vec.push(Box::new(src.to_string()));
        }

        let total: i64 = conn
            .query_row(
                &format!("SELECT COUNT(*) FROM articles {}", where_clause),
                rusqlite::params_from_iter(params_vec.iter().map(|p| p.as_ref())),
                |r| r.get(0),
            )
            .unwrap_or(0);

        let sql = format!(
            "SELECT {} FROM articles {} ORDER BY {} {} LIMIT {} OFFSET {}",
            LIST_COLUMNS,
            where_clause,
            sort_col,
            order,
            page_size,
            (page - 1) * page_size
        );

        let mut stmt = conn.prepare(&sql)?;
        let articles = stmt
            .query_map(
                rusqlite::params_from_iter(params_vec.iter().map(|p| p.as_ref())),
                Article::from_row_lean,
            )?
            .collect::<Result<Vec<_>, _>>()?;

        Ok((articles, total))
    }

    pub fn get(conn: &Connection, id: i64) -> rusqlite::Result<Option<Article>> {
        conn.query_row("SELECT * FROM articles WHERE id = ?1", params![id], |row| {
            Article::from_row(row)
        })
        .map(Some)
        .or_else(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            e => Err(e),
        })
    }

    /// 插入文章；URL 已存在时跳过并返回 None
    pub fn insert_if_new(conn: &Connection, input: &ArticleInput) -> rusqlite::Result<Option<i64>> {
        let tags_json = input
            .tags
            .as_ref()
            .map(|t| serde_json::to_string(t).unwrap_or_default());
        let affected = conn.execute(
            "INSERT OR IGNORE INTO articles (title, url, source, author, content_html, content_md, excerpt, tags, category, cover_image, status, collected_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?12)",
            params![
                input.title,
                input.url,
                input.source,
                input.author,
                input.content_html,
                input.content_md,
                input.excerpt,
                tags_json,
                input.category,
                input.cover_image,
                input.status.as_deref().unwrap_or("collected"),
                now(),
            ],
        )?;
        if affected == 0 {
            return Ok(None);
        }
        Ok(Some(conn.last_insert_rowid()))
    }

    pub fn create(conn: &Connection, input: &ArticleInput) -> rusqlite::Result<Article> {
        let id = Self::insert_if_new(conn, input)?
            .ok_or_else(|| rusqlite::Error::InvalidParameterName("URL 已存在".into()))?;
        Self::get(conn, id).map(|a| a.unwrap())
    }

    pub fn update(conn: &Connection, id: i64, input: &ArticleInput) -> rusqlite::Result<()> {
        let tags_json = input
            .tags
            .as_ref()
            .map(|t| serde_json::to_string(t).unwrap_or_default());
        conn.execute(
            "UPDATE articles SET title = ?1, url = ?2, source = COALESCE(?3, source), author = COALESCE(?4, author),
             content_md = COALESCE(?5, content_md), excerpt = COALESCE(?6, excerpt), tags = COALESCE(?7, tags),
             category = COALESCE(?8, category), status = 'edited', updated_at = ?9
             WHERE id = ?10",
            params![
                input.title,
                input.url,
                input.source,
                input.author,
                input.content_md,
                input.excerpt,
                tags_json,
                input.category,
                now(),
                id,
            ],
        )?;
        Ok(())
    }

    pub fn update_status(conn: &Connection, id: i64, status: &str) -> rusqlite::Result<()> {
        let published_at = if status == "published" { Some(now()) } else { None };
        conn.execute(
            "UPDATE articles SET status = ?1, published_at = COALESCE(?2, published_at), updated_at = ?3 WHERE id = ?4",
            params![status, published_at, now(), id],
        )?;
        Ok(())
    }

    pub fn update_content_md(conn: &Connection, id: i64, md: &str) -> rusqlite::Result<()> {
        conn.execute(
            "UPDATE articles SET content_md = ?1, status = 'edited', updated_at = ?2 WHERE id = ?3",
            params![md, now(), id],
        )?;
        Ok(())
    }

    pub fn delete(conn: &Connection, ids: &[i64]) -> rusqlite::Result<()> {
        for id in ids {
            // 先取出去重所需字段，删除后同步清理 dedup_cache
            let cached = conn
                .query_row(
                    "SELECT url, content_md FROM articles WHERE id = ?1",
                    params![id],
                    |r| Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?)),
                )
                .ok();
            conn.execute("DELETE FROM articles WHERE id = ?1", params![id])?;
            conn.execute("DELETE FROM publish_records WHERE article_id = ?1", params![id])?;
            if let Some((url, content_md)) = cached {
                super::DedupRepo::remove_for_article(conn, &url, content_md.as_deref())?;
            }
        }
        Ok(())
    }
}
