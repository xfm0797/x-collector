use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::models::{
    ArticleRepo, CmsConnection, CmsConnectionInput, CmsRepo, PublishRecord, PublishRepo, SettingsRepo,
};
use crate::publisher::{hexo_exporter, typecho_publisher, wordpress_publisher, zblog_publisher};
use crate::AppState;

/// 发布目标
#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct PublishTarget {
    pub publish_type: String,
    pub cms_id: Option<i64>,
    /// Hexo 导出目录（留空使用设置默认值）
    pub output_dir: Option<String>,
}

/// CMS 连接列表
#[tauri::command]
pub fn list_cms_connections(state: State<'_, AppState>) -> Result<Vec<CmsConnection>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    CmsRepo::list(&conn).map_err(|e| e.to_string())
}

/// 保存（新增/更新）CMS 连接
#[tauri::command]
pub fn save_cms_connection(state: State<'_, AppState>, input: CmsConnectionInput) -> Result<CmsConnection, String> {
    if input.name.trim().is_empty() || input.site_url.trim().is_empty() || input.username.trim().is_empty() {
        return Err("连接名称、站点地址与用户名不能为空".to_string());
    }
    if input.id.is_none() && input.password.as_deref().unwrap_or("").is_empty() {
        return Err("请填写密码".to_string());
    }
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    CmsRepo::save(&conn, &input).map_err(|e| e.to_string())
}

/// 删除 CMS 连接
#[tauri::command]
pub fn delete_cms_connections(state: State<'_, AppState>, ids: Vec<i64>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    CmsRepo::delete(&conn, &ids).map_err(|e| e.to_string())
}

/// 测试 CMS 连接（XML-RPC 可达性与凭证校验）
#[tauri::command]
pub async fn test_cms_connection(state: State<'_, AppState>, cms_id: i64) -> Result<TestResult, String> {
    let (cms, password) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let cms = CmsRepo::get(&conn, cms_id).map_err(|e| e.to_string())?;
        let password = CmsRepo::get_password(&conn, cms_id).map_err(|e| e.to_string())?;
        (cms, password)
    };

    let result = match cms.cms_type.as_str() {
        "wordpress" => wordpress_publisher::test_connection(&cms, &password).await,
        "typecho" => typecho_publisher::test_connection(&cms, &password).await,
        "zblog" => zblog_publisher::test_connection(&cms, &password).await,
        _ => Err("自定义 CMS 暂不支持连接测试".to_string()),
    };

    match result {
        Ok(message) => Ok(TestResult {
            success: true,
            message,
        }),
        Err(e) => Ok(TestResult {
            success: false,
            message: e,
        }),
    }
}

#[derive(Serialize)]
pub struct TestResult {
    pub success: bool,
    pub message: String,
}

/// 单篇发布
#[tauri::command]
pub async fn publish_article(
    state: State<'_, AppState>,
    article_id: i64,
    target: PublishTarget,
) -> Result<PublishRecord, String> {
    let records = publish_batch_inner(state, &[article_id], &target).await?;
    records.into_iter().next().ok_or_else(|| "发布失败：未知错误".to_string())
}

/// 批量发布
#[tauri::command]
pub async fn batch_publish(
    state: State<'_, AppState>,
    article_ids: Vec<i64>,
    target: PublishTarget,
) -> Result<Vec<PublishRecord>, String> {
    publish_batch_inner(state, &article_ids, &target).await
}

async fn publish_batch_inner(
    state: State<'_, AppState>,
    article_ids: &[i64],
    target: &PublishTarget,
) -> Result<Vec<PublishRecord>, String> {
    let mut records = Vec::new();

    for &article_id in article_ids {
        let (article, cms, password, default_hexo_dir) = {
            let conn = state.db.lock().map_err(|e| e.to_string())?;
            let article = ArticleRepo::get(&conn, article_id)
                .map_err(|e| e.to_string())?
                .ok_or_else(|| format!("文章 {} 不存在", article_id))?;
            let cms = match target.cms_id {
                Some(id) => Some(CmsRepo::get(&conn, id).map_err(|e| e.to_string())?),
                None => None,
            };
            let password = match target.cms_id {
                Some(id) => CmsRepo::get_password(&conn, id).map_err(|e| e.to_string())?,
                None => String::new(),
            };
            let hexo_dir = SettingsRepo::get(&conn, "hexo_output_dir").unwrap_or_default();
            (article, cms, password, hexo_dir)
        };

        let publish_type = target.publish_type.clone();

        match publish_type.as_str() {
            "hexo" => {
                let output_dir = target
                    .output_dir
                    .clone()
                    .filter(|d| !d.trim().is_empty())
                    .unwrap_or(default_hexo_dir);
                if output_dir.trim().is_empty() {
                    let record = record_failure(&state, article_id, None, "hexo", "未配置 Hexo 导出目录，请先在发布弹窗填写或在设置中配置默认目录").await;
                    records.push(record);
                    continue;
                }
                let opts = hexo_exporter::HexoOptions {
                    output_dir,
                    default_category: "未分类".to_string(),
                    include_source: true,
                };
                match hexo_exporter::export_article(&article, &opts) {
                    Ok(path) => {
                        let record = record_success(&state, article_id, None, "hexo", "", &path).await;
                        records.push(record);
                    }
                    Err(e) => {
                        let record = record_failure(&state, article_id, None, "hexo", &e).await;
                        records.push(record);
                    }
                }
            }
            "wordpress" | "typecho" | "zblog" | "custom" => {
                let Some(cms) = cms.as_ref() else {
                    let record = record_failure(&state, article_id, None, &publish_type, "未找到 CMS 连接").await;
                    records.push(record);
                    continue;
                };
                let cms_type = if publish_type == "custom" { cms.cms_type.clone() } else { publish_type.clone() };
                let result = match cms_type.as_str() {
                    "wordpress" => wordpress_publisher::publish(cms, &password, &article).await,
                    "typecho" => typecho_publisher::publish(cms, &password, &article).await,
                    "zblog" => zblog_publisher::publish(cms, &password, &article).await,
                    _ => Err("自定义 CMS 发布暂未实现".to_string()),
                };
                match result {
                    Ok((remote_id, remote_url)) => {
                        let record = record_success(&state, article_id, Some(cms.id), &cms_type, &remote_id, &remote_url).await;
                        records.push(record);
                    }
                    Err(e) => {
                        let record = record_failure(&state, article_id, Some(cms.id), &cms_type, &e).await;
                        records.push(record);
                    }
                }
            }
            _ => {
                let record = record_failure(&state, article_id, None, &publish_type, "未知的发布方式").await;
                records.push(record);
            }
        }
    }

    Ok(records)
}

