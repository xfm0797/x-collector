use std::time::Duration;

use serde::Deserialize;

use super::anti_crawl;
use crate::utils::retry::RetryError;

/// 单次响应体最大字节数（解压后），防止超大文件拖垮内存
const MAX_BODY_BYTES: usize = 8 * 1024 * 1024;

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

/// HTTP 抓取引擎：UA 轮换 + 完整请求头 + 随机延迟 + 分类重试 + 编码识别 + Cookie 携带
pub struct Fetcher {
    client: reqwest::Client,
    settings: FetchSettings,
}

impl Fetcher {
    pub fn new(settings: FetchSettings) -> Result<Self, String> {
        let timeout = Duration::from_secs(settings.timeout.max(5));
        let mut builder = reqwest::Client::builder()
            .timeout(timeout)
            // 建连与读取分开限时，避免慢站点长时间占用
            .connect_timeout(Duration::from_secs(settings.timeout.clamp(5, 10)))
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
                        .map_err(|e| classify_request_error(&e, settings.timeout))?;

                    let status = base.status();
                    let content_type = base
                        .headers()
                        .get(reqwest::header::CONTENT_TYPE)
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("")
                        .to_string();
                    let retry_after = parse_retry_after(base.headers());
                    let final_url = base.url().to_string();

                    if !status.is_success() {
                        let code = status.as_u16();
                        let message = format!("HTTP {code}");
                        return if is_retryable_status(code) {
                            Err(RetryError::retryable(message).with_retry_after(retry_after))
                        } else {
                            Err(RetryError::fatal(message))
                        };
                    }
                    if is_binary_content_type(&content_type) {
                        return Err(RetryError::fatal(format!("不支持的内容类型：{content_type}")));
                    }

                    let mut response = base;
                    let mut body: Vec<u8> = Vec::new();
                    while let Some(chunk) = response
                        .chunk()
                        .await
                        .map_err(|e| RetryError::retryable(format!("读取响应失败：{e}")))?
                    {
                        if body.len() + chunk.len() > MAX_BODY_BYTES {
                            return Err(RetryError::fatal(format!(
                                "响应体超过 {} MB 上限",
                                MAX_BODY_BYTES / 1024 / 1024
                            )));
                        }
                        body.extend_from_slice(&chunk);
                    }
                    if body.is_empty() {
                        return Err(RetryError::retryable("响应内容为空"));
                    }

                    let text = decode_body(&body, &content_type);
                    Ok(FetchedPage { final_url, text })
                }
            },
            settings.retries,
            800,
        )
        .await
    }
}

/// 请求发送失败：区分超时/连接类临时故障与永久失败
fn classify_request_error(err: &reqwest::Error, timeout_secs: u64) -> RetryError {
    if err.is_builder() || err.is_redirect() {
        // 配置错误、重定向超限：重试无法解决
        RetryError::fatal(format!("请求失败：{err}"))
    } else if err.is_timeout() {
        RetryError::retryable(format!("请求超时（{}s）", timeout_secs.max(5)))
    } else if err.is_connect() {
        RetryError::retryable(format!("连接失败：{err}"))
    } else if err.is_body() {
        RetryError::retryable(format!("读取响应失败：{err}"))
    } else if err.is_request() {
        RetryError::retryable(format!("请求失败：{err}"))
    } else {
        RetryError::fatal(format!("请求失败：{err}"))
    }
}

/// 408 请求超时、425 Too Early、429 限流与常见的临时性 5xx 可重试
fn is_retryable_status(code: u16) -> bool {
    matches!(code, 408 | 425 | 429 | 500 | 502 | 503 | 504)
}

/// 解析 Retry-After（秒），最多等待 10s
fn parse_retry_after(headers: &reqwest::header::HeaderMap) -> Option<Duration> {
    let value = headers.get(reqwest::header::RETRY_AFTER)?.to_str().ok()?;
    let secs: u64 = value.trim().parse().ok()?;
    Some(Duration::from_secs(secs.min(10)))
}

/// 图片 / 音视频 / 压缩包等二进制响应直接拒绝，避免解码出乱码
fn is_binary_content_type(content_type: &str) -> bool {
    let ct = content_type
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    if ct.is_empty() {
        return false;
    }
    if ct.starts_with("image/")
        || ct.starts_with("audio/")
        || ct.starts_with("video/")
        || ct.starts_with("font/")
    {
        return true;
    }
    matches!(
        ct.as_str(),
        "application/pdf"
            | "application/zip"
            | "application/gzip"
            | "application/x-gzip"
            | "application/x-rar-compressed"
            | "application/x-7z-compressed"
            | "application/x-tar"
            | "application/octet-stream"
    )
}

/// 取 URL 的站点根（作为 Referer）
fn base_of(url: &str) -> String {
    url::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|h| format!("{}://{}/", u.scheme(), h)))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retryable_status_classification() {
        for code in [408, 425, 429, 500, 502, 503, 504] {
            assert!(is_retryable_status(code), "HTTP {code} 应可重试");
        }
        for code in [400, 401, 403, 404, 410, 501] {
            assert!(!is_retryable_status(code), "HTTP {code} 不应重试");
        }
    }

    #[test]
    fn retry_after_is_capped() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(reqwest::header::RETRY_AFTER, "3600".parse().unwrap());
        assert_eq!(parse_retry_after(&headers), Some(Duration::from_secs(10)));

        headers.insert(reqwest::header::RETRY_AFTER, "3".parse().unwrap());
        assert_eq!(parse_retry_after(&headers), Some(Duration::from_secs(3)));

        headers.insert(reqwest::header::RETRY_AFTER, "abc".parse().unwrap());
        assert_eq!(parse_retry_after(&headers), None);
    }

    #[test]
    fn binary_content_types_are_rejected() {
        for ct in [
            "image/png",
            "video/mp4",
            "application/pdf",
            "application/octet-stream",
            "application/zip; charset=binary",
        ] {
            assert!(is_binary_content_type(ct), "{ct} 应被拒绝");
        }
        for ct in [
            "text/html; charset=utf-8",
            "application/xhtml+xml",
            "application/xml",
            "application/json",
            "",
        ] {
            assert!(!is_binary_content_type(ct), "{ct} 应被允许");
        }
    }
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
