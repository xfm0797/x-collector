/// 简书适配：正文在 article 内，部分模板使用 div.show-content
pub const CONTENT_SELECTOR: &str = "article, div.show-content";

/// 简书站点额外需要移除的元素（推荐阅读、底部操作区等）
pub const EXTRA_REMOVE: &[&str] = &[
    "div.recommend-notes",
    "div.note-bottom",
    "div.follow-dialog",
];
