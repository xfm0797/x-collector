use rusqlite::Connection;

/// 文章全文索引（FTS5）+ 同步触发器
///
/// 使用 trigram 分词器，支持中日韩文本与英文的子串检索。
const FTS_SCHEMA: &str = "
CREATE VIRTUAL TABLE IF NOT EXISTS articles_fts USING fts5(
    title, content_md, tags, source,
    content='articles',
    content_rowid='id',
    tokenize='trigram'
);

CREATE TRIGGER IF NOT EXISTS articles_ai AFTER INSERT ON articles BEGIN
    INSERT INTO articles_fts(rowid, title, content_md, tags, source)
    VALUES (new.id, new.title, new.content_md, new.tags, new.source);
END;

CREATE TRIGGER IF NOT EXISTS articles_ad AFTER DELETE ON articles BEGIN
    INSERT INTO articles_fts(articles_fts, rowid, title, content_md, tags, source)
    VALUES ('delete', old.id, old.title, old.content_md, old.tags, old.source);
END;

-- 仅正文相关字段变化时重建索引，状态变更不触发
CREATE TRIGGER IF NOT EXISTS articles_au AFTER UPDATE OF title, content_md, tags, source ON articles BEGIN
    INSERT INTO articles_fts(articles_fts, rowid, title, content_md, tags, source)
    VALUES ('delete', old.id, old.title, old.content_md, old.tags, old.source);
    INSERT INTO articles_fts(rowid, title, content_md, tags, source)
    VALUES (new.id, new.title, new.content_md, new.tags, new.source);
END;
";

/// 数据库表结构迁移（v0.0.1）
pub fn run(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "
-- 文章表
CREATE TABLE IF NOT EXISTS articles (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    title         TEXT NOT NULL,
    url           TEXT NOT NULL UNIQUE,
    source        TEXT,
    author        TEXT,
    content_html  TEXT,
    content_md    TEXT,
    excerpt       TEXT,
    tags          TEXT,
    category      TEXT,
    cover_image   TEXT,
    status        TEXT DEFAULT 'collected',
    is_draft      INTEGER DEFAULT 1,
    collected_at  TEXT,
    updated_at    TEXT,
    published_at  TEXT
);

-- 关键词任务表
CREATE TABLE IF NOT EXISTS keyword_tasks (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    keyword           TEXT NOT NULL,
    group_name        TEXT DEFAULT '默认',
    search_engine     TEXT DEFAULT 'baidu',
    site_limit        TEXT,
    match_mode        TEXT DEFAULT 'title',
    max_pages         INTEGER DEFAULT 3,
    interval_minutes  INTEGER DEFAULT 0,
    enabled           INTEGER DEFAULT 1,
    last_run          TEXT,
    created_at        TEXT
);

-- 采集源表
CREATE TABLE IF NOT EXISTS collect_sources (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    name              TEXT NOT NULL,
    url               TEXT NOT NULL UNIQUE,
    source_type       TEXT NOT NULL,
    selector_title    TEXT,
    selector_content  TEXT,
    selector_author   TEXT,
    selector_date     TEXT,
    selector_tags     TEXT,
    selector_cover    TEXT,
    remove_selectors  TEXT,
    group_name        TEXT DEFAULT '默认',
    enabled           INTEGER DEFAULT 1,
    last_collected    TEXT,
    created_at        TEXT
);

-- CMS 连接配置表
CREATE TABLE IF NOT EXISTS cms_connections (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    name              TEXT NOT NULL,
    cms_type          TEXT NOT NULL,
    site_url          TEXT NOT NULL,
    api_path          TEXT,
    username          TEXT NOT NULL,
    password          TEXT NOT NULL,
    default_category  TEXT,
    default_status    TEXT DEFAULT 'publish',
    tag_mapping       TEXT,
    enabled           INTEGER DEFAULT 1,
    last_used         TEXT,
    created_at        TEXT
);

-- 发布记录表
CREATE TABLE IF NOT EXISTS publish_records (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    article_id      INTEGER NOT NULL,
    cms_id          INTEGER,
    publish_type    TEXT NOT NULL,
    status          TEXT NOT NULL,
    remote_id       TEXT,
    remote_url      TEXT,
    error_message   TEXT,
    published_at    TEXT,
    created_at      TEXT,
    FOREIGN KEY (article_id) REFERENCES articles(id) ON DELETE CASCADE,
    FOREIGN KEY (cms_id) REFERENCES cms_connections(id)
);

-- 采集日志表
CREATE TABLE IF NOT EXISTS collect_logs (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id     INTEGER,
    task_type   TEXT,
    keyword     TEXT,
    url         TEXT,
    status      TEXT,
    message     TEXT,
    created_at  TEXT
);

-- 设置表
CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT
);

