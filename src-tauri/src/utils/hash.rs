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

/// 密码加密存储（v0.0.1 使用可逆混淆，后续版本升级为系统钥匙串）
const KEY: &[u8] = b"x-collector-maple-bell-2026";

pub fn encrypt_password(plain: &str) -> String {
    if plain.is_empty() {
        return String::new();
    }
    let bytes: Vec<u8> = plain
        .bytes()
        .enumerate()
        .map(|(i, b)| b ^ KEY[i % KEY.len()])
        .collect();
    hex::encode(bytes)
}

pub fn decrypt_password(encrypted: &str) -> String {
    if encrypted.is_empty() {
        return String::new();
    }
    let Ok(bytes) = hex::decode(encrypted) else {
        return String::new();
    };
    let decrypted: Vec<u8> = bytes
        .iter()
        .enumerate()
        .map(|(i, b)| b ^ KEY[i % KEY.len()])
        .collect();
    String::from_utf8(decrypted).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_roundtrip() {
        let enc = encrypt_password("secret123");
        assert_eq!(decrypt_password(&enc), "secret123");
    }
}
