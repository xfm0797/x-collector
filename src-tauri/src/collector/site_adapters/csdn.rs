/// CSDN / 博客园适配：页面结构复杂、广告多，使用专用正文选择器 + 黑名单清洗
pub const CONTENT_SELECTOR: &str = "div#content_views, div.article_content, div#cnblogs_post_body";

/// CSDN 站点额外需要移除的元素
pub const EXTRA_REMOVE: &[&str] = &[
    "div.blog-content-bottom",
    "div.recommend-box",
    "div.template-box",
    "div.hide-article-box",
    "div.admire-box",
    "div.toolbar-advert",
];
