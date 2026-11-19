use once_cell::sync::Lazy;
use scraper::{ElementRef, Html, Selector};
use serde_json::Value;

use super::site_adapters;

/// 正文候选结构（每个页面只编译一次）
static STRUCT_SEL: Lazy<Selector> =
    Lazy::new(|| Selector::parse("article, div, section, main, td").expect("结构选择器"));
/// 链接元素（用于计算链接密度）
static LINK_SEL: Lazy<Selector> = Lazy::new(|| Selector::parse("a").expect("链接选择器"));
/// body 兜底
static BODY_SEL: Lazy<Selector> = Lazy::new(|| Selector::parse("body").expect("body 选择器"));
/// JSON-LD 结构化数据
static JSON_LD_SEL: Lazy<Selector> =
    Lazy::new(|| Selector::parse("script[type='application/ld+json']").expect("JSON-LD 选择器"));

/// 解析结果
#[derive(Debug, Default, Clone)]
pub struct ParsedPage {
    pub title: String,
    pub author: Option<String>,
    pub publish_date: Option<String>,
    pub content_html: String,
    pub tags: Vec<String>,
    pub cover: Option<String>,
}

/// 自定义选择器配置
#[derive(Debug, Default, Clone)]
pub struct CustomSelectors {
    pub title: Option<String>,
    pub content: Option<String>,
    pub author: Option<String>,
    pub date: Option<String>,
    pub tags: Option<String>,
    pub cover: Option<String>,
}

fn try_selector(css: &str) -> Option<Selector> {
    Selector::parse(css).ok()
}

fn first_text(doc: &Html, css: &str) -> Option<String> {
    let sel = try_selector(css)?;
    doc.select(&sel)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .filter(|s| !s.is_empty())
}

fn first_attr(doc: &Html, css: &str, attr: &str) -> Option<String> {
    let sel = try_selector(css)?;
    doc.select(&sel)
        .next()
        .and_then(|el| el.value().attr(attr))
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
}

/// 读取所有匹配元素的 content 属性（如多个 article:tag）
fn all_contents(doc: &Html, css: &str) -> Vec<String> {
    let Some(sel) = try_selector(css) else {
        return Vec::new();
    };
    doc.select(&sel)
        .filter_map(|el| el.value().attr("content"))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// 按中英文分隔符拆分标签，限制数量
fn split_tags(text: &str) -> Vec<String> {
    text.split(|c: char| matches!(c, ',' | '，' | ';' | '；') || c.is_whitespace())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && s.chars().count() <= 20)
        .take(8)
        .collect()
}

/// 解析 HTML 页面为结构化内容：自定义选择器 → 站点适配 → 智能提取
pub fn parse(html: &str, url: &str, custom: Option<&CustomSelectors>) -> ParsedPage {
    let doc = Html::parse_document(html);
    let json_ld = json_ld_meta(&doc);
    let mut result = ParsedPage::default();

    // 标题：og:title → 自定义 → JSON-LD → h1 → <title>
    result.title = first_attr(&doc, "meta[property='og:title']", "content")
        .or_else(|| first_attr(&doc, "meta[name='twitter:title']", "content"))
        .or_else(|| custom.and_then(|c| c.title.as_deref()).and_then(|css| first_text(&doc, css)))
        .or(json_ld.title.clone())
        .or_else(|| first_text(&doc, "h1"))
        .or_else(|| first_text(&doc, "title"))
        .map(|t| clean_title(&t))
        .unwrap_or_default();

    result.author = first_attr(&doc, "meta[name='author']", "content")
        .or_else(|| first_attr(&doc, "meta[property='article:author']", "content"))
        .or_else(|| {
            custom
                .and_then(|c| c.author.as_deref())
                .and_then(|css| first_text(&doc, css))
        })
        .or(json_ld.author.clone());

    result.publish_date = first_attr(&doc, "meta[property='article:published_time']", "content")
        .or_else(|| {
            custom
                .and_then(|c| c.date.as_deref())
                .and_then(|css| first_text(&doc, css))
        })
        .or(json_ld.published.clone())
        .or_else(|| first_attr(&doc, "time[datetime]", "datetime"))
        .or_else(|| first_text(&doc, "time"));

    if let Some(keywords) = first_attr(&doc, "meta[name='keywords']", "content") {
        result.tags = split_tags(&keywords);
    }
    if result.tags.is_empty() {
        result.tags = json_ld.tags.clone();
    }
    // 多个 article:tag / article:section 追加为标签
    for tag in all_contents(&doc, "meta[property='article:tag'], meta[property='article:section']") {
        if result.tags.len() >= 8 || result.tags.iter().any(|t| t == &tag) {
            continue;
        }
        result.tags.push(tag);
    }
    if let Some(css) = custom.and_then(|c| c.tags.as_deref()) {
        if let Some(text) = first_text(&doc, css) {
            result.tags = split_tags(&text);
        }
    }

    result.cover = first_attr(&doc, "meta[property='og:image']", "content")
        .or_else(|| {
            custom
                .and_then(|c| c.cover.as_deref())
                .and_then(|css| first_attr(&doc, css, "src"))
        })
        .or(json_ld.cover.clone());

    // 正文：自定义选择器 → 站点适配 → 智能提取（类 Readability）
    let content_css = custom
        .and_then(|c| c.content.clone())
        .or_else(|| site_adapters::content_selector_for(url));
    let content_el = content_css
        .as_deref()
        .and_then(|css| try_selector(css))
        .and_then(|sel| doc.select(&sel).next())
        .or_else(|| smart_content_element(&doc));

    if let Some(el) = content_el {
        result.content_html = el.inner_html();
    } else if let Some(body) = doc.select(&BODY_SEL).next() {
        // 兜底：结构提取失败时保留整个 body，避免标题存在但正文为空
        result.content_html = body.inner_html();
    }

    result
}

