//! 可视化选择器：页面 DOM 结构检查 + CSS 选择器生成与测试

use serde::Serialize;
use scraper::{Element, Html, Selector};

use super::engine::Fetcher;

/// DOM 树节点（传给前端渲染）
#[derive(Debug, Serialize, Clone)]
pub struct DomNode {
    /// 标签名（小写）
    pub tag: String,
    /// id 属性（如有）
    pub id: Option<String>,
    /// class 列表
    pub classes: Vec<String>,
    /// 该节点的直接 CSS 选择器片段（如 `div.article-content`）
    pub css: String,
    /// 从 body 起的完整 CSS 选择器路径
    pub path: String,
    /// 节点自身直接文本（前后各截取）
    pub text_preview: Option<String>,
    /// 元素内文本总长度（用于判断正文容器）
    pub text_length: usize,
    /// 子节点
    pub children: Vec<DomNode>,
}

/// 页面检查结果
#[derive(Debug, Serialize)]
pub struct PageInspect {
    pub final_url: String,
    pub title: String,
    pub dom: DomNode,
}

/// 选择器测试结果
#[derive(Debug, Serialize)]
pub struct SelectorMatch {
    pub index: usize,
    /// 文本前 300 字符
    pub text_preview: String,
    /// inner_html 长度
    pub html_length: usize,
}

#[derive(Debug, Serialize)]
pub struct SelectorTestResult {
    pub selector: String,
    pub matched: usize,
    pub matches: Vec<SelectorMatch>,
}

/// 忽略这些标签（不进入树）
const SKIP_TAGS: &[&str] = &[
    "script", "style", "noscript", "svg", "iframe", "template", "link", "meta", "br", "head",
];

