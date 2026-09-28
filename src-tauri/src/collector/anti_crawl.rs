use rand::Rng;

/// 内置 User-Agent 池（随机轮换）
pub const USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36 Edg/125.0.0.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.4 Safari/605.1.15",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:127.0) Gecko/20100101 Firefox/127.0",
];

/// 随机挑选一个 User-Agent
pub fn random_user_agent() -> &'static str {
    let idx = rand::thread_rng().gen_range(0..USER_AGENTS.len());
    USER_AGENTS[idx]
}

/// 构造模拟浏览器的完整请求头
pub fn build_headers(base_url: &str, user_agent: Option<&str>) -> reqwest::header::HeaderMap {
    let mut headers = reqwest::header::HeaderMap::new();
    let ua: &str = match user_agent {
        Some(u) => u,
        None => random_user_agent(),
    };
    let _ = headers.insert(reqwest::header::USER_AGENT, ua.parse().unwrap());
    let _ = headers.insert(
        reqwest::header::ACCEPT,
        "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8"
            .parse()
            .unwrap(),
    );
    let _ = headers.insert(
        reqwest::header::ACCEPT_LANGUAGE,
        "zh-CN,zh;q=0.9,en;q=0.8".parse().unwrap(),
    );
    let _ = headers.insert(reqwest::header::CACHE_CONTROL, "no-cache".parse().unwrap());
    let _ = headers.insert(reqwest::header::UPGRADE_INSECURE_REQUESTS, "1".parse().unwrap());
    let _ = headers.insert("Sec-Fetch-Dest", "document".parse().unwrap());
    let _ = headers.insert("Sec-Fetch-Mode", "navigate".parse().unwrap());
    let _ = headers.insert("Sec-Fetch-Site", "none".parse().unwrap());
    let _ = headers.insert("Sec-Fetch-User", "?1".parse().unwrap());
    if let Ok(referer) = base_url.parse() {
        let _ = headers.insert(reqwest::header::REFERER, referer);
    }
    headers
}

/// 随机延迟（秒），默认 2-5 秒区间
pub async fn random_delay(min_sec: u64, max_sec: u64) {
    let lo = min_sec.min(max_sec);
    let hi = max_sec.max(min_sec).max(lo);
    let delay_ms = if hi == 0 {
        0
    } else {
        rand::thread_rng().gen_range((lo * 1000)..=(hi * 1000))
    };
    if delay_ms > 0 {
        tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
    }
}
