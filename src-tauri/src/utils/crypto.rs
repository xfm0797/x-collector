//! CMS 密码加密存储：AES-256-GCM
//!
//! 主密钥保存在操作系统钥匙串（macOS Keychain / Windows Credential Manager /
//! Linux Secret Service）；钥匙串不可用时退回应用数据目录下的 `secret.key`
//! （Unix 权限 0600）。旧版本的固定 XOR 混淆数据仍可读取，并在首次使用时自动迁移。

use std::fs;
use std::path::Path;
use std::sync::OnceLock;

use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use rand::rngs::OsRng;
use rand::RngCore;

const KEY_FILE: &str = "secret.key";
const KEYRING_SERVICE: &str = "com.x-collector.desktop";
const KEYRING_USER: &str = "database-encryption-key";
const PREFIX: &str = "v2:";
const NONCE_LEN: usize = 12;
/// 仅用于解密历史数据，不再用于写入
const LEGACY_KEY: &[u8] = b"x-collector-maple-bell-2026";

static KEY: OnceLock<[u8; 32]> = OnceLock::new();

/// 初始化加密密钥（应用启动时调用一次）
pub fn init(data_dir: &Path) -> Result<(), String> {
    let key = resolve_key(data_dir)?;
    let _ = KEY.set(key);
    Ok(())
}

/// 加密密码；空密码保持为空字符串
pub fn encrypt_password(plain: &str) -> Result<String, String> {
    if plain.is_empty() {
        return Ok(String::new());
    }
    let cipher = cipher()?;
    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce_bytes), plain.as_bytes())
        .map_err(|_| "密码加密失败".to_string())?;

    let mut payload = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    payload.extend_from_slice(&nonce_bytes);
    payload.extend_from_slice(&ciphertext);
    Ok(format!("{}{}", PREFIX, STANDARD.encode(payload)))
}

/// 解密密码，返回 `(明文, 是否为旧版 XOR 格式)`
pub fn decrypt_password(stored: &str) -> Result<(String, bool), String> {
    if stored.is_empty() {
        return Ok((String::new(), false));
    }

    let Some(encoded) = stored.strip_prefix(PREFIX) else {
        return Ok((legacy_decrypt(stored), true));
    };

    let payload = STANDARD
        .decode(encoded)
        .map_err(|_| "密码数据格式无效".to_string())?;
    if payload.len() <= NONCE_LEN {
        return Err("密码数据长度无效".to_string());
    }
    let (nonce_bytes, ciphertext) = payload.split_at(NONCE_LEN);
    let plain = cipher()?
        .decrypt(Nonce::from_slice(nonce_bytes), ciphertext)
        .map_err(|_| "密码解密失败（密钥不匹配或数据已损坏）".to_string())?;
    let plain = String::from_utf8(plain).map_err(|_| "密码内容不是有效文本".to_string())?;
    Ok((plain, false))
}

fn cipher() -> Result<Aes256Gcm, String> {
    let key = KEY.get().ok_or_else(|| "加密密钥尚未初始化".to_string())?;
    Aes256Gcm::new_from_slice(key).map_err(|_| "加密密钥长度无效".to_string())
}

fn resolve_key(data_dir: &Path) -> Result<[u8; 32], String> {
    // 单元测试不访问系统钥匙串
    if cfg!(test) {
        return load_or_create_file_key(data_dir);
    }

    match keychain_key() {
        Ok(Some(key)) => Ok(key),
        Ok(None) => {
            let key = match read_key_file(data_dir)? {
                Some(key) => key,
                None => random_key(),
            };
            if save_keychain_key(&key).is_ok() {
                // 已迁移到系统钥匙串，移除本地密钥文件
                let _ = fs::remove_file(data_dir.join(KEY_FILE));
                Ok(key)
            } else {
                write_key_file(data_dir, &key)?;
                Ok(key)
            }
        }
        // 系统钥匙串不可用：退回本地密钥文件
        Err(_) => load_or_create_file_key(data_dir),
    }
}

