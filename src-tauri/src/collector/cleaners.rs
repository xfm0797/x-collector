use once_cell::sync::Lazy;
use regex::Regex;
use scraper::{Html, Selector};

/// 广告 / 导航 / 侧边栏元素黑名单（类名 / ID / 标签）
const DEFAULT_REMOVE: &[&str] = &[
    "script",
    "style",
    "noscript",
    "iframe",
    "form",
    "nav",
    "header",
    "footer",
    "aside",
    "ins.adsbygoogle",
    "div.ad",
    "div.ads",
    "div.advert",
    "div.advertisement",
    "div.sidebar",
    "div.comment",
    "div.comments",
    "div.related",
    "div.related-posts",
    "div.recommend",
    "div.share",
    "div.social",
    "div.copyright",
    "[class*='sponsor']",
    "[id*='google_ads']",
];

static LINK_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?i)(href|src|data-src)="([^"]+)""#).unwrap());

/// 内容清洗选项
#[derive(Debug, Default, Clone)]
pub struct CleanOptions {
    /// 额外移除的选择器（用户自定义）
    pub remove_selectors: Vec<String>,
    /// 是否保留图片（false 时移除 img）
    pub keep_images: bool,
}

/// 内容清洗：移除广告/导航/脚本 → 相对链接转绝对 → 空白规范化
pub fn clean_fragment(html: &str, base_url: &str, opts: &CleanOptions) -> String {
    let mut doc = Html::parse_fragment(html);

    // 1. 收集需要移除的节点 ID
    let mut remove_css: Vec<String> = DEFAULT_REMOVE.iter().map(|s| s.to_string()).collect();
    remove_css.extend(opts.remove_selectors.iter().cloned());
    if !opts.keep_images {
        remove_css.push("img".to_string());
        remove_css.push("figure".to_string());
    }

    let mut ids_to_remove: Vec<_> = Vec::new();
    {
        let root = doc.root_element();
        for css in &remove_css {
            if let Ok(sel) = Selector::parse(css) {
                for el in root.select(&sel) {
                    ids_to_remove.push(el.id());
                }
            }
        }
    }

    // 2. 从树中移除命中的元素
    for id in ids_to_remove {
        if let Some(mut node) = doc.tree.get_mut(id) {
            node.detach();
        }
    }

    // 3. 序列化并做链接绝对化
    let root = doc.root_element();
    let serialized = root.inner_html();
    let absolutized = absolutize_links(&serialized, base_url);

    // 4. 空白规范化
    normalize_whitespace(&absolutized)
}

/// 将 HTML 中的相对链接（href/src/data-src）转为绝对链接
fn absolutize_links(html: &str, base_url: &str) -> String {
    let base = match url::Url::parse(base_url) {
        Ok(u) => u,
        Err(_) => return html.to_string(),
    };

    LINK_RE
        .replace_all(html, |caps: &regex::Captures| {
            let attr = caps.get(1).map(|m| m.as_str()).unwrap_or("href");
            let link = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            let resolved: Option<url::Url> = if link.starts_with("//") {
                url::Url::parse(&format!("{}:{}", base.scheme(), link)).ok()
            } else if link.starts_with("http")
                || link.starts_with('#')
                || link.starts_with("data:")
                || link.starts_with("javascript:")
                || link.starts_with("mailto:")
            {
                None
            } else {
                base.join(link).ok()
            };
            match resolved {
                Some(u) => format!(r#"{}="{}""#, attr, u),
                None => caps.get(0).map(|m| m.as_str().to_string()).unwrap_or_default(),
            }
        })
        .to_string()
}

/// 合并连续空白行、去除首尾空白、压缩连续 <br>
pub fn normalize_whitespace(html: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut blank_run = 0;
    for line in html.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            blank_run += 1;
            if blank_run <= 1 && !out.is_empty() {
                out.push(String::new());
            }
        } else {
            blank_run = 0;
            out.push(trimmed.to_string());
        }
    }
    let mut result = out.join("\n");
    for pair in [
        ("<br>\n<br>", "<br>"),
        ("<br/>\n<br/>", "<br/>"),
        ("<br />\n<br />", "<br />"),
        ("<br><br>", "<br>"),
    ] {
        while result.contains(pair.0) {
            result = result.replace(pair.0, pair.1);
        }
    }
    result.trim().to_string()
}
