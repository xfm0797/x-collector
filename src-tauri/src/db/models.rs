use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};

fn now() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

fn parse_json_list<T: serde::de::DeserializeOwned>(text: Option<String>) -> Option<Vec<T>> {
    text.and_then(|s| serde_json::from_str(&s).ok())
}

// ── 文章 ────────────────────────────────────────────────

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

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct ArticleInput {
    pub title: String,
    pub url: String,
    pub source: Option<String>,
    pub author: Option<String>,
    pub content_html: Option<String>,
    pub content_md: Option<String>,
    pub excerpt: Option<String>,
    pub tags: Option<Vec<String>>,
    pub category: Option<String>,
    pub cover_image: Option<String>,
    pub status: Option<String>,
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
            "SELECT * FROM articles {} ORDER BY {} {} LIMIT {} OFFSET {}",
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
                Article::from_row,
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
            conn.execute("DELETE FROM articles WHERE id = ?1", params![id])?;
            conn.execute("DELETE FROM publish_records WHERE article_id = ?1", params![id])?;
        }
        Ok(())
    }
}

// ── 关键词任务 ──────────────────────────────────────────

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

// ── 采集源 ──────────────────────────────────────────────

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

// ── CMS 连接 ────────────────────────────────────────────

#[derive(Debug, Serialize, Clone)]
pub struct CmsConnection {
    pub id: i64,
    pub name: String,
    pub cms_type: String,
    pub site_url: String,
    pub api_path: Option<String>,
    pub username: String,
    pub default_category: Option<String>,
    pub default_status: String,
    pub tag_mapping: Option<String>,
    pub enabled: i64,
    pub has_password: bool,
    pub last_used: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct CmsConnectionInput {
    pub id: Option<i64>,
    pub name: String,
    pub cms_type: Option<String>,
    pub site_url: String,
    pub api_path: Option<String>,
    pub username: String,
    pub password: Option<String>,
    pub default_category: Option<String>,
    pub default_status: Option<String>,
    pub tag_mapping: Option<serde_json::Value>,
    pub enabled: Option<i64>,
}

impl CmsConnection {
    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        let password: String = row.get("password")?;
        Ok(CmsConnection {
            id: row.get("id")?,
            name: row.get("name")?,
            cms_type: row.get("cms_type")?,
            site_url: row.get("site_url")?,
            api_path: row.get("api_path")?,
            username: row.get("username")?,
            default_category: row.get("default_category")?,
            default_status: row.get("default_status")?,
            tag_mapping: row.get("tag_mapping")?,
            enabled: row.get("enabled")?,
            has_password: !password.is_empty(),
            last_used: row.get("last_used")?,
            created_at: row.get("created_at")?,
        })
    }
}

pub struct CmsRepo;

impl CmsRepo {
    pub fn list(conn: &Connection) -> rusqlite::Result<Vec<CmsConnection>> {
        let mut stmt = conn.prepare("SELECT * FROM cms_connections ORDER BY created_at DESC")?;
        let rows = stmt.query_map([], CmsConnection::from_row)?;
        let connections = rows.collect::<Result<Vec<_>, _>>()?;
        Ok(connections)
    }

    pub fn get(conn: &Connection, id: i64) -> rusqlite::Result<CmsConnection> {
        conn.query_row("SELECT * FROM cms_connections WHERE id = ?1", params![id], |r| {
            CmsConnection::from_row(r)
        })
    }

    /// 读取（解密后的）密码，供发布模块使用
    pub fn get_password(conn: &Connection, id: i64) -> rusqlite::Result<String> {
        let encrypted: String =
            conn.query_row("SELECT password FROM cms_connections WHERE id = ?1", params![id], |r| r.get(0))?;
        Ok(crate::utils::hash::decrypt_password(&encrypted))
    }

