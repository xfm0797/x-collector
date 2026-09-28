use std::future::Future;

/// 指数退避重试：失败后等待 base * 2^attempt 毫秒再重试
pub async fn with_retry<F, Fut, T>(mut f: F, retries: u32, base_delay_ms: u64) -> Result<T, String>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, String>>,
{
    let mut attempt: u32 = 0;
    loop {
        match f().await {
            Ok(v) => return Ok(v),
            Err(e) => {
                if attempt >= retries {
                    return Err(format!("重试 {} 次后仍失败：{}", retries, e));
                }
                let delay = base_delay_ms * (1u64 << attempt);
                tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                attempt += 1;
            }
        }
    }
}
