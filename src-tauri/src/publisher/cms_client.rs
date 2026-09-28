/// 极简 XML-RPC 客户端（基于 reqwest，手工构造/解析 XML）
use super::super::collector::rss_collector::decode_entities;

/// XML-RPC 参数值
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum RpcValue {
    Str(String),
    Int(i64),
    Bool(bool),
    Array(Vec<RpcValue>),
    Struct(Vec<(String, RpcValue)>),
}

impl RpcValue {
    fn to_xml(&self) -> String {
        match self {
            RpcValue::Str(s) => format!("<value><string><![CDATA[{}]]></string></value>", escape_cdata(s)),
            RpcValue::Int(i) => format!("<value><int>{}</int></value>", i),
            RpcValue::Bool(b) => format!("<value><boolean>{}</boolean></value>", if *b { 1 } else { 0 }),
            RpcValue::Array(items) => format!(
                "<value><array><data>{}</data></array></value>",
                items.iter().map(|v| format!("<param>{}</param>", v.to_xml())).collect::<String>()
            ),
            RpcValue::Struct(fields) => {
                let members: String = fields
                    .iter()
                    .map(|(k, v)| format!("<member><name>{}</name>{}</member>", xml_escape(k), v.to_xml()))
                    .collect();
                format!("<value><struct>{}</struct></value>", members)
            }
        }
    }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// CDATA 内容中的 "]]>" 需拆分转义
fn escape_cdata(s: &str) -> String {
    s.replace("]]>", "]]]]><![CDATA[>")
}

/// 构造 XML-RPC 请求报文
pub fn build_request(method: &str, params: &[RpcValue]) -> String {
    let params_xml: String = params
        .iter()
        .map(|p| format!("<param>{}</param>", p.to_xml()))
        .collect();
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><methodCall><methodName>{}</methodName><params>{}</params></methodCall>",
        method,
        params_xml
    )
}

/// 调用 XML-RPC 接口，返回原始响应文本
pub async fn call(endpoint: &str, username: &str, password: &str, body: &str) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("x-collector/0.0.1 (XML-RPC)")
        .build()
        .map_err(|e| format!("HTTP 客户端构建失败：{e}"))?;

    let resp = client
        .post(endpoint)
        .header("Content-Type", "text/xml")
        .basic_auth(username, Some(password))
        .body(body.to_string())
        .send()
        .await
        .map_err(|e| format!("请求失败：{e}"))?;

    let status = resp.status();
    let text = resp.text().await.map_err(|e| format!("读取响应失败：{e}"))?;
    if !status.is_success() {
        return Err(format!("HTTP {}", status.as_u16()));
    }
    Ok(text)
}

/// 从响应中提取 fault 错误信息（若存在）
pub fn extract_fault(response: &str) -> Option<String> {
    if !response.contains("<fault>") {
        return None;
    }
    // 提取 faultString
    let start = response.find("faultString")?;
    let segment = &response[start..];
    let value_start = segment.find("<value>")? + "<value>".len();
    let value_end = segment[value_start..].find("</value>")? + value_start;
    Some(decode_entities(&strip_tags(&segment[value_start..value_end])))
}

/// 从响应中提取第一个 string/int 值
pub fn extract_value(response: &str) -> Option<String> {
    for tag in ["string", "int", "i4", "boolean"] {
        if let Some(start) = response.find(&format!("<{tag}>")) {
            let content_start = start + tag.len() + 2;
            if let Some(end) = response[content_start..].find(&format!("</{tag}>")) {
                let raw = &response[content_start..content_start + end];
                return Some(decode_entities(&strip_tags(raw)));
            }
        }
        // CDATA 形式
        if let Some(start) = response.find("<string><![CDATA[") {
            let content_start = start + "<string><![CDATA[".len();
            if let Some(end) = response[content_start..].find("]]></string>") {
                return Some(response[content_start..content_start + end].to_string());
            }
        }
    }
    None
}

/// 提取响应中全部 string 值（用于 struct 字段提取）
pub fn extract_struct_string(response: &str, member_name: &str) -> Option<String> {
    let needle = format!("<name>{}</name>", member_name);
    let pos = response.find(&needle)?;
    let segment = &response[pos..];
    if let Some(vstart) = segment.find("<string>") {
        let content_start = vstart + "<string>".len();
        if let Some(vend) = segment[content_start..].find("</string>") {
            return Some(decode_entities(&segment[content_start..content_start + vend]));
        }
    }
    if let Some(vstart) = segment.find("<string><![CDATA[") {
        let content_start = vstart + "<string><![CDATA[".len();
        if let Some(vend) = segment[content_start..].find("]]></string>") {
            return Some(segment[content_start..content_start + vend].to_string());
        }
    }
    None
}

fn strip_tags(s: &str) -> String {
    once_cell_regex_replace(s)
}

fn once_cell_regex_replace(s: &str) -> String {
    // 移除残留标签
    let mut out = String::new();
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out
}
