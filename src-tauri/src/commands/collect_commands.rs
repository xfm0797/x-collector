use serde::Serialize;
use tauri::{Emitter, State};

use crate::collector::cleaners::CleanOptions;
use crate::collector::engine::{FetchSettings, Fetcher};
use crate::collector::page_collector::collect_page;
use crate::collector::parser::CustomSelectors;
use crate::collector::rewriter::{self, RewriteOptions};
use crate::collector::search_collector;
use crate::collector::synonym_dict;
use crate::db::models::{
    ArticleInput, ArticleRepo, CollectSource, DedupRepo, KeywordRepo, LogRepo, SettingsRepo, SourceRepo,
};
use crate::utils::hash::{fingerprint, sha256_hex};
use crate::AppState;

/// 采集进度事件
#[derive(Serialize, Clone)]
pub struct CollectProgress {
    pub stage: String,
    pub current: usize,
    pub total: usize,
    pub message: String,
}

/// 采集日志事件
#[derive(Serialize, Clone)]
pub struct CollectLogEvent {
    pub status: String,
    pub url: Option<String>,
    pub message: String,
}

#[derive(Serialize, Default)]
pub struct CollectSummary {
    pub found: usize,
    pub collected: usize,
    pub skipped: usize,
    pub failed: usize,
}

/// 从设置表读取抓取配置
pub(crate) fn load_fetch_settings(conn: &rusqlite::Connection) -> FetchSettings {
    let get = |k: &str| SettingsRepo::get(conn, k);
    FetchSettings {
        timeout: get("collect_timeout").and_then(|v| v.parse().ok()).unwrap_or(30),
        interval_min: get("collect_interval_min").and_then(|v| v.parse().ok()).unwrap_or(2),
        interval_max: get("collect_interval_max").and_then(|v| v.parse().ok()).unwrap_or(5),
        retries: get("collect_retries").and_then(|v| v.parse().ok()).unwrap_or(3),
        user_agent: get("collect_user_agent").unwrap_or_default(),
        proxy_url: get("proxy_url").unwrap_or_default(),
    }
}

/// 从设置表读取伪原创选项与自定义词库
fn load_rewrite_config(conn: &rusqlite::Connection) -> (RewriteOptions, Vec<Vec<String>>) {
    let get = |k: &str| SettingsRepo::get(conn, k);
    let options = RewriteOptions {
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
    };
    let custom = get("rewrite_custom_dict")
        .and_then(|v| synonym_dict::parse_custom_dict(&v).ok())
        .unwrap_or_default();
    (options, custom)
}

/// 单页 URL 采集
#[tauri::command]
pub async fn collect_single_page(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    url: String,
) -> Result<crate::db::models::Article, String> {
    if !url.starts_with("http") {
        return Err("请输入以 http/https 开头的有效 URL".to_string());
    }

    let (fetch_settings, rewrite_opts, custom_dict) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        (load_fetch_settings(&conn), load_rewrite_config(&conn).0, load_rewrite_config(&conn).1)
    };
    let fetcher = Fetcher::new(fetch_settings)?;

    emit_progress(&app, "抓取页面", 0, 1, &url);
    let (mut input, _) = collect_page(&fetcher, &url, None, &[]).await?;

    // 去重检查
    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        if is_duplicate(&conn, &input) {
            return Err(format!("文章已存在（URL 或内容重复）：{}", input.title));
        }
    }

    // 伪原创
    apply_rewrite_if_enabled(&mut input, &rewrite_opts, &custom_dict);

    let id = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let id = ArticleRepo::insert_if_new(&conn, &input)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "文章已存在".to_string())?;
        record_dedup(&conn, &input);
        LogRepo::insert(&conn, None, "manual", None, Some(&url), "success", Some("单页采集成功"))
            .map_err(|e| e.to_string())?;
        id
    };

    emit_progress(&app, "完成", 1, 1, "采集完成");
    emit_log(&app, "success", Some(&url), "采集成功");

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    ArticleRepo::get(&conn, id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "文章不存在".to_string())
}

