//! 采集服务：去重、入库、伪原创与日志的共享管线

use rusqlite::Connection;

use crate::app::config::RewriteConfig;
use crate::collector::engine::Fetcher;
use crate::collector::page_collector::{collect_page, extract_excerpt};
use crate::collector::parser::CustomSelectors;
use crate::collector::rewriter;
use crate::db::models::{ArticleInput, ArticleRepo, DedupRepo, LogRepo};
use crate::utils::hash::{fingerprint, sha256_hex};

/// 抓取单页并解析为文章输入（不落库、不做伪原创）
pub async fn fetch_page(
    fetcher: &Fetcher,
    url: &str,
    selectors: Option<&CustomSelectors>,
    remove_selectors: &[String],
) -> Result<ArticleInput, String> {
    collect_page(fetcher, url, selectors, remove_selectors)
        .await
        .map(|(input, _)| input)
}

/// 三重去重：URL 哈希 + 标题相似度（≥0.85 视为重复）+ 内容指纹
pub fn is_duplicate(conn: &Connection, input: &ArticleInput) -> bool {
    let url_hash = sha256_hex(&input.url);
    if DedupRepo::url_exists(conn, &url_hash) {
        return true;
    }
    if let Some(md) = input.content_md.as_deref() {
        let fp = fingerprint(md);
        if DedupRepo::fingerprint_exists(conn, &fp) {
            return true;
        }
    }
    !DedupRepo::find_similar_title(conn, &input.title, 0.85).is_empty()
}

/// 记录去重缓存
pub fn record_dedup(conn: &Connection, input: &ArticleInput) {
    let url_hash = sha256_hex(&input.url);
    let title_hash = sha256_hex(&input.title);
    let fp = input
        .content_md
        .as_deref()
        .map(fingerprint)
        .unwrap_or_default();
    DedupRepo::insert(conn, &url_hash, &title_hash, &fp).ok();
}

/// 按设置自动应用伪原创（就地修改正文与摘要）
pub fn apply_rewrite(input: &mut ArticleInput, config: &RewriteConfig) {
    if !config.options.enabled {
        return;
    }
    if let Some(md) = input.content_md.as_ref() {
        let rewritten = rewriter::rewrite_markdown(md, &config.options, &config.custom_dict);
        input.content_md = Some(rewritten);
        input.excerpt = Some(extract_excerpt(
            input.content_md.as_deref().unwrap_or(""),
        ));
    }
}

/// 下载正文图片并以 data URI 内嵌（失败保留原链接）
pub async fn localize_images(input: &mut ArticleInput, fetcher: &Fetcher) {
    crate::utils::images::localize_article(input, fetcher).await;
}

/// 入库并写入去重缓存；URL 已存在时返回 None
pub fn save_article(conn: &Connection, input: &ArticleInput) -> Result<Option<i64>, String> {
    let id = ArticleRepo::insert_if_new(conn, input).map_err(|e| e.to_string())?;
    if id.is_some() {
        record_dedup(conn, input);
    }
    Ok(id)
}

/// 写采集日志（失败时忽略）
pub fn log(
    conn: &Connection,
    task_id: Option<i64>,
    task_type: &str,
    keyword: Option<&str>,
    url: Option<&str>,
    status: &str,
    message: &str,
) {
    LogRepo::insert(conn, task_id, task_type, keyword, url, status, Some(message)).ok();
}
