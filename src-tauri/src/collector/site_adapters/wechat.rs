/// 微信公众号适配：正文在 div#js_content 中（visibility: hidden 由 JS 解除，静态 HTML 仍可提取）
pub const CONTENT_SELECTOR: &str = "div#js_content, div.rich_media_content";

/// 微信公众号站点的访问策略：
/// 1. 使用搜狗微信搜索（weixin.sogou.com）发现文章链接
/// 2. 模拟微信客户端 UA 降低拦截概率
pub const WECHAT_UA: &str = "Mozilla/5.0 (Linux; Android 14) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Mobile Safari/537.36 MicroMessenger/8.0.49";
