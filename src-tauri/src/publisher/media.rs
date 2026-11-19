//! 发布前媒体处理：把正文中内嵌的 data URI 图片上传到 CMS 媒体库

use std::collections::HashMap;

use base64::Engine;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::utils::hash::sha256_hex;

use super::cms_client::{self, RpcValue};

/// 单张图片解码后大小上限（发布阶段）
const MAX_UPLOAD_BYTES: usize = 10 * 1024 * 1024;

static DATA_IMAGE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"!\[([^\]]*)\]\((data:(image/[a-zA-Z0-9.+-]+);base64,([A-Za-z0-9+/=]+))\)"#).unwrap()
});

/// 上传 Markdown 中所有内嵌图片，成功者替换为 CMS 返回的远程地址
pub async fn upload_embedded_images(
    md: &str,
    endpoint: &str,
    username: &str,
    password: &str,
) -> String {
    let mut pending: Vec<(String, String, String)> = Vec::new(); // (完整 data uri, mime, base64)
    for caps in DATA_IMAGE_RE.captures_iter(md) {
        let (Some(full), Some(mime), Some(b64)) = (caps.get(2), caps.get(3), caps.get(4)) else {
            continue;
        };
        let (full, mime) = (full.as_str().to_string(), mime.as_str().to_string());
        if pending.iter().any(|(f, _, _)| f == &full) {
            continue;
        }
        pending.push((full, mime, b64.as_str().to_string()));
    }
    if pending.is_empty() {
        return md.to_string();
    }

    let mut mapping: HashMap<String, String> = HashMap::new();
    for (data_uri, mime, b64) in pending {
        let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(b64.as_bytes()) else {
            continue;
        };
        if bytes.is_empty() || bytes.len() > MAX_UPLOAD_BYTES {
            continue;
        }

        let name = format!(
            "{}.{}",
            sha256_hex(&data_uri).chars().take(16).collect::<String>(),
            ext_for(&mime)
        );
        let media = RpcValue::Struct(vec![
            ("name".into(), RpcValue::Str(name)),
            ("type".into(), RpcValue::Str(mime)),
            ("bits".into(), RpcValue::Str(b64)),
        ]);
        let body = cms_client::build_request(
            "metaWeblog.newMediaObject",
            &[
                RpcValue::Str("1".to_string()),
                RpcValue::Str(username.to_string()),
                RpcValue::Str(password.to_string()),
                media,
            ],
        );

        let Ok(response) = cms_client::call(endpoint, username, password, &body).await else {
            continue;
        };
        if cms_client::extract_fault(&response).is_some() {
            continue;
        }
        if let Some(url) = extract_media_url(&response) {
            mapping.insert(data_uri, url);
        }
    }

    if mapping.is_empty() {
        return md.to_string();
    }
    rewrite_embedded(md, &mapping)
}

/// 用上传后的远程地址替换内嵌图片，保留 alt 文本
fn rewrite_embedded(md: &str, mapping: &HashMap<String, String>) -> String {
    DATA_IMAGE_RE
        .replace_all(md, |caps: &regex::Captures| {
            let alt = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let Some(full) = caps.get(2).map(|m| m.as_str()) else {
                return caps
                    .get(0)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();
            };
            match mapping.get(full) {
                Some(url) => format!("![{alt}]({url})"),
                None => caps
                    .get(0)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default(),
            }
        })
        .to_string()
}

/// 从 newMediaObject 响应中提取 url 字段
fn extract_media_url(response: &str) -> Option<String> {
    let raw = cms_client::extract_struct_string(response, "url")?;
    let url = raw
        .trim()
        .trim_start_matches("<![CDATA[")
        .trim_end_matches("]]>")
        .trim()
        .to_string();
    url.starts_with("http").then_some(url)
}

/// 常见图片 MIME 对应的扩展名
fn ext_for(mime: &str) -> &'static str {
    match mime {
        "image/jpeg" | "image/jpg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/svg+xml" => "svg",
        "image/avif" => "avif",
        "image/bmp" => "bmp",
        "image/x-icon" => "ico",
        _ => "bin",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data_uri(mime: &str, payload: &str) -> String {
        format!("data:{mime};base64,{payload}")
    }

    #[test]
    fn rewrites_only_uploaded_images() {
        let uploaded = data_uri("image/png", "AAAA");
        let kept = data_uri("image/jpeg", "BBBB");
        let md = format!("![封面]({uploaded})\n\n![保留]({kept})");
        let mut mapping = HashMap::new();
        mapping.insert(uploaded, "https://blog.example.com/wp-content/uploads/a.png".to_string());

        let out = rewrite_embedded(&md, &mapping);
        assert!(out.contains("![封面](https://blog.example.com/wp-content/uploads/a.png)"));
        assert!(out.contains(&format!("![保留]({kept})")));
    }

    #[test]
    fn extracts_url_from_xml_rpc_response() {
        let response = "<methodResponse><params><param><value><struct>\
            <member><name>url</name><value><string>https://example.com/a.png</string></value></member>\
            </struct></value></param></params></methodResponse>";
        assert_eq!(
            extract_media_url(response),
            Some("https://example.com/a.png".to_string())
        );

        let cdata = "<member><name>url</name><value><string><![CDATA[https://example.com/b.png]]></string></value></member>";
        assert_eq!(
            extract_media_url(cdata),
            Some("https://example.com/b.png".to_string())
        );

        assert_eq!(extract_media_url("<fault><name>faultCode</name></fault>"), None);
    }

    #[test]
    fn maps_common_image_extensions() {
        assert_eq!(ext_for("image/png"), "png");
        assert_eq!(ext_for("image/jpeg"), "jpg");
        assert_eq!(ext_for("image/svg+xml"), "svg");
        assert_eq!(ext_for("application/octet-stream"), "bin");
    }
}
