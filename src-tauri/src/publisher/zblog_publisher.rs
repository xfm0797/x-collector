use crate::db::models::{Article, CmsConnection};

use super::cms_client::{self, RpcValue};

/// Z-Blog XML-RPC 自动发布（metaWeblog.newPost）
pub async fn publish(
    conn: &CmsConnection,
    password: &str,
    article: &Article,
) -> Result<(String, String), String> {
    let endpoint = format!(
        "{}{}",
        conn.site_url.trim_end_matches('/'),
        conn.api_path.as_deref().unwrap_or("/zb_system/xml-rpc/index.php")
    );

    let publish_now = conn.default_status == "publish";
    let content_html = build_html_content(article);

    let post_struct = RpcValue::Struct(vec![
        ("title".into(), RpcValue::Str(article.title.clone())),
        ("description".into(), RpcValue::Str(content_html)),
        (
            "dateCreated".into(),
            RpcValue::Str(article.published_at.clone().unwrap_or_else(|| {
                chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string()
            })),
        ),
        (
            "categories".into(),
            RpcValue::Array(
                conn.default_category
                    .as_deref()
                    .filter(|c| !c.is_empty())
                    .map(|c| vec![RpcValue::Str(c.to_string())])
                    .unwrap_or_default(),
            ),
        ),
    ]);

    let body = cms_client::build_request(
        "metaWeblog.newPost",
        &[
            RpcValue::Str("1".to_string()),
            RpcValue::Str(conn.username.clone()),
            RpcValue::Str(password.to_string()),
            post_struct,
            RpcValue::Bool(publish_now),
        ],
    );

    let response = cms_client::call(&endpoint, &conn.username, password, &body).await?;

    if let Some(fault) = cms_client::extract_fault(&response) {
        return Err(format!("Z-Blog 返回错误：{fault}"));
    }

    let post_id = cms_client::extract_value(&response).unwrap_or_default();
    let remote_url = format!("{}/post/{}.html", conn.site_url.trim_end_matches('/'), post_id);
    Ok((post_id, remote_url))
}

/// 测试 Z-Blog 连接
pub async fn test_connection(conn: &CmsConnection, password: &str) -> Result<String, String> {
    let endpoint = format!(
        "{}{}",
        conn.site_url.trim_end_matches('/'),
        conn.api_path.as_deref().unwrap_or("/zb_system/xml-rpc/index.php")
    );
    let body = cms_client::build_request(
        "blogger.getUsersBlogs",
        &[
            RpcValue::Str("".to_string()),
            RpcValue::Str(conn.username.clone()),
            RpcValue::Str(password.to_string()),
        ],
    );
    let response = cms_client::call(&endpoint, &conn.username, password, &body).await?;
    if let Some(fault) = cms_client::extract_fault(&response) {
        return Err(format!("连接失败：{fault}"));
    }
    Ok("连接成功".to_string())
}

fn build_html_content(article: &Article) -> String {
    let md = article.content_md.clone().unwrap_or_default();
    let html = super::markdown_to_html(&md);
    let source_note = if article.url.starts_with("http") {
        format!(
            "<hr/><p><small>原文来源：<a href=\"{}\" target=\"_blank\">{}</a></small></p>",
            article.url,
            article.source.as_deref().unwrap_or("原文链接")
        )
    } else {
        String::new()
    };
    format!("{}{}", html, source_note)
}
