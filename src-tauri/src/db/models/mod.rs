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
    use rusqlite::{params, Connection};

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

        // 模拟采集时写入的去重缓存
        let url_hash = sha256_hex(&input.url);
        let fp = fingerprint(input.content_md.as_deref().unwrap());
        DedupRepo::insert(&conn, &url_hash, &sha256_hex(&input.title), &fp).unwrap();
        assert!(DedupRepo::url_exists(&conn, &url_hash));

        let (list, total) = ArticleRepo::list(&conn, &ArticleQuery::default()).unwrap();
        assert_eq!(total, 1);
        assert_eq!(list[0].id, id);
        assert!(list[0].content_html.is_none());

        ArticleRepo::delete(&conn, &[id]).unwrap();
        assert!(ArticleRepo::get(&conn, id).unwrap().is_none());
        // 删除文章后应同步清理去重缓存，允许重新采集
        assert!(!DedupRepo::url_exists(&conn, &url_hash));
        assert!(!DedupRepo::fingerprint_exists(&conn, &fp));
    }

    #[test]
    fn update_keeps_url_when_input_empty() {
        let conn = setup();
        let input = ArticleInput {
            title: "原标题".to_string(),
            url: "https://example.com/original".to_string(),
            ..Default::default()
        };
        let id = ArticleRepo::insert_if_new(&conn, &input).unwrap().unwrap();

        // 编辑表单未回传 url 时，不应清空已有地址
        let edit = ArticleInput {
            title: "新标题".to_string(),
            url: String::new(),
            ..Default::default()
        };
        ArticleRepo::update(&conn, id, &edit).unwrap();
        let article = ArticleRepo::get(&conn, id).unwrap().unwrap();
        assert_eq!(article.title, "新标题");
        assert_eq!(article.url, "https://example.com/original");
        assert_eq!(article.status, "edited");

        // 传入新 url 时正常更新
        let edit_with_url = ArticleInput {
            title: "新标题".to_string(),
            url: "https://example.com/new".to_string(),
            ..Default::default()
        };
        ArticleRepo::update(&conn, id, &edit_with_url).unwrap();
        assert_eq!(
            ArticleRepo::get(&conn, id).unwrap().unwrap().url,
            "https://example.com/new"
        );
    }

    #[test]
    fn cms_password_is_encrypted_and_legacy_data_migrates() {
        let key_dir = std::env::temp_dir().join(format!("x-collector-key-test-{}", std::process::id()));
        std::fs::create_dir_all(&key_dir).unwrap();
        crate::utils::crypto::init(&key_dir).unwrap();

        let conn = setup();
        let input = CmsConnectionInput {
            name: "站点".to_string(),
            site_url: "https://example.com".to_string(),
            username: "admin".to_string(),
            password: Some("secret123".to_string()),
            ..Default::default()
        };
        let saved = CmsRepo::save(&conn, &input).unwrap();

        let stored: String = conn
            .query_row(
                "SELECT password FROM cms_connections WHERE id = ?1",
                params![saved.id],
                |r| r.get(0),
            )
            .unwrap();
        assert!(stored.starts_with("v2:"), "密码应以 AES-GCM 格式存储");
        assert_eq!(CmsRepo::get_password(&conn, saved.id).unwrap(), "secret123");

        // 旧版 XOR 数据：读取时应能解密并自动迁移
        const LEGACY_KEY: &[u8] = b"x-collector-maple-bell-2026";
        let legacy = hex::encode(
            "secret123"
                .bytes()
                .enumerate()
                .map(|(i, b)| b ^ LEGACY_KEY[i % LEGACY_KEY.len()])
                .collect::<Vec<_>>(),
        );
        conn.execute(
            "UPDATE cms_connections SET password = ?1 WHERE id = ?2",
            params![legacy, saved.id],
        )
        .unwrap();
        assert_eq!(CmsRepo::get_password(&conn, saved.id).unwrap(), "secret123");
        let migrated: String = conn
            .query_row(
                "SELECT password FROM cms_connections WHERE id = ?1",
                params![saved.id],
                |r| r.get(0),
            )
            .unwrap();
        assert!(migrated.starts_with("v2:"));
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
