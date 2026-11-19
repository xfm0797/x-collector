pub mod cms_client;
pub mod hexo_exporter;
pub mod typecho_publisher;
pub mod wordpress_publisher;
pub mod zblog_publisher;

use crate::db::models::{Article, CmsConnection};

/// CMS 发布器注册表：按 CMS 类型分发到具体实现
pub async fn publish(
    cms_type: &str,
    conn: &CmsConnection,
    password: &str,
    article: &Article,
) -> Result<(String, String), String> {
    match cms_type {
        "wordpress" => wordpress_publisher::publish(conn, password, article).await,
        "typecho" => typecho_publisher::publish(conn, password, article).await,
        "zblog" => zblog_publisher::publish(conn, password, article).await,
        _ => Err("自定义 CMS 发布暂未实现".to_string()),
    }
}

/// CMS 连接测试注册表
pub async fn test_connection(
    cms_type: &str,
    conn: &CmsConnection,
    password: &str,
) -> Result<String, String> {
    match cms_type {
        "wordpress" => wordpress_publisher::test_connection(conn, password).await,
        "typecho" => typecho_publisher::test_connection(conn, password).await,
        "zblog" => zblog_publisher::test_connection(conn, password).await,
        _ => Err("自定义 CMS 暂不支持连接测试".to_string()),
    }
}

/// Markdown → HTML 转换（用于 CMS 发布正文）
///
/// 使用 CommonMark 解析器，原始 HTML 默认被转义，避免正文注入。
/// 支持表格、删除线与任务列表。
pub fn markdown_to_html(md: &str) -> String {
    use pulldown_cmark::{html, Event, Options, Parser};

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    // 原始 HTML 作为纯文本输出，由渲染器统一转义
    let parser = Parser::new_ext(md, options).map(|event| match event {
        Event::Html(text) | Event::InlineHtml(text) => Event::Text(text),
        other => other,
    });
    let mut html_output = String::with_capacity(md.len() * 3 / 2);
    html::push_html(&mut html_output, parser);
    html_output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_md_to_html() {
        let html = markdown_to_html("# 标题\n\n段落 **加粗** 文本\n\n```rust\nfn a(){}\n```\n");
        assert!(html.contains("<h1>标题</h1>"));
        assert!(html.contains("<strong>加粗</strong>"));
        assert!(html.contains("language-rust"));
    }

    #[test]
    fn raw_html_is_escaped() {
        let html = markdown_to_html("前 <script>alert(1)</script> 后");
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }
}
