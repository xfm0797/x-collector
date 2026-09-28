use scraper::{ElementRef, Html, Selector};

use super::site_adapters;

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

/// 解析 HTML 页面为结构化内容：自定义选择器 → 站点适配 → 智能提取
pub fn parse(html: &str, url: &str, custom: Option<&CustomSelectors>) -> ParsedPage {
    let doc = Html::parse_document(html);
    let mut result = ParsedPage::default();

    // 标题：og:title → 自定义 → h1 → <title>
    result.title = first_attr(&doc, "meta[property='og:title']", "content")
        .or_else(|| first_attr(&doc, "meta[name='twitter:title']", "content"))
        .or_else(|| custom.and_then(|c| c.title.as_deref()).and_then(|css| first_text(&doc, css)))
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
        });

    result.publish_date = first_attr(&doc, "meta[property='article:published_time']", "content")
        .or_else(|| {
            custom
                .and_then(|c| c.date.as_deref())
                .and_then(|css| first_text(&doc, css))
        })
        .or_else(|| first_attr(&doc, "time[datetime]", "datetime"))
        .or_else(|| first_text(&doc, "time"));

    if let Some(keywords) = first_attr(&doc, "meta[name='keywords']", "content") {
        result.tags = keywords
            .split(|c: char| matches!(c, ',' | '，' | ';' | '；'))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() && s.chars().count() <= 20)
            .take(8)
            .collect();
    }
    if let Some(css) = custom.and_then(|c| c.tags.as_deref()) {
        if let Some(text) = first_text(&doc, css) {
            result.tags = text
                .split(|c: char| matches!(c, ',' | '，' | ';' | '；') || c.is_whitespace())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .take(8)
                .collect();
        }
    }

    result.cover = first_attr(&doc, "meta[property='og:image']", "content")
        .or_else(|| {
            custom
                .and_then(|c| c.cover.as_deref())
                .and_then(|css| first_attr(&doc, css, "src"))
        });

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
    } else if result.title.is_empty() {
        // 兜底：整个 body
        if let Some(body) = try_selector("body").and_then(|s| doc.select(&s).next()) {
            result.content_html = body.inner_html();
        }
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
    let sel = try_selector("article, div, section, main, td")?;
    let mut best: Option<(i64, ElementRef)> = None;

    for el in doc.select(&sel) {
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
            .select(&try_selector("a").unwrap())
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
