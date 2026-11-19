/// 开源中国（oschina.net）适配：资讯与博客正文在 div.article-detail
pub const CONTENT_SELECTOR: &str = "div.article-detail, div.article-content, article";

/// 开源中国额外需要移除的元素（底部推荐、相关文章、评论区）
pub const EXTRA_REMOVE: &[&str] = &[
    "div.article-footer",
    "div.related-article",
    "div.comment-list",
];
