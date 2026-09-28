#![allow(dead_code)]
// 特殊网站适配器：针对知乎 / CSDN / 掘金 / 微信公众号等站点的正文选择器与请求头策略
pub mod csdn;
pub mod juejin;
pub mod wechat;
pub mod zhihu;

/// 根据站点域名返回正文 CSS 选择器
pub fn content_selector_for(url: &str) -> Option<String> {
    let host = url::Url::parse(url).ok()?.host_str()?.to_lowercase();
    if host.contains("zhihu.com") {
        Some(zhihu::CONTENT_SELECTOR.to_string())
    } else if host.contains("csdn.net") || host.contains("cnblogs.com") {
        Some(csdn::CONTENT_SELECTOR.to_string())
    } else if host.contains("juejin.cn") {
        Some(juejin::CONTENT_SELECTOR.to_string())
    } else if host.contains("weixin.qq.com") || host.contains("mp.weixin") {
        Some(wechat::CONTENT_SELECTOR.to_string())
    } else {
        None
    }
}