/// 执行关键词采集任务
#[tauri::command]
pub async fn run_keyword_task(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    task_id: i64,
) -> Result<CollectSummary, String> {
    let (task, fetch_settings, rewrite_opts, custom_dict) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let task = KeywordRepo::get(&conn, task_id).map_err(|e| e.to_string())?;
        let (opts, dict) = load_rewrite_config(&conn);
        (task, load_fetch_settings(&conn), opts, dict)
    };
    let fetcher = Fetcher::new(fetch_settings)?;
    let mut summary = CollectSummary::default();

    // 1. 搜索引擎结果
    emit_progress(&app, "搜索引擎查询", 0, 1, &format!("正在通过 {} 搜索「{}」", task.search_engine, task.keyword));
    let results = search_collector::search(
        &fetcher,
        &task.search_engine,
        &task.keyword,
        task.max_pages as u64,
        task.site_limit.as_deref(),
    )
    .await?;
    summary.found = results.len();
    emit_progress(&app, "搜索完成", 1, 1, &format!("发现 {} 条结果", results.len()));

    // 2. 匹配过滤 + 逐篇采集
    let total = results.len();
    for (i, item) in results.into_iter().enumerate() {
        emit_progress(&app, "采集文章", i, total, &format!("{}（{}/{}）", item.title, i + 1, total));

        let matched = match task.match_mode.as_str() {
            "exact" => item.title.replace(' ', "").contains(&task.keyword.replace(' ', "")),
            _ => item.title.contains(task.keyword.as_str()),
        };
        if !matched {
            // content / title_or_content 模式：抓取后由正文判断
            if task.match_mode == "title" {
                summary.skipped += 1;
                emit_log(&app, "info", Some(&item.url), "标题不匹配，跳过");
                continue;
            }
        }

        match collect_page(&fetcher, &item.url, None, &[]).await {
            Ok((mut input, _)) => {
                // 全文匹配检查
                let content_match = match task.match_mode.as_str() {
                    "content" => input
                        .content_md
                        .as_deref()
                        .map(|c| c.contains(task.keyword.as_str()))
                        .unwrap_or(false),
                    "title_or_content" => {
                        input.title.contains(task.keyword.as_str())
                            || input
                                .content_md
                                .as_deref()
                                .map(|c| c.contains(task.keyword.as_str()))
                                .unwrap_or(false)
                    }
                    _ => true,
                };
                if !content_match {
                    summary.skipped += 1;
                    emit_log(&app, "info", Some(&item.url), "内容不匹配，跳过");
                    continue;
                }

                let duplicated = {
                    let conn = state.db.lock().map_err(|e| e.to_string())?;
                    is_duplicate(&conn, &input)
                };
                if duplicated {
                    summary.skipped += 1;
                    emit_log(&app, "info", Some(&item.url), "已存在（去重跳过）");
                    continue;
                }

                apply_rewrite_if_enabled(&mut input, &rewrite_opts, &custom_dict);

                let inserted = {
                    let conn = state.db.lock().map_err(|e| e.to_string())?;
                    let id = ArticleRepo::insert_if_new(&conn, &input).map_err(|e| e.to_string())?;
                    if id.is_some() {
                        record_dedup(&conn, &input);
                        LogRepo::insert(
                            &conn,
                            Some(task.id),
                            "keyword",
                            Some(&task.keyword),
                            Some(&item.url),
                            "success",
                            Some("采集入库"),
                        )
                        .ok();
                    }
                    id
                };

                if inserted.is_some() {
                    summary.collected += 1;
                    emit_log(&app, "success", Some(&item.url), &format!("已入库：{}", input.title));
                } else {
                    summary.skipped += 1;
                }
            }
            Err(e) => {
                summary.failed += 1;
                emit_log(&app, "failed", Some(&item.url), &e);
                let conn = state.db.lock().map_err(|e| e.to_string())?;
                LogRepo::insert(
                    &conn,
                    Some(task.id),
                    "keyword",
                    Some(&task.keyword),
                    Some(&item.url),
                    "failed",
                    Some(&e),
                )
                .ok();
            }
        }
    }

    // 3. 更新任务状态
    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        KeywordRepo::set_last_run(&conn, task.id).ok();
    }
    emit_progress(&app, "采集完成", total, total, "任务完成");

    Ok(summary)
}

