//! 图片本地化：下载 Markdown 中的远程图片并以 data URI 内嵌，避免外链失效

use std::collections::HashMap;

use base64::Engine;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::collector::engine::Fetcher;
use crate::domain::ArticleInput;

/// 单篇最多处理的图片数量
const MAX_IMAGES: usize = 20;
/// 单张图片大小上限
const MAX_IMAGE_BYTES: usize = 3 * 1024 * 1024;
/// 单篇内嵌图片总大小上限
const MAX_TOTAL_BYTES: usize = 10 * 1024 * 1024;

static IMAGE_MD_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"!\[([^\]]*)\]\((https?://[^)\s]+)(?:\s+"[^"]*")?\)"#).unwrap()
});

/// 下载 Markdown 中的远程图片并替换为 data URI；失败或超限时保留原链接
pub async fn localize_markdown(md: &str, fetcher: &Fetcher, referer: &str) -> String {
    let mut urls: Vec<String> = Vec::new();
    for caps in IMAGE_MD_RE.captures_iter(md) {
        if let Some(url) = caps.get(2).map(|m| m.as_str().to_string()) {
            if !urls.contains(&url) {
                urls.push(url);
            }
        }
    }
    if urls.is_empty() {
        return md.to_string();
    }

    let mut embedded: HashMap<String, String> = HashMap::new();
    let mut total = 0usize;
    for url in urls.iter().take(MAX_IMAGES) {
        let Ok((bytes, content_type)) = fetcher
            .fetch_bytes(url, Some(referer), MAX_IMAGE_BYTES)
            .await
        else {
            continue;
        };
        let Some(mime) = image_mime(&content_type, &bytes) else {
            continue;
        };
        if total + bytes.len() > MAX_TOTAL_BYTES {
            break;
        }
        total += bytes.len();
        let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
        embedded.insert(url.clone(), format!("data:{mime};base64,{encoded}"));
    }

    if embedded.is_empty() {
        return md.to_string();
    }
    rewrite_images(md, &embedded)
}

/// 就地把文章正文中的图片替换为本地内嵌数据
pub async fn localize_article(input: &mut ArticleInput, fetcher: &Fetcher) {
    let Some(md) = input.content_md.take() else {
        return;
    };
    input.content_md = Some(localize_markdown(&md, fetcher, &input.url).await);
}

/// 用已下载的数据替换 Markdown 图片链接
fn rewrite_images(md: &str, embedded: &HashMap<String, String>) -> String {
    IMAGE_MD_RE
        .replace_all(md, |caps: &regex::Captures| {
            let Some(url) = caps.get(2) else {
                return caps.get(0).map(|m| m.as_str().to_string()).unwrap_or_default();
            };
            match embedded.get(url.as_str()) {
                Some(data) => format!(
                    "![{}]({data})",
                    caps.get(1).map(|m| m.as_str()).unwrap_or("")
                ),
                None => caps
                    .get(0)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default(),
            }
        })
        .to_string()
}

/// 根据响应头与文件魔数识别图片类型，仅接受已知安全格式
fn image_mime(content_type: &str, bytes: &[u8]) -> Option<&'static str> {
    if bytes.len() >= 3 && bytes[0..3] == [0xFF, 0xD8, 0xFF] {
        return Some("image/jpeg");
    }
    if bytes.len() >= 8 && bytes[0..8] == [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A] {
        return Some("image/png");
    }
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return Some("image/gif");
    }
    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Some("image/webp");
    }
    if bytes.len() >= 12
        && bytes.len() > 14
        && &bytes[4..8] == b"ftyp"
        && (&bytes[8..12] == b"avif" || &bytes[8..12] == b"avis")
    {
        return Some("image/avif");
    }
    if bytes.len() >= 4 && bytes[0..4] == [0x00, 0x00, 0x01, 0x00] {
        return Some("image/x-icon");
    }
    if bytes.starts_with(b"BM") && bytes.len() > 1024 {
        return Some("image/bmp");
    }

    let head = String::from_utf8_lossy(&bytes[..bytes.len().min(512)]);
    let trimmed = head.trim_start().trim_start_matches('\u{feff}');
    if trimmed.starts_with("<svg") || (trimmed.starts_with("<?xml") && head.contains("<svg")) {
        return Some("image/svg+xml");
    }

    let ct = content_type
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    match ct.as_str() {
        "image/jpeg" | "image/jpg" => Some("image/jpeg"),
        "image/png" => Some("image/png"),
        "image/gif" => Some("image/gif"),
        "image/webp" => Some("image/webp"),
        "image/avif" => Some("image/avif"),
        "image/bmp" => Some("image/bmp"),
        "image/x-icon" | "image/vnd.microsoft.icon" => Some("image/x-icon"),
        "image/svg+xml" => Some("image/svg+xml"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data_map(url: &str, value: &str) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert(url.to_string(), value.to_string());
        map
    }

    #[test]
    fn rewrites_images_and_keeps_alt() {
        let md = "前文\n\n![封面图](https://example.com/a.png \"标题\")\n\n后文";
        let out = rewrite_images(
            md,
            &data_map("https://example.com/a.png", "data:image/png;base64,AAA"),
        );
        assert!(out.contains("![封面图](data:image/png;base64,AAA)"));
        assert!(!out.contains("https://example.com/a.png"));
    }

    #[test]
    fn keeps_unresolved_images() {
        let md = "![x](https://example.com/b.png)";
        let out = rewrite_images(md, &data_map("https://example.com/other.png", "data:x"));
        assert_eq!(out, md);
    }

    #[test]
    fn ignores_links_and_data_uris() {
        let md = "[链接](https://example.com) ![x](data:image/png;base64,AAA)";
        let out = rewrite_images(md, &data_map("https://example.com", "data:bad"));
        assert_eq!(out, md);
    }

    #[test]
    fn detects_mime_by_magic_bytes() {
        assert_eq!(
            image_mime("application/octet-stream", &[0xFF, 0xD8, 0xFF, 0xE0]),
            Some("image/jpeg")
        );
        assert_eq!(
            image_mime(
                "application/octet-stream",
                &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0]
            ),
            Some("image/png")
        );
        assert_eq!(
            image_mime("text/plain", b"<?xml version=\"1.0\"?><svg></svg>"),
            Some("image/svg+xml")
        );
        assert_eq!(image_mime("text/html", b"<html></html>"), None);
    }
}
