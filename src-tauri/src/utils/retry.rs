use std::future::Future;
use std::time::Duration;

use rand::Rng;

/// 单次重试的最大退避时间
const MAX_BACKOFF_MS: u64 = 8_000;

/// 重试错误：区分临时故障（可重试）与永久失败（立即返回）
#[derive(Debug, Clone)]
pub struct RetryError {
    message: String,
    retryable: bool,
    retry_after: Option<Duration>,
}

impl RetryError {
    /// 临时故障：超时、连接失败、5xx、429 等
    pub fn retryable(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            retryable: true,
            retry_after: None,
        }
    }

    /// 永久失败：4xx、配置错误等，不再重试
    pub fn fatal(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            retryable: false,
            retry_after: None,
        }
    }

    /// 服务端 Retry-After 建议的等待时间
    pub fn with_retry_after(mut self, delay: Option<Duration>) -> Self {
        self.retry_after = delay;
        self
    }
}

/// 指数退避重试
///
/// - 仅对 `RetryError::retryable` 的错误重试；
/// - 优先遵循服务端 `Retry-After`，否则按 `base * 2^attempt`（上限 8s）并附加 25% 抖动；
/// - `retries` 为额外重试次数，总尝试次数为 `retries + 1`。
pub async fn with_retry<F, Fut, T>(mut f: F, retries: u32, base_delay_ms: u64) -> Result<T, String>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, RetryError>>,
{
    let mut attempt: u32 = 0;
    loop {
        match f().await {
            Ok(v) => return Ok(v),
            Err(err) => {
                if !err.retryable || attempt >= retries {
                    return Err(if attempt == 0 {
                        err.message
                    } else {
                        format!("重试 {} 次后仍失败：{}", attempt, err.message)
                    });
                }
                let delay = err
                    .retry_after
                    .unwrap_or_else(|| backoff(base_delay_ms, attempt));
                tokio::time::sleep(delay).await;
                attempt += 1;
            }
        }
    }
}

fn backoff(base_delay_ms: u64, attempt: u32) -> Duration {
    let exp = base_delay_ms
        .saturating_mul(1u64 << attempt.min(10))
        .min(MAX_BACKOFF_MS);
    let jitter = if exp == 0 {
        0
    } else {
        rand::thread_rng().gen_range(0..=exp / 4)
    };
    Duration::from_millis(exp + jitter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test(flavor = "current_thread")]
    async fn fatal_error_is_not_retried() {
        let calls = Arc::new(AtomicU32::new(0));
        let c = calls.clone();
        let result: Result<(), String> = with_retry(
            || {
                let c = c.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Err(RetryError::fatal("HTTP 404"))
                }
            },
            3,
            0,
        )
        .await;

        assert_eq!(result.unwrap_err(), "HTTP 404");
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn retryable_error_stops_after_retries() {
        let calls = Arc::new(AtomicU32::new(0));
        let c = calls.clone();
        let result: Result<(), String> = with_retry(
            || {
                let c = c.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Err(RetryError::retryable("请求超时"))
                }
            },
            2,
            0,
        )
        .await;

        assert_eq!(result.unwrap_err(), "重试 2 次后仍失败：请求超时");
        assert_eq!(calls.load(Ordering::SeqCst), 3);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn succeeds_after_transient_failures() {
        let calls = Arc::new(AtomicU32::new(0));
        let c = calls.clone();
        let result: Result<u32, String> = with_retry(
            || {
                let c = c.clone();
                async move {
                    let n = c.fetch_add(1, Ordering::SeqCst) + 1;
                    if n < 3 {
                        Err(RetryError::retryable("连接失败"))
                    } else {
                        Ok(n)
                    }
                }
            },
            3,
            0,
        )
        .await;

        assert_eq!(result.unwrap(), 3);
        assert_eq!(calls.load(Ordering::SeqCst), 3);
    }
}
