use scraper::Html;
use urlencoding::encode;

use super::engine::Fetcher;

/// 搜索结果条目
#[derive(Debug, Clone)]
pub struct SearchItem {
    pub title: String,
    pub url: String,
}

/// 生成搜索 URL
pub fn build_search_url(engine: &str, keyword: &str, page: u64, site_limit: Option<&str>) -> String {
    let mut q = keyword.to_string();
    if let Some(site) = site_limit.filter(|s| !s.trim().is_empty()) {
        q = format!("{} site:{}", q, site.trim());
    }
    let q = encode(&q);

    match engine {
        "google" => format!(
            "https://www.google.com/search?q={q}&start={}&num=10&hl=zh-CN",
            page.saturating_sub(1) * 10
        ),
        "bing" => format!(
            "https://www.bing.com/search?q={q}&first={}&setlang=zh-hans",
            page.saturating_sub(1) * 10 + 1
        ),
        "sogou" => format!("https://www.sogou.com/web?query={q}&page={page}"),
        _ => format!("https://www.baidu.com/s?wd={q}&pn={}", page.saturating_sub(1) * 10),
    }
}

/// 执行搜索引擎查询并解析结果列表
pub async fn search(
    fetcher: &Fetcher,
    engine: &str,
    keyword: &str,
    pages: u64,
    site_limit: Option<&str>,
) -> Result<Vec<SearchItem>, String> {
    let mut results: Vec<SearchItem> = Vec::new();
    let mut last_error: Option<String> = None;
    let mut succeeded = 0usize;

    for page in 1..=pages.max(1) {
        let url = build_search_url(engine, keyword, page, site_limit);
        let fetched = match fetcher.fetch_text(&url).await {
            Ok(f) => f,
            Err(e) => {
                // 单页失败不中断整体采集，但记录错误用于全失败时反馈
                last_error = Some(e);
                continue;
            }
        };
        succeeded += 1;

        let items = parse_search_results(&fetched.text, engine);
        if items.is_empty() {
            // 没有更多结果（或已触发反爬验证页），继续翻页没有意义
            break;
        }
        for item in items {
            let Some(url) = normalize_result_url(&item.url) else {
                continue;
            };
            if !results.iter().any(|r| r.url == url) {
                results.push(SearchItem {
                    title: item.title,
                    url,
                });
            }
        }
    }

    if succeeded == 0 {
        if let Some(err) = last_error {
            return Err(format!("搜索引擎请求失败：{err}"));
        }
    }
    Ok(results)
}

/// 结果 URL 归一化：仅保留 http(s)，去除 fragment 与常见跟踪参数
pub fn normalize_result_url(raw: &str) -> Option<String> {
    let mut url = url::Url::parse(raw).ok()?;
    if !matches!(url.scheme(), "http" | "https") {
        return None;
    }
    url.set_fragment(None);
    if url.query().is_some() {
        let kept: Vec<(String, String)> = url
            .query_pairs()
            .filter(|(k, _)| !is_tracking_param(k))
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect();
        if kept.is_empty() {
            url.set_query(None);
        } else {
            url.query_pairs_mut().clear().extend_pairs(kept);
        }
    }
    Some(url.to_string())
}

fn is_tracking_param(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    key.starts_with("utm_")
        || matches!(
            key.as_str(),
            "spm" | "scm" | "fbclid" | "gclid" | "msclkid" | "yclid" | "share_source" | "share_medium"
        )
}

/// 解析各搜索引擎结果页
pub fn parse_search_results(html: &str, engine: &str) -> Vec<SearchItem> {
    let doc = Html::parse_document(html);
    let mut items = Vec::new();

    let selectors: &[(&str, &str)] = match engine {
        "google" => &[("div.g, div[data-sokoban-container]", "a[href] h3, h3")],
        "bing" => &[("li.b_algo", "h2 a")],
        "sogou" => &[("div.vrwrap, div.rb", "h3 a")],
        _ => &[("div.result, div.c-container", "h3 a")],
    };

    for (item_sel, link_sel) in selectors {
        let Ok(item_selector) = scraper::Selector::parse(item_sel) else { continue };
        let Ok(link_selector) = scraper::Selector::parse(link_sel) else { continue };

        for item in doc.select(&item_selector) {
            let Some(link) = item.select(&link_selector).next() else { continue };
            let Some(href) = link.value().attr("href") else { continue };
            let title = link.text().collect::<String>().trim().to_string();
            if title.is_empty() || href.is_empty() {
                continue;
            }
            // 过滤站内链接
            if href.starts_with('/') || !href.starts_with("http") {
                continue;
            }
            items.push(SearchItem {
                title,
                url: href.to_string(),
            });
        }
    }

    items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_tracking_params_and_fragment() {
        let url = normalize_result_url("https://Example.com/post?utm_source=x&id=3&spm=abc#top")
            .unwrap();
        assert_eq!(url, "https://example.com/post?id=3");
    }

    #[test]
    fn drops_query_when_only_tracking_params() {
        let url = normalize_result_url("https://example.com/post?utm_medium=social").unwrap();
        assert_eq!(url, "https://example.com/post");
    }

    #[test]
    fn rejects_non_http_urls() {
        assert!(normalize_result_url("javascript:alert(1)").is_none());
        assert!(normalize_result_url("mailto:a@b.com").is_none());
        assert!(normalize_result_url("/relative/path").is_none());
    }
}
