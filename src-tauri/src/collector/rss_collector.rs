use once_cell::sync::Lazy;
use regex::Regex;

use super::engine::Fetcher;
use crate::utils::html::decode_entities;

/// RSS / Atom 条目
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FeedItem {
    pub title: String,
    pub link: String,
    pub published: Option<String>,
    pub description: Option<String>,
}

static ITEM_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?is)<item>(.*?)</item>").unwrap());
static ENTRY_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?is)<entry>(.*?)</entry>").unwrap());
static TITLE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?is)<title>(?:<!\[CDATA\[)?(.*?)(?:\]\]>)?</title>").unwrap());
static LINK_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?is)<link[^>]*href="([^"]+)"[^>]*/?>|<link>(?:<!\[CDATA\[)?([^<\]]+?)(?:\]\]>)?</link>"#).unwrap());
static PUBDATE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?is)<(?:pubDate|published|updated|dc:date)>(?:<!\[CDATA\[)?(.*?)(?:\]\]>)?</(?:pubDate|published|updated|dc:date)>")
        .unwrap()
});
static DESC_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?is)<(?:description|summary|content)(?:[^>]*)>(?:<!\[CDATA\[)?(.*?)(?:\]\]>)?</(?:description|summary|content)>").unwrap());
static LOC_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?is)<loc>(.*?)</loc>").unwrap());

/// 抓取并解析 RSS / Atom 订阅源
pub async fn fetch_feed(fetcher: &Fetcher, feed_url: &str) -> Result<Vec<FeedItem>, String> {
    let page = fetcher.fetch_text(feed_url).await?;
    Ok(parse_feed(&page.text))
}

/// 解析 RSS / Atom XML 文本
pub fn parse_feed(xml: &str) -> Vec<FeedItem> {
    let mut items = Vec::new();

    let rss_blocks: Vec<String> = ITEM_RE
        .captures_iter(xml)
        .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
        .collect();
    let is_rss = !rss_blocks.is_empty();

    let blocks: Vec<String> = if is_rss {
        rss_blocks
    } else {
        ENTRY_RE
            .captures_iter(xml)
            .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
            .collect()
    };

    for block in blocks {
        let title = TITLE_RE
            .captures(&block)
            .and_then(|c| c.get(1))
            .map(|m| decode_entities(m.as_str().trim()))
            .unwrap_or_default();
        let link = LINK_RE
            .captures(&block)
            .and_then(|c| c.get(1).or_else(|| c.get(2)))
            .map(|m| decode_entities(m.as_str().trim()))
            .unwrap_or_default();
        let published = PUBDATE_RE
            .captures(&block)
            .and_then(|c| c.get(1))
            .map(|m| decode_entities(m.as_str().trim()));
        let description = DESC_RE
            .captures(&block)
            .and_then(|c| c.get(1))
            .map(|m| decode_entities(m.as_str().trim()));

        if !link.is_empty() {
            items.push(FeedItem {
                title,
                link,
                published,
                description,
            });
        }
    }
    items
}

/// 解析 sitemap.xml，返回去重后的全部链接
pub fn parse_sitemap(xml: &str) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    LOC_RE
        .find_iter(xml)
        .map(|m| {
            m.as_str()
                .replace("<loc>", "")
                .replace("</loc>", "")
                .replace("<![CDATA[", "")
                .replace("]]>", "")
                .trim()
                .to_string()
        })
        .filter(|s| !s.is_empty())
        .filter(|s| seen.insert(s.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sitemap_dedupes_and_strips_cdata() {
        let xml = r#"<?xml version="1.0"?>
            <urlset>
              <url><loc>https://example.com/a</loc></url>
              <url><loc><![CDATA[https://example.com/b]]></loc></url>
              <url><loc> https://example.com/a </loc></url>
            </urlset>"#;
        assert_eq!(
            parse_sitemap(xml),
            vec!["https://example.com/a", "https://example.com/b"]
        );
    }
}

