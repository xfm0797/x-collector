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
    "[class*='advert']",
    "[id*='advert']",
    "[class*='breadcrumb']",
    "[class*='sponsor']",
    "[id*='google_ads']",
];

static LINK_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?i)(href|src|data-src)=["']([^"']+)["']"#).unwrap());
static IMG_TAG_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?is)<img\b[^>]*>").unwrap());
static IMG_SRC_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?is)\bsrc\s*=\s*["']([^"']*)["']"#).unwrap());
static IMG_LAZY_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?is)\b(?:data-src|data-original|data-lazy-src|data-actualsrc)\s*=\s*["']([^"']+)["']"#)
        .unwrap()
});
static PRE_BLOCK_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?is)<pre\b([^>]*)>(.*?)</pre>").unwrap());
static CODE_OPEN_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?is)^<code\b([^>]*)>").unwrap());
static CLASS_ATTR_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?is)\bclass\s*=\s*["']([^"']*)["']"#).unwrap());
static DATA_LANG_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?is)\bdata-(?:lang|language)\s*=\s*["']([^"']+)["']"#).unwrap()
});
static LANG_TOKEN_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-z0-9_+#.-]{1,20}$").unwrap());

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
    let html = fix_lazy_images(html);
    let mut doc = Html::parse_fragment(&html);

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

    // 4. 代码块语言标记补齐
    let with_langs = inject_code_languages(&absolutized);

    // 5. 空白规范化
    normalize_whitespace(&with_langs)
}

/// 从标签属性中识别代码语言
fn detect_language(attrs: &str) -> Option<String> {
    if let Some(lang) = DATA_LANG_RE
        .captures(attrs)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_lowercase())
        .filter(|s| LANG_TOKEN_RE.is_match(s))
    {
        return Some(lang);
    }

    let class = CLASS_ATTR_RE.captures(attrs)?.get(1)?.as_str();

    // SyntaxHighlighter 风格：class="brush: js; gutter: true"
    if let Some(rest) = class.trim().strip_prefix("brush:") {
        if let Some(candidate) = rest
            .trim()
            .split(|c: char| c.is_whitespace() || c == ';')
            .next()
            .map(|s| s.trim().to_lowercase())
        {
            if LANG_TOKEN_RE.is_match(&candidate) {
                return Some(candidate);
            }
        }
    }

    for token in class.split_whitespace() {
        let token = token.to_lowercase();
        let candidate = if let Some(v) = token.strip_prefix("language-") {
            v
        } else if let Some(v) = token.strip_prefix("lang-") {
            v
        } else if let Some(v) = token.strip_prefix("highlight-source-") {
            v
        } else if let Some(v) = token.strip_prefix("highlight-") {
            v
        } else if let Some(v) = token.strip_prefix("brush:") {
            v.trim()
        } else {
            continue;
        };
        if LANG_TOKEN_RE.is_match(candidate) {
            return Some(candidate.to_string());
        }
    }
    None
}

/// 确保 `<pre><code>` 上带有 html2md 能识别的 `language-xxx` class
fn inject_code_languages(html: &str) -> String {
    PRE_BLOCK_RE
        .replace_all(html, |caps: &regex::Captures| {
            let full = caps.get(0).map(|m| m.as_str()).unwrap_or("");
            let pre_attrs = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let body = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            let Some(code) = CODE_OPEN_RE.captures(body) else {
                return full.to_string();
            };
            let code_attrs = code.get(1).map(|m| m.as_str()).unwrap_or("");
            let Some(lang) =
                detect_language(code_attrs).or_else(|| detect_language(pre_attrs))
            else {
                return full.to_string();
            };
            let new_attrs = with_language_class(code_attrs, &lang);
            if new_attrs == code_attrs {
                return full.to_string();
            }
            let new_body = CODE_OPEN_RE
                .replace(body, format!("<code{new_attrs}>"))
                .to_string();
            format!("<pre{pre_attrs}>{new_body}</pre>")
        })
        .to_string()
}

