use tauri::State;

use crate::app::config::{CollectSettings, RewriteConfig};
use crate::collector::engine::Fetcher;
use crate::collector::parser::CustomSelectors;
use crate::collector::search_collector;
use crate::db::models::{ArticleRepo, CollectSource, KeywordRepo, SourceRepo};
use crate::events::{emit_log, emit_progress, CollectSummary};
use crate::services::collect as collect_service;
use crate::AppState;

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

    let (collect_settings, rewrite_config) = {
        let conn = state.conn()?;
        (CollectSettings::load(&conn), RewriteConfig::load(&conn))
    };
    let fetcher = Fetcher::new(collect_settings.to_fetch_settings())?;

    emit_progress(&app, "抓取页面", 0, 1, &url);
    let mut input = collect_service::fetch_page(&fetcher, &url, None, &[]).await?;

    let duplicate = {
        let conn = state.conn()?;
        collect_service::is_duplicate(&conn, &input)
    };
    if duplicate {
        return Err(format!("文章已存在（URL 或内容重复）：{}", input.title));
    }

    // 伪原创
    collect_service::apply_rewrite(&mut input, &rewrite_config);
    if collect_settings.localize_images {
        collect_service::localize_images(&mut input, &fetcher).await;
    }

    let id = {
        let conn = state.conn()?;
        let id = collect_service::save_article(&conn, &input)?
            .ok_or_else(|| "文章已存在".to_string())?;
        collect_service::log(&conn, None, "manual", None, Some(&url), "success", "单页采集成功");
        id
    };

    emit_progress(&app, "完成", 1, 1, "采集完成");
    emit_log(&app, "success", Some(&url), "采集成功");

    let conn = state.conn()?;
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
    let (task, collect_settings, rewrite_config) = {
        let conn = state.conn()?;
        let task = KeywordRepo::get(&conn, task_id).map_err(|e| e.to_string())?;
        (
            task,
            CollectSettings::load(&conn),
            RewriteConfig::load(&conn),
        )
    };
    let fetcher = Fetcher::new(collect_settings.to_fetch_settings())?;
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

        match collect_service::fetch_page(&fetcher, &item.url, None, &[]).await {
            Ok(mut input) => {
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
                    let conn = state.conn()?;
                    collect_service::is_duplicate(&conn, &input)
                };
                if duplicated {
                    summary.skipped += 1;
                    emit_log(&app, "info", Some(&item.url), "已存在（去重跳过）");
                    continue;
                }

                collect_service::apply_rewrite(&mut input, &rewrite_config);
                if collect_settings.localize_images {
                    collect_service::localize_images(&mut input, &fetcher).await;
                }

                let inserted = {
                    let conn = state.conn()?;
                    let id = collect_service::save_article(&conn, &input)?;
                    if id.is_some() {
                        collect_service::log(
                            &conn,
                            Some(task.id),
                            "keyword",
                            Some(&task.keyword),
                            Some(&item.url),
                            "success",
                            "采集入库",
                        );
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
                let conn = state.conn()?;
                collect_service::log(
                    &conn,
                    Some(task.id),
                    "keyword",
                    Some(&task.keyword),
                    Some(&item.url),
                    "failed",
                    &e,
                );
            }
        }
    }

    // 3. 更新任务状态
    {
        let conn = state.conn()?;
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
    let (source, collect_settings, rewrite_config) = {
        let conn = state.conn()?;
        let source: CollectSource = SourceRepo::get(&conn, source_id).map_err(|e| e.to_string())?;
        (
            source,
            CollectSettings::load(&conn),
            RewriteConfig::load(&conn),
        )
    };
    let fetcher = Fetcher::new(collect_settings.to_fetch_settings())?;
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
        match collect_service::fetch_page(&fetcher, &url, Some(&selectors), &remove_selectors).await {
            Ok(mut input) => {
                let duplicated = {
                    let conn = state.conn()?;
                    collect_service::is_duplicate(&conn, &input)
                };
                if duplicated {
                    summary.skipped += 1;
                    emit_log(&app, "info", Some(&url), "已存在（去重跳过）");
                    continue;
                }
                collect_service::apply_rewrite(&mut input, &rewrite_config);
                if collect_settings.localize_images {
                    collect_service::localize_images(&mut input, &fetcher).await;
                }
                let inserted = {
                    let conn = state.conn()?;
                    let id = collect_service::save_article(&conn, &input)?;
                    if id.is_some() {
                        collect_service::log(
                            &conn,
                            Some(source.id),
                            "source",
                            None,
                            Some(&url),
                            "success",
                            "采集入库",
                        );
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
                let conn = state.conn()?;
                collect_service::log(
                    &conn,
                    Some(source.id),
                    "source",
                    None,
                    Some(&url),
                    "failed",
                    &e,
                );
            }
        }
    }

    {
        let conn = state.conn()?;
        SourceRepo::set_last_collected(&conn, source.id).ok();
    }
    emit_progress(&app, "采集完成", total, total, "采集源任务完成");

    Ok(summary)
}

/// 从 sitemap.xml 发现链接（不采集）
#[tauri::command]
pub async fn discover_from_sitemap(state: State<'_, AppState>, url: String) -> Result<Vec<String>, String> {
    let collect_settings = {
        let conn = state.conn()?;
        CollectSettings::load(&conn)
    };
    let fetcher = Fetcher::new(collect_settings.to_fetch_settings())?;
    let page = fetcher.fetch_text(&url).await?;
    Ok(crate::collector::rss_collector::parse_sitemap(&page.text))
}

/// 采集失败时写日志（忽略加锁与写入错误）
fn log_collect(
    state: &State<'_, AppState>,
    task_id: Option<i64>,
    task_type: &str,
    keyword: Option<&str>,
    url: Option<&str>,
    status: &str,
    message: &str,
) {
    if let Ok(conn) = state.conn() {
        collect_service::log(&conn, task_id, task_type, keyword, url, status, message);
    }
}
