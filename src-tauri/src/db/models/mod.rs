//! 数据访问层：按实体拆分的仓储，统一由本模块对外导出

pub mod article;
pub mod cms;
pub mod dedup;
pub mod keyword;
pub mod log;
pub mod publish;
pub mod settings;
pub mod source;

pub use article::{Article, ArticleInput, ArticleQuery, ArticleRepo};
pub use cms::{CmsConnection, CmsConnectionInput, CmsRepo};
pub use dedup::DedupRepo;
pub use keyword::{KeywordRepo, KeywordTask, KeywordTaskInput};
pub use log::{CollectLog, LogRepo};
pub use publish::{PublishRecord, PublishRepo};
pub use settings::SettingsRepo;
pub use source::{CollectSource, CollectSourceInput, SourceRepo};

pub(crate) fn now() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

pub(crate) fn parse_json_list<T: serde::de::DeserializeOwned>(text: Option<String>) -> Option<Vec<T>> {
    text.and_then(|s| serde_json::from_str(&s).ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::hash::{fingerprint, sha256_hex};
    use rusqlite::Connection;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&conn).unwrap();
        conn
    }

    #[test]
    fn article_insert_list_delete_flow() {
        let conn = setup();
        let input = ArticleInput {
            title: "Rust 采集实践".to_string(),
            url: "https://example.com/rust".to_string(),
            content_md: Some("正文内容，用于验证列表查询不加载 HTML。".to_string()),
            ..Default::default()
        };

        let id = ArticleRepo::insert_if_new(&conn, &input).unwrap().unwrap();
        // URL 唯一约束：重复插入返回 None
        assert!(ArticleRepo::insert_if_new(&conn, &input).unwrap().is_none());

        let (list, total) = ArticleRepo::list(&conn, &ArticleQuery::default()).unwrap();
        assert_eq!(total, 1);
        assert_eq!(list[0].id, id);
        assert!(list[0].content_html.is_none());

        ArticleRepo::delete(&conn, &[id]).unwrap();
        assert!(ArticleRepo::get(&conn, id).unwrap().is_none());
    }

    #[test]
    fn dedup_cache_flow() {
        let conn = setup();
        let url_hash = sha256_hex("https://example.com/a");
        let fp = fingerprint("一段足够长的正文内容，用于生成内容指纹。");

        assert!(!DedupRepo::url_exists(&conn, &url_hash));
        DedupRepo::insert(&conn, &url_hash, &sha256_hex("标题"), &fp).unwrap();
        assert!(DedupRepo::url_exists(&conn, &url_hash));
        assert!(DedupRepo::fingerprint_exists(&conn, &fp));
    }
}