/// 在 code 属性中补充（或追加）language class
fn with_language_class(attrs: &str, lang: &str) -> String {
    let target = format!("language-{lang}");
    if let Some(caps) = CLASS_ATTR_RE.captures(attrs) {
        let classes = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        if classes
            .split_whitespace()
            .any(|t| t.eq_ignore_ascii_case(&target))
        {
            return attrs.to_string();
        }
        let merged = format!("{} {target}", classes.trim())
            .trim()
            .to_string();
        return CLASS_ATTR_RE
            .replace(attrs, format!(r#"class="{merged}""#))
            .to_string();
    }
    format!("{attrs} class=\"{target}\"")
}

/// 将懒加载图片的 data-src 提升为 src，避免 Markdown 里只剩占位图
fn fix_lazy_images(html: &str) -> String {
    IMG_TAG_RE
        .replace_all(html, |caps: &regex::Captures| {
            let tag = caps.get(0).map(|m| m.as_str()).unwrap_or("");
            let Some(lazy) = IMG_LAZY_RE
                .captures(tag)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().to_string())
            else {
                return tag.to_string();
            };
            let current = IMG_SRC_RE
                .captures(tag)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().to_string());
            let replace_needed = match current.as_deref() {
                None => true,
                Some(src) => {
                    src.trim().is_empty()
                        || src.starts_with("data:")
                        || src.contains("placeholder")
                        || src.contains("loading")
                }
            };
            if !replace_needed {
                return tag.to_string();
            }
            match current {
                Some(_) => IMG_SRC_RE
                    .replace(tag, format!(r#"src="{lazy}""#))
                    .to_string(),
                None => tag.replacen("<img", &format!(r#"<img src="{lazy}""#), 1),
            }
        })
        .to_string()
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

/// 合并连续空白行、去除首尾空白、压缩连续 <br>（保留代码块内原始缩进）
pub fn normalize_whitespace(html: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut blank_run = 0;
    let mut pre_depth: isize = 0;
    for line in html.lines() {
        let lower = line.to_ascii_lowercase();
        let was_in_pre = pre_depth > 0;
        let opens = lower.matches("<pre").count() as isize;
        let closes = lower.matches("</pre").count() as isize;
        pre_depth = (pre_depth + opens - closes).max(0);

        // 代码块内的行原样保留，避免缩进与空行被破坏
        if was_in_pre || opens > 0 {
            out.push(line.to_string());
            blank_run = 0;
            continue;
        }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_language_variants() {
        for (html, lang) in [
            (r#"<pre><code class="lang-rust">x</code></pre>"#, "rust"),
            (r#"<pre class="language-js"><code>x</code></pre>"#, "js"),
            (r#"<pre><code data-lang="python">x</code></pre>"#, "python"),
            (r#"<pre><code class="brush: go">x</code></pre>"#, "go"),
            (
                r#"<pre><code class="highlight-source-ts">x</code></pre>"#,
                "ts",
            ),
        ] {
            let out = inject_code_languages(html);
            assert!(out.contains(&format!("language-{lang}")), "{out}");
        }
    }

    #[test]
    fn keeps_existing_language_class() {
        let html = r#"<pre><code class="language-rust hljs">x</code></pre>"#;
        let out = inject_code_languages(html);
        assert_eq!(out.matches("language-rust").count(), 1);
        assert!(out.contains("hljs"));
    }

    #[test]
    fn markdown_contains_language_fence() {
        let html =
            inject_code_languages(r#"<pre><code class="lang-rust">fn main() {}</code></pre>"#);
        let md = html2md::parse_html(&html);
        assert!(md.contains("```rust"), "{md}");
    }

    #[test]
    fn preserves_code_block_indentation() {
        let html = "<pre><code>fn main() {\n    let x = 1;\n\n    call();\n}</code></pre>";
        let out = normalize_whitespace(html);
        assert!(out.contains("    let x = 1;"), "{out}");
        assert!(out.contains("\n\n    call();"), "{out}");
    }

    #[test]
    fn does_not_guess_without_hint() {
        let html = r#"<pre><code class="hljs">x</code></pre>"#;
        assert_eq!(inject_code_languages(html), html);
    }
}