/// 清理标题中的站点后缀，如 " - 知乎"
fn clean_title(t: &str) -> String {
    let t = t.trim();
    for sep in [" - ", " | ", " _ ", "——"] {
        if let Some(pos) = t.rfind(sep) {
            let tail = &t[pos + sep.len()..];
            if tail.chars().count() <= 12 {
                return t[..pos].trim().to_string();
            }
        }
    }
    t.to_string()
}

/// 智能正文提取：按文本密度与语义类名打分选择最佳容器
fn smart_content_element(doc: &Html) -> Option<ElementRef<'_>> {
    let mut best: Option<(i64, ElementRef)> = None;

    for el in doc.select(&STRUCT_SEL) {
        let text_len: usize = el.text().map(|t| t.trim().len()).sum();
        if text_len < 200 {
            continue;
        }
        // 语义提示：class/id 命中内容关键词加分
        let class_id = format!(
            "{} {}",
            el.value().attr("class").unwrap_or(""),
            el.value().attr("id").unwrap_or("")
        )
        .to_lowercase();
        let mut score = text_len as i64;
        for hint in ["content", "article", "post", "entry", "main", "text", "blog", "markdown"] {
            if class_id.contains(hint) {
                score += 300;
                break;
            }
        }
        for bad in ["comment", "sidebar", "footer", "nav", "related", "recommend", "ad"] {
            if class_id.contains(bad) {
                score -= 500;
                break;
            }
        }
        // 链接密度过高（导航类）降权
        let link_text: usize = el
            .select(&LINK_SEL)
            .map(|a| a.text().map(|t| t.trim().len()).sum::<usize>())
            .sum();
        if text_len > 0 && link_text * 10 > text_len * 7 {
            score -= 800;
        }

        if best.as_ref().map(|(s, _)| score > *s).unwrap_or(true) {
            best = Some((score, el));
        }
    }
    best.map(|(_, el)| el)
}

/// JSON-LD 中提取到的元数据
#[derive(Default)]
struct JsonLdMeta {
    title: Option<String>,
    author: Option<String>,
    published: Option<String>,
    cover: Option<String>,
    tags: Vec<String>,
}

/// 解析页面中的 JSON-LD 结构化数据（Article / NewsArticle / BlogPosting 等）
fn json_ld_meta(doc: &Html) -> JsonLdMeta {
    let mut meta = JsonLdMeta::default();
    for el in doc.select(&JSON_LD_SEL) {
        let raw: String = el.text().collect();
        let Ok(value) = serde_json::from_str::<Value>(raw.trim()) else {
            continue;
        };
        scan_json_ld(&value, &mut meta, 0);
        if meta.title.is_some()
            && meta.author.is_some()
            && meta.published.is_some()
            && meta.cover.is_some()
        {
            break;
        }
    }
    meta
}

