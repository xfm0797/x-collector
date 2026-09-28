use crate::db::models::Article;
use crate::utils::slug::hexo_filename;

/// Hexo 导出选项
#[derive(Debug, Clone)]
pub struct HexoOptions {
    /// 导出目录（如 C:/blog/source/_posts）
    pub output_dir: String,
    pub default_category: String,
    pub include_source: bool,
}

/// 导出单篇文章为 Hexo Markdown 文件，返回写入的文件路径
pub fn export_article(article: &Article, opts: &HexoOptions) -> Result<String, String> {
    let dir = std::path::Path::new(&opts.output_dir);
    std::fs::create_dir_all(dir).map_err(|e| format!("创建目录失败：{e}"))?;

    let date = article
        .published_at
        .clone()
        .or_else(|| article.collected_at.clone())
        .or_else(|| article.updated_at.clone())
        .unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());

    let filename = hexo_filename(&article.title, &date);
    let path = dir.join(&filename);

    let front_matter = build_front_matter(article, &date, opts);
    let content = article.content_md.clone().unwrap_or_default();
    let full = format!("{}\n\n{}\n", front_matter, content);

    std::fs::write(&path, full.as_bytes()).map_err(|e| format!("写入文件失败：{e}"))?;
    Ok(path.to_string_lossy().to_string())
}

/// 生成 Hexo Front-matter（title/date/tags/categories/自定义字段）
fn build_front_matter(article: &Article, date: &str, opts: &HexoOptions) -> String {
    let mut lines = vec!["---".to_string()];
    lines.push(format!("title: {}", yaml_str(&article.title)));
    lines.push(format!("date: {}", normalize_date(date)));
    if let Some(author) = article.author.as_deref().filter(|a| !a.is_empty()) {
        lines.push(format!("author: {}", yaml_str(author)));
    }
    if let Some(excerpt) = article.excerpt.as_deref().filter(|e| !e.is_empty()) {
        let brief: String = excerpt.chars().take(100).collect();
        lines.push(format!("excerpt: {}", yaml_str(&brief)));
    }
    // 分类
    let category = article
        .category
        .clone()
        .filter(|c| !c.is_empty())
        .unwrap_or_else(|| {
            if opts.default_category.is_empty() {
                "未分类".to_string()
            } else {
                opts.default_category.clone()
            }
        });
    lines.push("categories:".to_string());
    lines.push(format!("  - {}", yaml_str(&category)));
    // 标签
    let tags = article.tags.clone().unwrap_or_default();
    if tags.is_empty() {
        lines.push("tags:".to_string());
        lines.push("  - 采集".to_string());
    } else {
        lines.push("tags:".to_string());
        for t in tags.iter().take(10) {
            lines.push(format!("  - {}", yaml_str(t)));
        }
    }
    // 自定义字段：原文来源
    if opts.include_source && article.url.starts_with("http") {
        lines.push(format!("source_url: {}", article.url));
    }
    lines.push("---".to_string());
    lines.join("\n")
}

/// YAML 字符串安全化
fn yaml_str(s: &str) -> String {
    if s.contains(':')
        || s.contains('#')
        || s.contains('"')
        || s.contains('\'')
        || s.starts_with(' ')
        || s.ends_with(' ')
        || s.is_empty()
    {
        format!("\"{}\"", s.replace('"', "\\\""))
    } else {
        s.to_string()
    }
}

/// 统一日期格式为 ISO（Hexo 兼容）
fn normalize_date(date: &str) -> String {
    let d = date.replace('T', " ");
    if d.len() >= 19 {
        format!("{}{}", &d[..19], if d.contains('+') { "" } else { " +08:00" })
    } else {
        format!("{} 00:00:00 +08:00", &d[..d.len().min(10)])
    }
}
