use serde::Deserialize;

/// 采集/新建文章时使用的输入模型
#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct ArticleInput {
    pub title: String,
    pub url: String,
    pub source: Option<String>,
    pub author: Option<String>,
    pub content_html: Option<String>,
    pub content_md: Option<String>,
    pub excerpt: Option<String>,
    pub tags: Option<Vec<String>>,
    pub category: Option<String>,
    pub cover_image: Option<String>,
    pub status: Option<String>,
}
