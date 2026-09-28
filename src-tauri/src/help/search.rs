use serde::Serialize;

use super::{examples, faq, manual};

#[derive(Serialize)]
pub struct SearchResult {
    pub section: String,
    pub title: String,
    pub snippet: String,
    pub path: String,
    pub relevance: f32,
}

/// 帮助全文搜索：手册 + 配置示例 + FAQ，按相关性排序，最多 20 条
pub fn search(query: &str) -> Vec<SearchResult> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return vec![];
    }

    let mut results: Vec<SearchResult> = Vec::new();

    // 手册
    for (title, content) in manual::all_sections() {
        let title_l = title.to_lowercase();
        let content_l = content.to_lowercase();
        if title_l.contains(&query) || content_l.contains(&query) {
            let relevance = if title_l.contains(&query) { 0.95 } else { 0.6 };
            let path = title_to_path(&title);
            results.push(SearchResult {
                section: "manual".to_string(),
                title,
                snippet: extract_snippet(&content, &query),
                path,
                relevance,
            });
        }
    }

    // 配置示例
    for example in examples::get_examples(None) {
        let haystack = format!("{} {} {}", example.title, example.description, example.content).to_lowercase();
        if haystack.contains(&query) {
            results.push(SearchResult {
                section: "example".to_string(),
                title: example.title,
                snippet: extract_snippet(&example.description, &query),
                path: example.id,
                relevance: 0.8,
            });
        }
    }

    // FAQ
    for item in faq::get_faqs(None) {
        let haystack = format!("{} {} {}", item.question, item.answer, item.tags.join(" ")).to_lowercase();
        if haystack.contains(&query) {
            let relevance = if item.question.to_lowercase().contains(&query) { 0.9 } else { 0.65 };
            results.push(SearchResult {
                section: "faq".to_string(),
                title: item.question,
                snippet: extract_snippet(&item.answer, &query),
                path: item.id,
                relevance,
            });
        }
    }

    results.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap_or(std::cmp::Ordering::Equal));
    results.truncate(20);
    results
}

/// 提取命中位置附近的摘要
fn extract_snippet(content: &str, query: &str) -> String {
    let content_l = content.to_lowercase();
    if let Some(pos) = content_l.find(query) {
        // 按字符边界截取（避免 panic）
        let chars: Vec<char> = content.chars().collect();
        let char_pos = content[..pos].chars().count();
        let start = char_pos.saturating_sub(20);
        let end = (char_pos + query.chars().count() + 30).min(chars.len());
        let snippet: String = chars[start..end].iter().collect();
        format!("...{}...", snippet.replace('\n', " "))
    } else {
        let chars: Vec<char> = content.chars().collect();
        let snippet: String = chars.iter().take(80).collect();
        snippet.replace('\n', " ")
    }
}

/// 手册标题 → 路径
fn title_to_path(title: &str) -> String {
    for (path, t) in [
        ("index", "手册首页"),
        ("quickstart", "快速开始"),
        ("keyword-collect", "关键词采集"),
        ("source-manager", "采集源管理"),
        ("content-cleaning", "内容清洗"),
        ("rewrite", "伪原创"),
        ("publish", "发布配置"),
        ("cms-config", "CMS 连接配置"),
        ("settings", "系统设置"),
        ("troubleshooting", "故障排除"),
    ] {
        if t == title {
            return path.to_string();
        }
    }
    title.to_string()
}