    pub fn save(conn: &Connection, input: &CmsConnectionInput) -> rusqlite::Result<CmsConnection> {
        let tag_mapping = input
            .tag_mapping
            .as_ref()
            .map(|v| serde_json::to_string(v).unwrap_or_default());
        let cms_type = input.cms_type.as_deref().unwrap_or("wordpress");
        let default_api = match cms_type {
            "wordpress" => "/xmlrpc.php",
            "typecho" => "/action/xmlrpc",
            "zblog" => "/zb_system/xml-rpc/index.php",
            _ => "",
        };

        if let Some(id) = input.id {
            conn.execute(
                "UPDATE cms_connections SET name = ?1, cms_type = ?2, site_url = ?3,
                 api_path = COALESCE(?4, api_path), username = ?5,
                 password = COALESCE(?6, password), default_category = COALESCE(?7, default_category),
                 default_status = COALESCE(?8, default_status), tag_mapping = COALESCE(?9, tag_mapping),
                 enabled = COALESCE(?10, enabled) WHERE id = ?11",
                params![
                    input.name,
                    cms_type,
                    input.site_url,
                    input.api_path.as_deref().filter(|s| !s.is_empty()).or(Some(default_api)),
                    input.username,
                    input.password.as_deref().filter(|s| !s.is_empty()).map(crate::utils::hash::encrypt_password),
                    input.default_category,
                    input.default_status.as_deref().unwrap_or("publish"),
                    tag_mapping,
                    input.enabled.unwrap_or(1),
                    id,
                ],
            )?;
            Self::get(conn, id)
        } else {
            conn.execute(
                "INSERT INTO cms_connections (name, cms_type, site_url, api_path, username, password, default_category, default_status, tag_mapping, enabled, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    input.name,
                    cms_type,
                    input.site_url,
                    input.api_path.as_deref().filter(|s| !s.is_empty()).or(Some(default_api)),
                    input.username,
                    input.password.as_deref().map(crate::utils::hash::encrypt_password).unwrap_or_default(),
                    input.default_category,
                    input.default_status.as_deref().unwrap_or("publish"),
                    tag_mapping,
                    input.enabled.unwrap_or(1),
                    now(),
                ],
            )?;
            Self::get(conn, conn.last_insert_rowid())
        }
    }

    pub fn delete(conn: &Connection, ids: &[i64]) -> rusqlite::Result<()> {
        for id in ids {
            conn.execute("DELETE FROM cms_connections WHERE id = ?1", params![id])?;
        }
        Ok(())
    }

    pub fn set_last_used(conn: &Connection, id: i64) -> rusqlite::Result<()> {
        conn.execute(
            "UPDATE cms_connections SET last_used = ?1 WHERE id = ?2",
            params![now(), id],
        )?;
        Ok(())
    }
}

// ── 发布记录 ────────────────────────────────────────────

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
    pub fn list(conn: &Connection, limit: i64) -> rusqlite::Result<Vec<PublishRecord>> {
        let mut stmt = conn.prepare(
            "SELECT p.*, a.title AS article_title, c.name AS cms_name
             FROM publish_records p
             LEFT JOIN articles a ON a.id = p.article_id
             LEFT JOIN cms_connections c ON c.id = p.cms_id
             ORDER BY p.created_at DESC LIMIT ?1",
        )?;
        let records = stmt
            .query_map(params![limit], |row| {
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
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(records)
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

// ── 采集日志 ────────────────────────────────────────────

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

// ── 设置 ────────────────────────────────────────────────

pub struct SettingsRepo;

impl SettingsRepo {
    pub fn get_all(conn: &Connection) -> rusqlite::Result<std::collections::HashMap<String, String>> {
        let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
        let mut map = std::collections::HashMap::new();
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        for row in rows {
            let (k, v) = row?;
            map.insert(k, v);
        }
        Ok(map)
    }

    pub fn get(conn: &Connection, key: &str) -> Option<String> {
        conn.query_row("SELECT value FROM settings WHERE key = ?1", params![key], |r| r.get(0))
            .ok()
    }

    pub fn set(conn: &Connection, key: &str, value: &str) -> rusqlite::Result<()> {
        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = ?2",
            params![key, value],
        )?;
        Ok(())
    }
}

// ── 去重缓存 ────────────────────────────────────────────

pub struct DedupRepo;

impl DedupRepo {
    pub fn url_exists(conn: &Connection, url_hash: &str) -> bool {
        conn.query_row("SELECT 1 FROM dedup_cache WHERE url_hash = ?1", params![url_hash], |_| Ok(()))
            .is_ok()
    }

    pub fn fingerprint_exists(conn: &Connection, fingerprint: &str) -> bool {
        conn.query_row(
            "SELECT 1 FROM dedup_cache WHERE fingerprint = ?1",
            params![fingerprint],
            |_| Ok(()),
        )
        .is_ok()
    }

    /// 返回与给定标题相似（超过阈值）的已有标题数量
    pub fn find_similar_title(conn: &Connection, title: &str, threshold: f64) -> Vec<String> {
        let Ok(mut stmt) = conn.prepare("SELECT title FROM articles") else {
            return vec![];
        };
        let Ok(titles) = stmt.query_map([], |r| r.get::<_, String>(0)) else {
            return vec![];
        };
        let mut similar = vec![];
        for t in titles.flatten() {
            let sim = crate::collector::dedup::title_similarity(&t, title);
            if sim >= threshold {
                similar.push(t);
            }
        }
        similar
    }

    pub fn insert(conn: &Connection, url_hash: &str, title_hash: &str, fingerprint: &str) -> rusqlite::Result<()> {
        conn.execute(
            "INSERT OR IGNORE INTO dedup_cache (url_hash, title_hash, fingerprint, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![url_hash, title_hash, fingerprint, now()],
        )?;
        Ok(())
    }
}