/// 从采集源执行一次采集（RSS / 网页 / 站点地图）
#[tauri::command]
pub async fn collect_from_source(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    source_id: i64,
) -> Result<CollectSummary, String> {
    let (source, fetch_settings, rewrite_opts, custom_dict) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let source: CollectSource = SourceRepo::get(&conn, source_id).map_err(|e| e.to_string())?;
        let (opts, dict) = load_rewrite_config(&conn);
        (source, load_fetch_settings(&conn), opts, dict)
    };
    let fetcher = Fetcher::new(fetch_settings)?;
    let mut summary = CollectSummary::default();

    let selectors = CustomSelectors {
        title: source.selector_title.clone(),
        content: source.selector_content.clone(),
        author: source.selector_author.clone(),
        date: source.selector_date.clone(),
        tags: source.selector_tags.clone(),
        cover: source.selector_cover.clone(),
    };
    let remove_selectors = source.remove_selectors.clone().unwrap_or_default();

    // 发现待采集 URL 列表
    let urls: Vec<String> = match source.source_type.as_str() {
        "rss" | "api" => {
            emit_progress(&app, "获取订阅源", 0, 1, &source.url);
            match crate::collector::rss_collector::fetch_feed(&fetcher, &source.url).await {
                Ok(items) => items.into_iter().map(|i| i.link).collect(),
                Err(e) => {
                    log_collect(&state, None, "source", None, Some(&source.url), "failed", &e);
                    return Err(e);
                }
            }
        }
        "sitemap" => {
            emit_progress(&app, "获取站点地图", 0, 1, &source.url);
            match fetcher.fetch_text(&source.url).await {
                Ok(page) => {
                    let urls = crate::collector::rss_collector::parse_sitemap(&page.text);
                    // 限制单次采集数量，避免长时间任务
                    urls.into_iter().take(30).collect()
                }
                Err(e) => {
                    log_collect(&state, None, "source", None, Some(&source.url), "failed", &e);
                    return Err(e);
                }
            }
        }
        _ => vec![source.url.clone()],
    };

    summary.found = urls.len();
    let total = urls.len();

    for (i, url) in urls.into_iter().enumerate() {
        emit_progress(&app, "采集文章", i, total, &format!("（{}/{}）{}", i + 1, total, url));
        match collect_page(&fetcher, &url, Some(&selectors), &remove_selectors).await {
            Ok((mut input, _)) => {
                let duplicated = {
                    let conn = state.db.lock().map_err(|e| e.to_string())?;
                    is_duplicate(&conn, &input)
                };
                if duplicated {
                    summary.skipped += 1;
                    emit_log(&app, "info", Some(&url), "已存在（去重跳过）");
                    continue;
                }
                apply_rewrite_if_enabled(&mut input, &rewrite_opts, &custom_dict);
                let inserted = {
                    let conn = state.db.lock().map_err(|e| e.to_string())?;
                    let id = ArticleRepo::insert_if_new(&conn, &input).map_err(|e| e.to_string())?;
                    if id.is_some() {
                        record_dedup(&conn, &input);
                        LogRepo::insert(&conn, Some(source.id), "source", None, Some(&url), "success", Some("采集入库")).ok();
                    }
                    id
                };
                if inserted.is_some() {
                    summary.collected += 1;
                    emit_log(&app, "success", Some(&url), &format!("已入库：{}", input.title));
                } else {
                    summary.skipped += 1;
                }
            }
            Err(e) => {
                summary.failed += 1;
                emit_log(&app, "failed", Some(&url), &e);
                let conn = state.db.lock().map_err(|e| e.to_string())?;
                LogRepo::insert(&conn, Some(source.id), "source", None, Some(&url), "failed", Some(&e)).ok();
            }
        }
    }

    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        SourceRepo::set_last_collected(&conn, source.id).ok();
    }
    emit_progress(&app, "采集完成", total, total, "采集源任务完成");

    Ok(summary)
}

/// 从 sitemap.xml 发现链接（不采集）
#[tauri::command]
pub async fn discover_from_sitemap(state: State<'_, AppState>, url: String) -> Result<Vec<String>, String> {
    let fetch_settings = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        load_fetch_settings(&conn)
    };
    let fetcher = Fetcher::new(fetch_settings)?;
    let page = fetcher.fetch_text(&url).await?;
    Ok(crate::collector::rss_collector::parse_sitemap(&page.text))
}

// ── 工具函数 ────────────────────────────────────────────

/// 三重去重：URL 哈希 + 标题相似度（≥0.85 视为重复）+ 内容指纹
fn is_duplicate(conn: &rusqlite::Connection, input: &ArticleInput) -> bool {
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
    if !DedupRepo::find_similar_title(conn, &input.title, 0.85).is_empty() {
        return true;
    }
    false
}

/// 记录去重缓存
fn record_dedup(conn: &rusqlite::Connection, input: &ArticleInput) {
    let url_hash = sha256_hex(&input.url);
    let title_hash = sha256_hex(&input.title);
    let fp = input
        .content_md
        .as_deref()
        .map(fingerprint)
        .unwrap_or_default();
    DedupRepo::insert(conn, &url_hash, &title_hash, &fp).ok();
}

/// 采集时按设置自动应用伪原创
fn apply_rewrite_if_enabled(input: &mut ArticleInput, opts: &RewriteOptions, custom_dict: &[Vec<String>]) {
    if !opts.enabled {
        return;
    }
    if let Some(md) = input.content_md.as_ref() {
        let rewritten = rewriter::rewrite_markdown(md, opts, custom_dict);
        input.content_md = Some(rewritten);
        input.excerpt = Some(crate::collector::page_collector::extract_excerpt(
            input.content_md.as_deref().unwrap_or(""),
        ));
    }
}

fn log_collect(
    state: &State<'_, AppState>,
    task_id: Option<i64>,
    task_type: &str,
    keyword: Option<&str>,
    url: Option<&str>,
    status: &str,
    message: &str,
) {
    if let Ok(conn) = state.db.lock() {
        LogRepo::insert(&conn, task_id, task_type, keyword, url, status, Some(message)).ok();
    }
}

fn emit_progress(app: &tauri::AppHandle, stage: &str, current: usize, total: usize, message: &str) {
    let _ = app.emit(
        "collect:progress",
        CollectProgress {
            stage: stage.to_string(),
            current,
            total,
            message: message.to_string(),
        },
    );
}

fn emit_log(app: &tauri::AppHandle, status: &str, url: Option<&str>, message: &str) {
    let _ = app.emit(
        "collect:log",
        CollectLogEvent {
            status: status.to_string(),
            url: url.map(|s| s.to_string()),
            message: message.to_string(),
        },
    );
}

/// 占位：CleanOptions 在 source 采集中通过 remove_selectors 传入
#[allow(dead_code)]
fn _clean_options(remove: &[String]) -> CleanOptions {
    CleanOptions {
        remove_selectors: remove.to_vec(),
        keep_images: true,
    }
}
