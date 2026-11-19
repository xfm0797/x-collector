#![allow(dead_code)]
// 特殊网站适配器：针对知乎 / CSDN / 掘金 / 微信公众号等站点的正文选择器与请求头策略
pub mod csdn;
pub mod juejin;
pub mod wechat;
pub mod zhihu;

/// 站点适配配置：正文选择器 + 额外清洗规则 + 请求策略
#[derive(Debug, Clone, Copy)]
pub struct SiteProfile {
    /// 正文 CSS 选择器（逗号分隔的候选）
    pub content_selector: &'static str,
    /// 需要额外移除的元素选择器
    pub extra_remove: &'static [&'static str],
    /// 站点专用 User-Agent（优先于全局设置）
    pub user_agent: Option<&'static str>,
    /// 站点专用请求头
    pub extra_headers: &'static [(&'static str, &'static str)],
}

/// 根据 URL 返回站点适配配置
pub fn profile_for(url: &str) -> Option<SiteProfile> {
    let host = url::Url::parse(url).ok()?.host_str()?.to_lowercase();
    if host.contains("zhihu.com") {
        Some(SiteProfile {
            content_selector: zhihu::CONTENT_SELECTOR,
            extra_remove: &[],
            user_agent: None,
            extra_headers: zhihu::EXTRA_HEADERS,
        })
    } else if host.contains("csdn.net") || host.contains("cnblogs.com") {
        Some(SiteProfile {
            content_selector: csdn::CONTENT_SELECTOR,
            extra_remove: csdn::EXTRA_REMOVE,
            user_agent: None,
            extra_headers: &[],
        })
    } else if host.contains("juejin.cn") {
        Some(SiteProfile {
            content_selector: juejin::CONTENT_SELECTOR,
            extra_remove: &[],
            user_agent: None,
            extra_headers: &[],
        })
    } else if host.contains("weixin.qq.com") || host.contains("mp.weixin") {
        Some(SiteProfile {
            content_selector: wechat::CONTENT_SELECTOR,
            extra_remove: &[],
            user_agent: Some(wechat::WECHAT_UA),
            extra_headers: &[],
        })
    } else {
        None
    }
}

/// 根据站点域名返回正文 CSS 选择器
pub fn content_selector_for(url: &str) -> Option<String> {
    profile_for(url).map(|p| p.content_selector.to_string())
}