fn scan_json_ld(value: &Value, meta: &mut JsonLdMeta, depth: usize) {
    if depth > 6 {
        return;
    }
    match value {
        Value::Array(items) => {
            for item in items {
                scan_json_ld(item, meta, depth + 1);
            }
        }
        Value::Object(map) => {
            if let Some(graph) = map.get("@graph") {
                scan_json_ld(graph, meta, depth + 1);
            }
            if !map.get("@type").map(is_article_type).unwrap_or(false) {
                return;
            }
            if meta.title.is_none() {
                meta.title = string_field(map, "headline").or_else(|| string_field(map, "name"));
            }
            if meta.author.is_none() {
                meta.author = author_of(map.get("author"));
            }
            if meta.published.is_none() {
                meta.published = string_field(map, "datePublished");
            }
            if meta.cover.is_none() {
                meta.cover = image_of(map.get("image"));
            }
            if meta.tags.is_empty() {
                meta.tags = keywords_of(map.get("keywords"));
            }
        }
        _ => {}
    }
}

fn is_article_type(value: &Value) -> bool {
    match value {
        Value::String(s) => matches!(
            s.as_str(),
            "Article" | "NewsArticle" | "BlogPosting" | "TechArticle" | "Report"
        ),
        Value::Array(items) => items.iter().any(is_article_type),
        _ => false,
    }
}

fn string_field(map: &serde_json::Map<String, Value>, key: &str) -> Option<String> {
    map.get(key).and_then(|v| v.as_str()).and_then(non_empty)
}

fn non_empty(s: &str) -> Option<String> {
    let t = s.trim();
    (!t.is_empty()).then(|| t.to_string())
}

fn author_of(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(s) => non_empty(s),
        Value::Array(items) => items.iter().find_map(|v| author_of(Some(v))),
        Value::Object(map) => string_field(map, "name"),
        _ => None,
    }
}

fn image_of(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(s) => non_empty(s),
        Value::Array(items) => items.iter().find_map(|v| image_of(Some(v))),
        Value::Object(map) => {
            string_field(map, "url").or_else(|| string_field(map, "contentUrl"))
        }
        _ => None,
    }
}

fn keywords_of(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::String(s)) => split_tags(s),
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|v| v.as_str())
            .flat_map(split_tags)
            .take(8)
            .collect(),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_json_ld_metadata() {
        let html = r#"<html><head><title>页面标题 - 站点</title>
            <script type="application/ld+json">
            {"@type":"NewsArticle","headline":"结构化标题","author":[{"@type":"Person","name":"张三"}],
             "datePublished":"2026-01-02T03:04:05Z","image":{"url":"https://cdn.example.com/a.jpg"},
             "keywords":"科技, AI"}
            </script></head>
            <body><article><p>正文内容。</p></article></body></html>"#;

        let parsed = parse(html, "https://example.com/post", None);
        assert_eq!(parsed.title, "结构化标题");
        assert_eq!(parsed.author.as_deref(), Some("张三"));
        assert_eq!(parsed.publish_date.as_deref(), Some("2026-01-02T03:04:05Z"));
        assert_eq!(parsed.cover.as_deref(), Some("https://cdn.example.com/a.jpg"));
        assert_eq!(parsed.tags, vec!["科技", "AI"]);
    }

    #[test]
    fn falls_back_to_body_when_structure_is_short() {
        let html = r#"<html><head><title>短文</title></head>
            <body><div>只有一句话的正文。</div></body></html>"#;
        let parsed = parse(html, "https://example.com/short", None);
        assert_eq!(parsed.title, "短文");
        assert!(parsed.content_html.contains("只有一句话的正文"));
    }

    #[test]
    fn collects_multiple_article_tags() {
        let html = r#"<html><head>
            <meta property="article:tag" content="Rust">
            <meta property="article:tag" content="Tauri">
            <meta property="article:tag" content="Rust">
            </head><body><article><p>正文</p></article></body></html>"#;
        let parsed = parse(html, "https://example.com/t", None);
        assert_eq!(parsed.tags, vec!["Rust", "Tauri"]);
    }
}
