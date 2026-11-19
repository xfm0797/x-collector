use rusqlite::Connection;
use serde::Serialize;

use crate::collector::engine::FetchSettings;
use crate::collector::rewriter::RewriteOptions;
use crate::collector::synonym_dict;
use crate::db::models::SettingsRepo;

/// 采集相关设置（向后端命令与采集服务共享）
#[derive(Debug, Clone, Serialize)]
pub struct CollectSettings {
    pub timeout: u64,
    pub concurrency: u64,
    pub interval_min: u64,
    pub interval_max: u64,
    pub retries: u32,
    pub user_agent: String,
    pub proxy_url: String,
    /// 是否把正文图片下载后内嵌为 data URI
    pub localize_images: bool,
}

impl CollectSettings {
    pub fn load(conn: &Connection) -> Self {
        let get = |k: &str| SettingsRepo::get(conn, k);
        Self {
            timeout: get("collect_timeout").and_then(|v| v.parse().ok()).unwrap_or(30),
            concurrency: get("collect_concurrency").and_then(|v| v.parse().ok()).unwrap_or(3),
            interval_min: get("collect_interval_min").and_then(|v| v.parse().ok()).unwrap_or(2),
            interval_max: get("collect_interval_max").and_then(|v| v.parse().ok()).unwrap_or(5),
            retries: get("collect_retries").and_then(|v| v.parse().ok()).unwrap_or(3),
            user_agent: get("collect_user_agent").unwrap_or_default(),
            proxy_url: get("proxy_url").unwrap_or_default(),
            localize_images: get("collect_localize_images")
                .map(|v| v == "true")
                .unwrap_or(false),
        }
    }

    pub fn to_fetch_settings(&self) -> FetchSettings {
        FetchSettings {
            timeout: self.timeout,
            interval_min: self.interval_min,
            interval_max: self.interval_max,
            retries: self.retries,
            user_agent: self.user_agent.clone(),
            proxy_url: self.proxy_url.clone(),
        }
    }
}

/// 伪原创配置：选项 + 自定义词库
#[derive(Debug, Clone)]
pub struct RewriteConfig {
    pub options: RewriteOptions,
    pub custom_dict: Vec<Vec<String>>,
}

impl RewriteConfig {
    pub fn load(conn: &Connection) -> Self {
        let get = |k: &str| SettingsRepo::get(conn, k);
        let options = RewriteOptions {
            enabled: get("rewrite_enabled").map(|v| v == "true").unwrap_or(false),
            intensity: get("rewrite_intensity").unwrap_or_else(|| "medium".to_string()),
            synonym_ratio: get("rewrite_synonym_ratio").and_then(|v| v.parse().ok()).unwrap_or(30),
            sentence_ratio: get("rewrite_sentence_ratio").and_then(|v| v.parse().ok()).unwrap_or(20),
            paragraph_shuffle: get("rewrite_paragraph_shuffle")
                .map(|v| v == "true")
                .unwrap_or(false),
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
        let custom_dict = get("rewrite_custom_dict")
            .and_then(|v| synonym_dict::parse_custom_dict(&v).ok())
            .unwrap_or_default();
        Self {
            options,
            custom_dict,
        }
    }
}
