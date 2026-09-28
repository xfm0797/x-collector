use rust_embed::RustEmbed;
use serde::Serialize;

#[derive(RustEmbed)]
#[folder = "../docs/faq/"]
struct FaqAssets;

#[derive(Serialize, Clone)]
pub struct FaqCategory {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub count: usize,
}

#[derive(Serialize, Clone)]
pub struct FaqItem {
    pub id: String,
    pub category: String,
    pub question: String,
    pub answer: String,
    pub tags: Vec<String>,
    pub related: Vec<String>,
}

/// FAQ 分类
pub fn get_categories() -> Vec<FaqCategory> {
    let all = get_faqs(None);
    let cats: Vec<(&str, &str, &str)> = vec![
        ("install", "安装", "📦"),
        ("collect", "采集", "🔍"),
        ("publish", "发布", "📤"),
        ("rewrite", "伪原创", "✨"),
        ("error", "错误码", "⚠️"),
    ];
    cats.into_iter()
        .map(|(id, name, icon)| FaqCategory {
            id: id.to_string(),
            name: name.to_string(),
            icon: icon.to_string(),
            count: all.iter().filter(|f| f.category == id).count(),
        })
        .collect()
}

/// 解析 FAQ Markdown 文件（front-matter + # 问题 + 正文）
pub fn get_faqs(category: Option<&str>) -> Vec<FaqItem> {
    let mut result = Vec::new();
    for file in FaqAssets::iter() {
        let filename = file.to_string();
        let Some(asset) = FaqAssets::get(&filename) else { continue };
        let content = String::from_utf8_lossy(asset.data.as_ref()).to_string();
        if let Some(item) = parse_faq(&filename, &content) {
            if let Some(c) = category.filter(|c| !c.is_empty() && *c != "all") {
                if item.category != c {
                    continue;
                }
            }
            result.push(item);
        }
    }
    result
}

/// FAQ 文件格式：
/// ---
/// category: collect
/// tags: 403,反爬
/// ---
/// # 问题标题？
/// 回答正文（Markdown）...
fn parse_faq(filename: &str, content: &str) -> Option<FaqItem> {
    let rest = content.trim().strip_prefix("---")?;
    let (front, body) = rest.split_once("---")?;

    let mut category = String::new();
    let mut tags = Vec::new();
    for line in front.lines() {
        if let Some(v) = line.strip_prefix("category:") {
            category = v.trim().to_string();
        } else if let Some(v) = line.strip_prefix("tags:") {
            tags = v
                .split(|c: char| matches!(c, ',' | '，'))
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect();
        }
    }

    let question = body
        .lines()
        .find(|l| l.starts_with("# "))?
        .trim_start_matches("# ")
        .trim()
        .to_string();
    let answer = body
        .lines()
        .skip_while(|l| !l.starts_with("# "))
        .skip(1)
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string();

    let id = filename.trim_end_matches(".md").to_string();
    Some(FaqItem {
        id,
        category,
        question,
        answer,
        tags,
        related: vec![],
    })
}
