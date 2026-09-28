use crate::db::models::{Article, CmsConnection};

use super::cms_client::{self, RpcValue};

/// WordPress XML-RPC 自动发布（metaWeblog.newPost）
pub async fn publish(
    conn: &CmsConnection,
    password: &str,
    article: &Article,
) -> Result<(String, String), String> {
    let endpoint = format!(
        "{}{}",
        conn.site_url.trim_end_matches('/'),
        conn.api_path.as_deref().unwrap_or("/xmlrpc.php")
    );

    let publish_now = conn.default_status == "publish";
    let content_html = build_html_content(article);

    let post_struct = RpcValue::Struct(vec![
        ("title".into(), RpcValue::Str(article.title.clone())),
        ("description".into(), RpcValue::Str(content_html)),
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
        (
            "mt_keywords".into(),
            RpcValue::Str(article.tags.clone().unwrap_or_default().join(",")),
        ),
        (
            "dateCreated".into(),
            RpcValue::Str(article.published_at.clone().unwrap_or_else(|| {
                chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string()
            })),
        ),
    ]);

    let body = cms_client::build_request(
        "metaWeblog.newPost",
        &[
            RpcValue::Str("1".to_string()), // blog id
            RpcValue::Str(conn.username.clone()),
            RpcValue::Str(password.to_string()),
            post_struct,
            RpcValue::Bool(publish_now),
        ],
    );

    let response = cms_client::call(&endpoint, &conn.username, password, &body).await?;

    if let Some(fault) = cms_client::extract_fault(&response) {
        return Err(format!("WordPress 返回错误：{fault}"));
    }

    let post_id = cms_client::extract_value(&response).unwrap_or_default();
    let remote_url = if post_id.is_empty() {
        format!("{}/?p={}", conn.site_url.trim_end_matches('/'), "")
    } else {
        format!("{}/?p={}", conn.site_url.trim_end_matches('/'), post_id)
    };

    Ok((post_id, remote_url))
}

/// 测试 WordPress 连接：blogger.getUsersBlogs
pub async fn test_connection(conn: &CmsConnection, password: &str) -> Result<String, String> {
    let endpoint = format!(
        "{}{}",
        conn.site_url.trim_end_matches('/'),
        conn.api_path.as_deref().unwrap_or("/xmlrpc.php")
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
    let blog_name = cms_client::extract_struct_string(&response, "blogName")
        .or_else(|| cms_client::extract_struct_string(&response, "url"))
        .unwrap_or_else(|| "连接成功".to_string());
    Ok(format!("连接成功：{blog_name}"))
}

/// 组装 HTML 正文（附来源声明）
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
