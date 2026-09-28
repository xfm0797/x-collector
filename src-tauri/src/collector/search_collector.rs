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
        _ => format!("https://www.baidu.com/s?wd={q}&pn={}", (page - 1) * 10),
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

    for page in 1..=pages.max(1) {
        let url = build_search_url(engine, keyword, page, site_limit);
        let fetched = match fetcher.fetch_text(&url).await {
            Ok(f) => f,
            Err(_) => {
                // 单页失败不中断整体采集
                continue;
            }
        };
        let items = parse_search_results(&fetched.text, engine);
        for item in items {
            if !results.iter().any(|r| r.url == item.url) {
                results.push(item);
            }
        }
    }

    Ok(results)
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