/// 检查页面：抓取 HTML 并生成 DOM 树
pub async fn inspect_page(fetcher: &Fetcher, url: &str) -> Result<PageInspect, String> {
    let page = fetcher.fetch_text(url).await?;
    let doc = Html::parse_document(&page.text);

    let title = doc
        .select(&Selector::parse("title").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_default();

    let body = doc
        .select(&Selector::parse("body").unwrap())
        .next()
        .ok_or_else(|| "页面中没有 <body> 元素".to_string())?;

    let dom = build_node(body, "body", 0);
    Ok(PageInspect {
        final_url: page.final_url,
        title,
        dom,
    })
}

/// 测试选择器：返回匹配数量与文本预览
pub async fn test_selector(
    fetcher: &Fetcher,
    url: &str,
    css: &str,
) -> Result<SelectorTestResult, String> {
    let sel = Selector::parse(css).map_err(|_| format!("无效的 CSS 选择器：{css}"))?;
    let page = fetcher.fetch_text(url).await?;
    let doc = Html::parse_document(&page.text);

    let mut matches: Vec<SelectorMatch> = doc
        .select(&sel)
        .take(20)
        .enumerate()
        .map(|(i, el)| {
            let text: String = el.text().collect::<String>();
            let preview: String = text.trim().chars().take(300).collect();
            SelectorMatch {
                index: i + 1,
                text_preview: preview,
                html_length: el.inner_html().len(),
            }
        })
        .collect();
    let matched = matches.len();
    // 精简预览数量
    matches.truncate(10);
    Ok(SelectorTestResult {
        selector: css.to_string(),
        matched,
        matches,
    })
}

/// 递归构建 DOM 树（限制深度 14、每层子节点 80 个、总节点 4000）
fn build_node(el: scraper::ElementRef<'_>, parent_path: &str, depth: usize) -> DomNode {
    let value = el.value();
    let tag = value.name().to_string();

    let id = value.attr("id").map(|s| s.to_string());
    let classes: Vec<String> = value
        .attr("class")
        .map(|c| {
            c.split_whitespace()
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();

    // 生成自身选择器片段：id 优先，其次语义 class（最多两个），最后标签兜底
    let css = node_css(&tag, id.as_deref(), &classes, &el);

    // 完整路径：有 id 时从该节点重新开始；根节点直接使用自身选择器
    let path = if id.is_some() || depth == 0 {
        css.clone()
    } else {
        format!("{parent_path} > {css}")
    };

    // 文本长度与直接文本预览
    let full_text: String = el.text().collect::<String>();
    let text_length = full_text.trim().len();
    let direct_text: String = el
        .children()
        .filter_map(|c| match c.value() {
            scraper::node::Node::Text(t) => Some(&**t),
            _ => None,
        })
        .collect::<String>()
        .trim()
        .to_string();
    let text_preview = if direct_text.is_empty() {
        None
    } else {
        Some(direct_text.chars().take(60).collect::<String>())
    };

    let mut children = Vec::new();
    if depth < 14 {
        let mut count = 0;
        for child in el.children() {
            if count >= 80 {
                break;
            }
            if let Some(child_el) = scraper::ElementRef::wrap(child) {
                if SKIP_TAGS.contains(&child_el.value().name()) {
                    continue;
                }
                // 跳过纯空元素
                let child_text: String = child_el.text().collect::<String>();
                if child_el.value().name() != "img"
                    && child_el.children().count() == 0
                    && child_text.trim().is_empty()
                    && child_el.value().id().is_none()
                {
                    continue;
                }
                children.push(build_node(child_el, &path, depth + 1));
                count += 1;
            }
        }
    }

    DomNode {
        tag,
        id,
        classes,
        css,
        path,
        text_preview,
        text_length,
        children,
    }
}

/// 过滤无意义 class（hashed 类名、状态/工具类等），并按语义排序
fn meaningful_classes(classes: &[String]) -> Vec<&str> {
    let mut scored: Vec<(i32, usize, &str)> = classes
        .iter()
        .enumerate()
        .map(|(i, c)| (class_score(c), i, c.as_str()))
        .filter(|(score, _, _)| *score > 0)
        .collect();
    // 分数高的优先；同分保持原顺序
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    scored.into_iter().take(2).map(|(_, _, c)| c).collect()
}

/// 状态类与布局工具类：不适合作为采集选择器
const STATE_CLASSES: &[&str] = &[
    "active", "selected", "current", "open", "hover", "focus", "disabled", "hidden",
    "show", "visible", "collapsed", "expanded", "sticky", "fixed", "clearfix",
];
const UTILITY_EXACT: &[&str] = &[
    "container", "wrapper", "row", "col", "flex", "grid", "block", "inline",
    "relative", "absolute", "static", "center", "left", "right",
];
const UTILITY_PREFIXES: &[&str] = &[
    "col-", "row-", "mt-", "mb-", "ml-", "mr-", "mx-", "my-", "p-", "px-", "py-",
    "pt-", "pb-", "pl-", "pr-", "m-", "w-", "h-", "min-", "max-", "bg-", "border-",
    "rounded", "justify-", "items-", "gap-", "space-", "font-", "leading-", "tracking-",
    "z-", "flex-", "grid-", "md:", "sm:", "lg:", "xl:", "hover:", "focus:",
];
/// 正文语义关键词
const SEMANTIC_HINTS: &[&str] = &[
    "article", "content", "post", "entry", "main", "body", "markdown", "rich", "detail", "prose",
];

/// class 可用性评分：负数表示不适合作为选择器
fn class_score(class: &str) -> i32 {
    let len = class.chars().count();
    if len < 3 || len > 40 {
        return -100;
    }
    let lower = class.to_lowercase();
    // 构建工具/哈希类名
    if lower.starts_with("css-")
        || lower.starts_with("js-")
        || lower.starts_with("sc-")
        || lower.starts_with('_')
        || lower.contains(':')
    {
        return -100;
    }
    let digits = lower.chars().filter(|c| c.is_ascii_digit()).count();
    if digits * 3 > len * 2 {
        return -100; // 数字占比过高，视为 hash
    }
    // 语义类优先
    if SEMANTIC_HINTS.iter().any(|h| lower.contains(h)) {
        return 60 + (len.min(20) as i32 / 2);
    }
    if STATE_CLASSES.contains(&lower.as_str()) || UTILITY_EXACT.contains(&lower.as_str()) {
        return -100;
    }
    if UTILITY_PREFIXES.iter().any(|p| lower.starts_with(p)) {
        return -50;
    }
    if class.contains('-') || class.contains('_') {
        return 20;
    }
    10
}

/// 生成节点的 CSS 片段
fn node_css(
    tag: &str,
    id: Option<&str>,
    classes: &[String],
    el: &scraper::ElementRef<'_>,
) -> String {
    if let Some(id) = id {
        let css = format!("{}#{}", tag, css_escape_ident(id));
        if Selector::parse(&css).is_ok() {
            return css;
        }
        return tag.to_string();
    }

    let meaningful = meaningful_classes(classes);
    if !meaningful.is_empty() {
        let mut css = tag.to_string();
        for class in meaningful {
            css.push('.');
            css.push_str(&css_escape_ident(class));
        }
        if Selector::parse(&css).is_ok() {
            return css;
        }
    }

    // 无可用 class 时，同类兄弟节点用 nth-of-type 区分，保证路径可定位
    if same_tag_sibling_exists(el) {
        format!("{}:nth-of-type({})", tag, nth_of_type(el))
    } else {
        tag.to_string()
    }
}

/// CSS 标识符转义（处理数字开头、含特殊字符的 id/class）
fn css_escape_ident(ident: &str) -> String {
    let mut out = String::with_capacity(ident.len() + 8);
    for (i, ch) in ident.chars().enumerate() {
        let safe = ch.is_ascii_alphanumeric() || ch == '-' || ch == '_';
        if i == 0 && ch.is_ascii_digit() {
            out.push_str(&format!("\\3{} ", ch));
        } else if safe {
            out.push(ch);
        } else {
            out.push('\\');
            out.push(ch);
        }
    }
    out
}

fn same_tag_sibling_exists(el: &scraper::ElementRef<'_>) -> bool {
    let name = el.value().name();
    let mut prev = el.prev_sibling_element();
    while let Some(p) = prev {
        if p.value().name() == name {
            return true;
        }
        prev = p.prev_sibling_element();
    }
    let mut next = el.next_sibling_element();
    while let Some(n) = next {
        if n.value().name() == name {
            return true;
        }
        next = n.next_sibling_element();
    }
    false
}

fn nth_of_type(el: &scraper::ElementRef<'_>) -> usize {
    let name = el.value().name();
    let mut index = 1;
    let mut prev = el.prev_sibling_element();
    while let Some(p) = prev {
        if p.value().name() == name {
            index += 1;
        }
        prev = p.prev_sibling_element();
    }
    index
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_path_has_no_duplicate_body() {
        let doc = Html::parse_document(
            r#"<html><body><div class="article-content"><p>正文内容</p></div></body></html>"#,
        );
        let body = doc.select(&Selector::parse("body").unwrap()).next().unwrap();
        let node = build_node(body, "body", 0);
        assert_eq!(node.path, "body");
        assert_eq!(node.children[0].path, "body > div.article-content");
    }

    #[test]
    fn bare_siblings_use_nth_of_type() {
        let doc = Html::parse_document("<html><body><div>一</div><div>二</div></body></html>");
        let body = doc.select(&Selector::parse("body").unwrap()).next().unwrap();
        let node = build_node(body, "body", 0);
        assert_eq!(node.children[0].css, "div:nth-of-type(1)");
        assert_eq!(node.children[1].css, "div:nth-of-type(2)");
    }

    #[test]
    fn semantic_class_is_preferred_over_utility() {
        assert!(class_score("article-content") > class_score("custom-box"));
        assert!(class_score("mt-4") < 0);
        assert!(class_score("active") < 0);
        assert!(class_score("css-1a2b3c") < 0);
    }

    #[test]
    fn unsafe_identifiers_are_escaped() {
        assert_eq!(css_escape_ident("123abc"), "\\31 23abc");
        assert_eq!(css_escape_ident("post:body"), "post\\:body");

        let doc = Html::parse_document(r#"<html><body><div id="1main">x</div></body></html>"#);
        let body = doc.select(&Selector::parse("body").unwrap()).next().unwrap();
        let node = build_node(body, "body", 0);
        assert_eq!(node.children[0].path, "div#\\31 main");
    }
}
