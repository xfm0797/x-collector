/// SegmentFault 适配：正文在 article 内，部分模板使用 div.article-content
pub const CONTENT_SELECTOR: &str = "article, div.article-content";

/// SegmentFault 站点额外需要移除的元素（赞赏、作者卡片、评论区入口）
pub const EXTRA_REMOVE: &[&str] = &[
    "div.article__reward",
    "div.article__footer",
    "div.widget-comment",
];