-- 去重缓存表
CREATE TABLE IF NOT EXISTS dedup_cache (
    url_hash    TEXT PRIMARY KEY,
    title_hash  TEXT,
    fingerprint TEXT,
    created_at  TEXT
);

-- 索引
-- status 单列索引由 (status, collected_at) 复合索引覆盖
DROP INDEX IF EXISTS idx_articles_status;
CREATE INDEX IF NOT EXISTS idx_articles_status_collected ON articles(status, collected_at DESC);
CREATE INDEX IF NOT EXISTS idx_articles_source ON articles(source);
CREATE INDEX IF NOT EXISTS idx_articles_collected_at ON articles(collected_at);
CREATE INDEX IF NOT EXISTS idx_keyword_tasks_enabled ON keyword_tasks(enabled);
CREATE INDEX IF NOT EXISTS idx_collect_logs_created ON collect_logs(created_at);
CREATE INDEX IF NOT EXISTS idx_dedup_cache_title ON dedup_cache(title_hash);
CREATE INDEX IF NOT EXISTS idx_dedup_cache_fingerprint ON dedup_cache(fingerprint);
CREATE INDEX IF NOT EXISTS idx_publish_records_article ON publish_records(article_id);
CREATE INDEX IF NOT EXISTS idx_publish_records_status ON publish_records(status);
CREATE INDEX IF NOT EXISTS idx_publish_records_created ON publish_records(created_at DESC);
        ",
    )?;

    // 全文索引：首次创建时为已有文章回填索引
    let fts_exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'articles_fts')",
        [],
        |r| r.get(0),
    )?;
    conn.execute_batch(FTS_SCHEMA)?;
    if !fts_exists {
        conn.execute("INSERT INTO articles_fts(articles_fts) VALUES('rebuild')", [])?;
    }

    // 伪原创配置默认值
    conn.execute_batch(
        "
INSERT OR IGNORE INTO settings (key, value) VALUES ('rewrite_enabled', 'false');
INSERT OR IGNORE INTO settings (key, value) VALUES ('rewrite_intensity', 'medium');
INSERT OR IGNORE INTO settings (key, value) VALUES ('rewrite_synonym_ratio', '30');
INSERT OR IGNORE INTO settings (key, value) VALUES ('rewrite_sentence_ratio', '20');
INSERT OR IGNORE INTO settings (key, value) VALUES ('rewrite_paragraph_shuffle', 'false');
INSERT OR IGNORE INTO settings (key, value) VALUES ('rewrite_rewrite_ends', 'false');
INSERT OR IGNORE INTO settings (key, value) VALUES ('rewrite_keywords', '');
INSERT OR IGNORE INTO settings (key, value) VALUES ('rewrite_custom_dict', '');
INSERT OR IGNORE INTO settings (key, value) VALUES ('rewrite_ai_api', '');
INSERT OR IGNORE INTO settings (key, value) VALUES ('rewrite_ai_key', '');
INSERT OR IGNORE INTO settings (key, value) VALUES ('language', 'zh');
INSERT OR IGNORE INTO settings (key, value) VALUES ('theme', 'light');
INSERT OR IGNORE INTO settings (key, value) VALUES ('cache_enabled', 'true');
INSERT OR IGNORE INTO settings (key, value) VALUES ('collect_timeout', '30');
INSERT OR IGNORE INTO settings (key, value) VALUES ('collect_concurrency', '3');
INSERT OR IGNORE INTO settings (key, value) VALUES ('collect_interval_min', '2');
INSERT OR IGNORE INTO settings (key, value) VALUES ('collect_interval_max', '5');
INSERT OR IGNORE INTO settings (key, value) VALUES ('collect_retries', '3');
INSERT OR IGNORE INTO settings (key, value) VALUES ('collect_user_agent', '');
INSERT OR IGNORE INTO settings (key, value) VALUES ('default_publish_type', 'hexo');
INSERT OR IGNORE INTO settings (key, value) VALUES ('hexo_output_dir', '');
INSERT OR IGNORE INTO settings (key, value) VALUES ('proxy_url', '');
        ",
    )?;

    Ok(())
}
