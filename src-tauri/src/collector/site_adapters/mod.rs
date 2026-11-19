#![allow(dead_code)]
// 特殊网站适配器：针对知乎 / CSDN / 掘金 / 微信公众号等站点的正文选择器与请求头策略
pub mod csdn;
pub mod cto51;
pub mod infoq;
pub mod jianshu;
pub mod juejin;
pub mod oschina;
pub mod segmentfault;
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

/// 精确匹配域名（含子域名），避免 `notzhihu.com` 之类的误命中
fn host_matches(host: &str, domain: &str) -> bool {
    host == domain || host.ends_with(&format!(".{domain}"))
}

/// 根据 URL 返回站点适配配置
pub fn profile_for(url: &str) -> Option<SiteProfile> {
    let host = url::Url::parse(url).ok()?.host_str()?.to_lowercase();

    let profile = if host_matches(&host, "zhihu.com") {
        SiteProfile {
            content_selector: zhihu::CONTENT_SELECTOR,
            extra_remove: &[],
            user_agent: None,
            extra_headers: zhihu::EXTRA_HEADERS,
        }
    } else if host_matches(&host, "csdn.net") || host_matches(&host, "cnblogs.com") {
        SiteProfile {
            content_selector: csdn::CONTENT_SELECTOR,
            extra_remove: csdn::EXTRA_REMOVE,
            user_agent: None,
            extra_headers: &[],
        }
    } else if host_matches(&host, "juejin.cn") {
        SiteProfile {
            content_selector: juejin::CONTENT_SELECTOR,
            extra_remove: &[],
            user_agent: None,
            extra_headers: &[],
        }
    } else if host_matches(&host, "weixin.qq.com") {
        SiteProfile {
            content_selector: wechat::CONTENT_SELECTOR,
            extra_remove: &[],
            user_agent: Some(wechat::WECHAT_UA),
            extra_headers: &[],
        }
    } else if host_matches(&host, "jianshu.com") {
        SiteProfile {
            content_selector: jianshu::CONTENT_SELECTOR,
            extra_remove: jianshu::EXTRA_REMOVE,
            user_agent: None,
            extra_headers: &[],
        }
    } else if host_matches(&host, "segmentfault.com") {
        SiteProfile {
            content_selector: segmentfault::CONTENT_SELECTOR,
            extra_remove: segmentfault::EXTRA_REMOVE,
            user_agent: None,
            extra_headers: &[],
        }
    } else if host_matches(&host, "infoq.cn") || host_matches(&host, "infoq.com") {
        SiteProfile {
            content_selector: infoq::CONTENT_SELECTOR,
            extra_remove: infoq::EXTRA_REMOVE,
            user_agent: None,
            extra_headers: &[],
        }
    } else if host_matches(&host, "oschina.net") {
        SiteProfile {
            content_selector: oschina::CONTENT_SELECTOR,
            extra_remove: oschina::EXTRA_REMOVE,
            user_agent: None,
            extra_headers: &[],
        }
    } else if host_matches(&host, "51cto.com") {
        SiteProfile {
            content_selector: cto51::CONTENT_SELECTOR,
            extra_remove: cto51::EXTRA_REMOVE,
            user_agent: None,
            extra_headers: &[],
        }
    } else {
        return None;
    };

    Some(profile)
}

/// 根据站点域名返回正文 CSS 选择器
pub fn content_selector_for(url: &str) -> Option<String> {
    profile_for(url).map(|p| p.content_selector.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_supported_sites() {
        let cases = [
            ("https://zhuanlan.zhihu.com/p/1", "div.Post-RichTextContainer"),
            ("https://blog.csdn.net/u/article/details/1", "div#content_views"),
            ("https://www.cnblogs.com/u/p/1.html", "div#cnblogs_post_body"),
            ("https://juejin.cn/post/1", "div.markdown-body"),
            ("https://mp.weixin.qq.com/s/abc", "div#js_content"),
            ("https://www.jianshu.com/p/abc", "article"),
            ("https://segmentfault.com/a/123", "article"),
            ("https://www.infoq.cn/article/x", "div.article-content"),
            ("https://www.oschina.net/news/1", "div.article-detail"),
            ("https://blog.51cto.com/u_1/2", "div.article-content"),
        ];
        for (url, expected) in cases {
            let profile = profile_for(url).unwrap_or_else(|| panic!("未匹配站点：{url}"));
            assert!(
                profile.content_selector.contains(expected),
                "{url} 应包含选择器 {expected}，实际为 {}",
                profile.content_selector
            );
        }
    }

    #[test]
    fn does_not_match_lookalike_hosts() {
        assert!(profile_for("https://notzhihu.com/p/1").is_none());
        assert!(profile_for("https://csdn.net.evil.com/a").is_none());
        assert!(profile_for("https://example.com/post").is_none());
    }

    #[test]
    fn wechat_uses_mobile_ua() {
        let profile = profile_for("https://mp.weixin.qq.com/s/abc").unwrap();
        assert!(profile.user_agent.unwrap().contains("MicroMessenger"));
    }
}
