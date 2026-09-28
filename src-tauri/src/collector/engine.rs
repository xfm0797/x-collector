use serde::Deserialize;

use super::anti_crawl;

/// 采集请求设置（从 settings 表读取）
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct FetchSettings {
    pub timeout: u64,
    pub interval_min: u64,
    pub interval_max: u64,
    pub retries: u32,
    pub user_agent: String,
    pub proxy_url: String,
}

impl Default for FetchSettings {
    fn default() -> Self {
        Self {
            timeout: 30,
            interval_min: 2,
            interval_max: 5,
            retries: 3,
            user_agent: String::new(),
            proxy_url: String::new(),
        }
    }
}

/// 抓取结果
pub struct FetchedPage {
    pub final_url: String,
    pub text: String,
}

/// HTTP 抓取引擎：UA 轮换 + 完整请求头 + 随机延迟 + 指数退避重试 + 编码识别 + Cookie 携带
pub struct Fetcher {
    client: reqwest::Client,
    settings: FetchSettings,
}

impl Fetcher {
    pub fn new(settings: FetchSettings) -> Result<Self, String> {
        let mut builder = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(settings.timeout.max(5)))
            .cookie_store(true)
            .redirect(reqwest::redirect::Policy::limited(10));
        if !settings.proxy_url.is_empty() {
            let proxy =
                reqwest::Proxy::all(&settings.proxy_url).map_err(|e| format!("代理配置无效：{e}"))?;
            builder = builder.proxy(proxy);
        }
        let client = builder.build().map_err(|e| format!("HTTP 客户端初始化失败：{e}"))?;
        Ok(Self { client, settings })
    }

    /// 抓取页面并解码为文本
    pub async fn fetch_text(&self, url: &str) -> Result<FetchedPage, String> {
        let settings = self.settings.clone();
        let url_owned = url.to_string();
        let client = self.client.clone();

        super::anti_crawl::random_delay(settings.interval_min, settings.interval_max).await;

        crate::utils::retry::with_retry(
            || {
                let client = client.clone();
                let url = url_owned.clone();
                let settings = settings.clone();
                async move {
                    let base = client
                        .get(&url)
                        .headers(anti_crawl::build_headers(
                            &base_of(&url),
                            if settings.user_agent.is_empty() { None } else { Some(&settings.user_agent) },
                        ))
                        .send()
                        .await
                        .map_err(|e| format!("请求失败：{e}"))?;

                    let status = base.status();
                    let content_type = base
                        .headers()
                        .get(reqwest::header::CONTENT_TYPE)
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("")
                        .to_string();
                    let final_url = base.url().to_string();
                    let bytes = base.bytes().await.map_err(|e| format!("读取响应失败：{e}"))?;

                    if !status.is_success() {
                        return Err(format!("HTTP {}", status.as_u16()));
                    }

                    let text = decode_body(&bytes, &content_type);
                    Ok(FetchedPage { final_url, text })
                }
            },
            settings.retries,
            800,
        )
        .await
    }
}

/// 取 URL 的站点根（作为 Referer）
fn base_of(url: &str) -> String {
    url::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|h| format!("{}://{}/", u.scheme(), h)))
        .unwrap_or_default()
}

/// 响应体解码：优先 HTTP 头 charset，其次 meta 嗅探，兜底 UTF-8（支持 GBK 站点）
fn decode_body(bytes: &[u8], content_type: &str) -> String {
    let ct = content_type.to_lowercase();
    let charset_from_header = ct.split("charset=").nth(1).map(|s| s.trim().to_string());

    let encoding = match charset_from_header.as_deref() {
        Some("gbk") | Some("gb2312") | Some("gb18030") => encoding_rs::GBK,
        Some("utf-8") | Some("utf8") => encoding_rs::UTF_8,
        _ => {
            // meta 嗅探前 2KB
            let head = String::from_utf8_lossy(&bytes[..bytes.len().min(2048)]).to_lowercase();
            if head.contains("charset=gb") {
                encoding_rs::GBK
            } else {
                encoding_rs::UTF_8
            }
        }
    };

    let (cow, _, _) = encoding.decode(bytes);
    cow.to_string()
}
