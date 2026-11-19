use sha2::{Digest, Sha256};

/// 计算 SHA-256 十六进制摘要
pub fn sha256_hex(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    hex::encode(hasher.finalize())
}

/// 内容指纹：取前 500 字（去除空白）的 SHA-256
pub fn fingerprint(text: &str) -> String {
    let normalized: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    let head: String = normalized.chars().take(500).collect();
    sha256_hex(&head)
}