async fn record_success(
    state: &State<'_, AppState>,
    article_id: i64,
    cms_id: Option<i64>,
    publish_type: &str,
    remote_id: &str,
    remote_url: &str,
) -> PublishRecord {
    let conn = state.db.lock().map_err(|e| e.to_string()).unwrap();
    ArticleRepo::update_status(&conn, article_id, "published").ok();
    if let Some(id) = cms_id {
        CmsRepo::set_last_used(&conn, id).ok();
    }
    let record_id = PublishRepo::insert(
        &conn,
        article_id,
        cms_id,
        publish_type,
        "success",
        Some(remote_id),
        Some(remote_url),
        None,
    )
    .unwrap_or(0);
    build_record(&conn, record_id)
}

async fn record_failure(
    state: &State<'_, AppState>,
    article_id: i64,
    cms_id: Option<i64>,
    publish_type: &str,
    error: &str,
) -> PublishRecord {
    let conn = state.db.lock().map_err(|e| e.to_string()).unwrap();
    ArticleRepo::update_status(&conn, article_id, "failed").ok();
    let record_id = PublishRepo::insert(&conn, article_id, cms_id, publish_type, "failed", None, None, Some(error))
        .unwrap_or(0);
    build_record(&conn, record_id)
}

fn build_record(conn: &rusqlite::Connection, record_id: i64) -> PublishRecord {
    PublishRepo::list(conn, 1000)
        .map(|records| {
            records
                .into_iter()
                .find(|r| r.id == record_id)
                .unwrap_or(PublishRecord {
                    id: record_id,
                    article_id: 0,
                    article_title: None,
                    cms_id: None,
                    cms_name: None,
                    publish_type: String::new(),
                    status: "failed".to_string(),
                    remote_id: None,
                    remote_url: None,
                    error_message: None,
                    published_at: None,
                    created_at: None,
                })
        })
        .unwrap_or(PublishRecord {
            id: record_id,
            article_id: 0,
            article_title: None,
            cms_id: None,
            cms_name: None,
            publish_type: String::new(),
            status: "failed".to_string(),
            remote_id: None,
            remote_url: None,
            error_message: None,
            published_at: None,
            created_at: None,
        })
}

/// 批量导出 Hexo（独立命令，不写发布记录）
#[tauri::command]
pub fn export_hexo(
    state: State<'_, AppState>,
    article_ids: Vec<i64>,
    output_dir: Option<String>,
) -> Result<HexoExportResult, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let default_dir = SettingsRepo::get(&conn, "hexo_output_dir").unwrap_or_default();
    let dir = output_dir.filter(|d| !d.trim().is_empty()).unwrap_or(default_dir);
    if dir.trim().is_empty() {
        return Err("未配置 Hexo 导出目录".to_string());
    }

    let opts = hexo_exporter::HexoOptions {
        output_dir: dir,
        default_category: "未分类".to_string(),
        include_source: true,
    };

    let mut files = Vec::new();
    let mut failed = 0;
    for id in article_ids {
        let article = match ArticleRepo::get(&conn, id).map_err(|e| e.to_string())? {
            Some(a) => a,
            None => {
                failed += 1;
                continue;
            }
        };
        match hexo_exporter::export_article(&article, &opts) {
            Ok(path) => files.push(path),
            Err(_) => failed += 1,
        }
    }

    Ok(HexoExportResult { files, failed })
}

#[derive(Serialize)]
pub struct HexoExportResult {
    pub files: Vec<String>,
    pub failed: usize,
}

/// 发布记录列表
#[tauri::command]
pub fn list_publish_records(state: State<'_, AppState>, limit: Option<i64>) -> Result<Vec<PublishRecord>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    PublishRepo::list(&conn, limit.unwrap_or(100)).map_err(|e| e.to_string())
}
