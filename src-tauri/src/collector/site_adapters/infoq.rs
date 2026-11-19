/// InfoQ（infoq.cn）适配：正文在 div.article-content 中
pub const CONTENT_SELECTOR: &str = "div.article-content, article, div[class*='articleContent']";

/// InfoQ 站点额外需要移除的元素（作者信息、推荐位）
pub const EXTRA_REMOVE: &[&str] = &[
    "div.article-info",
    "div.recommend",
    "div.related-article",
];
