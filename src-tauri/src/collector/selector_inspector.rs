//! 可视化选择器：页面 DOM 结构检查 + CSS 选择器生成与测试

use serde::Serialize;
use scraper::{Html, Selector};

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

    // 生成自身选择器片段：id 优先，其次 tag + 首个有意义 class
    let css = if let Some(id) = id.as_deref() {
        format!("{}#{id}", tag)
    } else if let Some(cls) = classes
        .iter()
        .find(|c| is_meaningful_class(c))
    {
        format!("{}.{}", tag, cls)
    } else {
        tag.clone()
    };

    // 完整路径：路径中若已含 id 可提前截断（id 唯一）
    let path = if let Some(id) = id.as_deref() {
        format!("{}#{id}", tag)
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

/// 过滤无意义 class（ hashed 类名、单字符、纯数字等）
fn is_meaningful_class(c: &str) -> bool {
    if c.chars().count() < 3 {
        return false;
    }
    // 类名中数字占比过高视为 hash 生成
    let digits = c.chars().filter(|ch| ch.is_ascii_digit()).count();
    if digits * 3 > c.chars().count() * 2 {
        return false;
    }
    !c.starts_with("css-") && !c.starts_with("js-") && !c.starts_with("_")
}
