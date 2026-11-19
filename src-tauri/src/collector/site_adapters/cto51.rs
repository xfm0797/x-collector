/// 51CTO 博客（blog.51cto.com）适配：正文在 div.article-content
pub const CONTENT_SELECTOR: &str = "div.article-content, div.blog-content, article";

/// 51CTO 额外需要移除的元素（推荐位、底部广告、版权提示）
pub const EXTRA_REMOVE: &[&str] = &[
    "div.article-footer",
    "div.recommend-box",
    "div.copyright-box",
];