fn keychain_entry() -> Result<keyring::Entry, String> {
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
        .map_err(|e| format!("系统钥匙串不可用：{e}"))
}

fn keychain_key() -> Result<Option<[u8; 32]>, String> {
    match keychain_entry()?.get_password() {
        Ok(encoded) => decode_key(encoded.trim()).map(Some),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("读取系统钥匙串失败：{e}")),
    }
}

fn save_keychain_key(key: &[u8; 32]) -> Result<(), String> {
    keychain_entry()?
        .set_password(&STANDARD.encode(key))
        .map_err(|e| format!("写入系统钥匙串失败：{e}"))
}

fn decode_key(encoded: &str) -> Result<[u8; 32], String> {
    let bytes = STANDARD
        .decode(encoded)
        .map_err(|_| "加密密钥格式无效".to_string())?;
    if bytes.len() != 32 {
        return Err("加密密钥长度无效".to_string());
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&bytes);
    Ok(key)
}

fn random_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

fn read_key_file(data_dir: &Path) -> Result<Option<[u8; 32]>, String> {
    let path = data_dir.join(KEY_FILE);
    match fs::read(&path) {
        Ok(bytes) if bytes.len() == 32 => {
            let mut key = [0u8; 32];
            key.copy_from_slice(&bytes);
            Ok(Some(key))
        }
        Ok(_) => Err(format!("密钥文件格式无效：{}", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("密钥文件读取失败：{e}")),
    }
}

fn write_key_file(data_dir: &Path, key: &[u8; 32]) -> Result<(), String> {
    let path = data_dir.join(KEY_FILE);
    fs::write(&path, key).map_err(|e| format!("密钥文件写入失败：{e}"))?;
    restrict_permissions(&path);
    Ok(())
}

fn load_or_create_file_key(data_dir: &Path) -> Result<[u8; 32], String> {
    match read_key_file(data_dir)? {
        Some(key) => Ok(key),
        None => {
            let key = random_key();
            write_key_file(data_dir, &key)?;
            Ok(key)
        }
    }
}

#[cfg(unix)]
fn restrict_permissions(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
}

#[cfg(not(unix))]
fn restrict_permissions(_path: &Path) {}

fn legacy_decrypt(encrypted: &str) -> String {
    let Ok(bytes) = hex::decode(encrypted) else {
        return String::new();
    };
    let decrypted: Vec<u8> = bytes
        .iter()
        .enumerate()
        .map(|(i, b)| b ^ LEGACY_KEY[i % LEGACY_KEY.len()])
        .collect();
    String::from_utf8(decrypted).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_test_key() {
        let dir = std::env::temp_dir().join(format!("x-collector-key-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        init(&dir).unwrap();
    }

    #[test]
    fn aes_roundtrip_uses_random_nonce() {
        init_test_key();
        let a = encrypt_password("secret123").unwrap();
        let b = encrypt_password("secret123").unwrap();
        assert!(a.starts_with(PREFIX));
        assert_ne!(a, b, "相同明文应产生不同密文");
        assert_eq!(decrypt_password(&a).unwrap(), ("secret123".to_string(), false));
        assert_eq!(encrypt_password("").unwrap(), "");
        assert_eq!(decrypt_password("").unwrap(), (String::new(), false));
    }

    #[test]
    fn legacy_xor_data_is_still_readable() {
        init_test_key();
        let legacy = hex::encode(
            "secret123"
                .bytes()
                .enumerate()
                .map(|(i, b)| b ^ LEGACY_KEY[i % LEGACY_KEY.len()])
                .collect::<Vec<_>>(),
        );
        let (plain, is_legacy) = decrypt_password(&legacy).unwrap();
        assert_eq!(plain, "secret123");
        assert!(is_legacy);
    }

    #[test]
    fn corrupted_payload_is_rejected() {
        init_test_key();
        assert!(decrypt_password("v2:not-base64!!").is_err());
        assert!(decrypt_password("v2:AAAA").is_err());
    }
}
