/// 掘金适配：SSR 渲染，正文可直接从静态 HTML 提取
pub const CONTENT_SELECTOR: &str = "div.markdown-body, article";

/// 掘金移动端 / API 备选方案说明：
/// 1. 直接调用 API：POST https://api.juejin.cn/content_api/v1/article/detail
/// 2. 从页面内嵌 JSON（__NUXT__）中提取
pub const API_ENDPOINT: &str = "https://api.juejin.cn/content_api/v1/article/detail";
