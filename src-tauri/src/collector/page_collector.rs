use once_cell::sync::Lazy;
use regex::Regex;

use crate::domain::ArticleInput;

use super::cleaners::{clean_fragment, CleanOptions};
use super::engine::Fetcher;
use super::parser::{parse, CustomSelectors, ParsedPage};
use super::postprocess;
use super::site_adapters;

/// 单页采集流程：抓取 → 解析 → 清洗 → 转 Markdown
pub async fn collect_page(
    fetcher: &Fetcher,
    url: &str,
    custom: Option<&CustomSelectors>,
    remove_selectors: &[String],
) -> Result<(ArticleInput, ParsedPage), String> {
    let profile = site_adapters::profile_for(url);
    let page = match profile {
        Some(p) => fetcher.fetch_text_with(url, p.user_agent, p.extra_headers).await?,
        None => fetcher.fetch_text(url).await?,
    };
    let parsed = parse(&page.text, &page.final_url, custom);

    if parsed.title.is_empty() && parsed.content_html.is_empty() {
        return Err(format!("无法从页面提取内容：{url}"));
    }

    // 用户自定义移除规则 + 站点专用广告/推荐区清理
    let mut remove = remove_selectors.to_vec();
    if let Some(p) = site_adapters::profile_for(&page.final_url) {
        remove.extend(p.extra_remove.iter().map(|s| s.to_string()));
    }
    let opts = CleanOptions {
        remove_selectors: remove,
        keep_images: true,
    };
    let cleaned = clean_fragment(&parsed.content_html, &page.final_url, &opts);
    let content_md = postprocess::normalize_markdown(&html2md::parse_html(&cleaned), &parsed.title);
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
    static MD_LINK_RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"!?\[([^\]]*)\]\([^)]*\)").unwrap());
    static MD_TOKEN_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[#>*_`~|]+").unwrap());

    let mut in_fence = false;
    let mut parts: Vec<&str> = Vec::new();
    for line in md.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        // 跳过代码块、表格、纯图片行
        if in_fence
            || trimmed.is_empty()
            || trimmed.starts_with('|')
            || trimmed.starts_with("![")
        {
            continue;
        }
        parts.push(trimmed.trim_start_matches("- ").trim_start_matches("+ "));
    }

    let text = parts.join(" ");
    let text = MD_LINK_RE.replace_all(&text, "$1");
    let text = text.replace("**", "").replace("__", "").replace("~~", "");
    let text = MD_TOKEN_RE.replace_all(&text, " ");
    let collapsed: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let chars: Vec<char> = collapsed.chars().collect();
    if chars.len() > 120 {
        chars.into_iter().take(120).collect::<String>() + "..."
    } else {
        collapsed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn excerpt_skips_images_code_and_markdown_noise() {
        let md = "![封面](https://example.com/cover.png)\n\n第一段 **内容**。\n\n```rust\nfn main() {}\n```\n\n- 列表项 [链接](https://example.com)";
        let out = extract_excerpt(md);
        assert!(out.starts_with("第一段 内容。"));
        assert!(!out.contains("封面"));
        assert!(!out.contains("fn main"));
        assert!(out.contains("列表项 链接"));
    }
}

fn host_of(url: &str) -> Option<String> {
    url::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_string()))
}
