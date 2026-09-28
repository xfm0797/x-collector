use crate::db::models::ArticleInput;

use super::cleaners::{clean_fragment, CleanOptions};
use super::engine::Fetcher;
use super::parser::{parse, CustomSelectors, ParsedPage};

/// 单页采集流程：抓取 → 解析 → 清洗 → 转 Markdown
pub async fn collect_page(
    fetcher: &Fetcher,
    url: &str,
    custom: Option<&CustomSelectors>,
    remove_selectors: &[String],
) -> Result<(ArticleInput, ParsedPage), String> {
    let page = fetcher.fetch_text(url).await?;
    let parsed = parse(&page.text, &page.final_url, custom);

    if parsed.title.is_empty() && parsed.content_html.is_empty() {
        return Err(format!("无法从页面提取内容：{url}"));
    }

    let opts = CleanOptions {
        remove_selectors: remove_selectors.to_vec(),
        keep_images: true,
    };
    let cleaned = clean_fragment(&parsed.content_html, &page.final_url, &opts);
    let content_md = html2md::parse_html(&cleaned);
    let excerpt = extract_excerpt(&content_md);

    let input = ArticleInput {
        title: if parsed.title.is_empty() {
            url.to_string()
        } else {
            parsed.title.clone()
        },
        url: page.final_url.clone(),
        source: host_of(&page.final_url),
        author: parsed.author.clone(),
        content_html: Some(cleaned),
        content_md: Some(content_md),
        excerpt: Some(excerpt),
        tags: Some(parsed.tags.clone()),
        category: None,
        cover_image: parsed.cover.clone(),
        status: Some("collected".to_string()),
    };

    Ok((input, parsed))
}

/// 提取 Markdown 纯文本摘要（前 120 字）
pub fn extract_excerpt(md: &str) -> String {
    let text: String = md
        .lines()
        .filter(|l| !l.trim_start().starts_with("```") && !l.starts_with('|'))
        .collect::<Vec<_>>()
        .join(" ");
    let cleaned: String = text
        .replace("```", " ")
        .replace("![", " ")
        .replace(']', " ")
        .replace('(', " ")
        .replace(')', " ")
        .replace('#', " ")
        .replace('>', " ")
        .replace('*', " ")
        .replace('-', " ");
    let collapsed: String = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    let chars: Vec<char> = collapsed.chars().collect();
    if chars.len() > 120 {
        chars.into_iter().take(120).collect::<String>() + "..."
    } else {
        collapsed
    }
}

fn host_of(url: &str) -> Option<String> {
    url::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_string()))
}
