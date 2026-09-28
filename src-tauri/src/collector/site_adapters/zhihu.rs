/// 知乎专栏适配
/// 说明：知乎对未登录请求有限制，专栏页面（zhuanlan.zhihu.com/p/xxx）可直接静态抓取，
/// 回答页需要 x-requested-with 等请求头或 Cookie，见反爬模块。
pub const CONTENT_SELECTOR: &str = "div.Post-RichTextContainer, article";

/// 附加请求头（知乎要求）
pub fn extra_headers() -> Vec<(&'static str, &'static str)> {
    vec![("x-requested-with", "XMLHttpRequest")]
}
