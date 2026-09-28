use serde::Serialize;

use crate::help::{examples, faq, manual, search};

/// 手册目录
#[tauri::command]
pub fn get_manual_index() -> Vec<manual::ManualSection> {
    manual::get_index()
}

/// 手册章节内容
#[tauri::command]
pub fn get_manual_section(path: String) -> Result<manual::ManualContent, String> {
    manual::get_section(&path).ok_or_else(|| format!("未找到手册章节：{path}"))
}

/// 配置示例列表
#[tauri::command]
pub fn get_examples(platform: Option<String>) -> Vec<examples::ConfigExample> {
    examples::get_examples(platform.as_deref())
}

/// FAQ 分类
#[tauri::command]
pub fn get_faq_categories() -> Vec<faq::FaqCategory> {
    faq::get_categories()
}

/// FAQ 列表
#[tauri::command]
pub fn get_faqs(category: Option<String>) -> Vec<faq::FaqItem> {
    faq::get_faqs(category.as_deref())
}

/// 帮助全文搜索
#[tauri::command]
pub fn search_help(query: String) -> Vec<search::SearchResult> {
    search::search(&query)
}

/// 快速开始引导
#[tauri::command]
pub fn get_quick_start() -> QuickStartGuide {
    QuickStartGuide {
        steps: vec![
            QuickStartStep {
                step: 1,
                title: "添加关键词任务".to_string(),
                description: "进入「关键词采集」页面，添加关键词并选择搜索引擎与采集深度。".to_string(),
                action: "/keywords".to_string(),
            },
            QuickStartStep {
                step: 2,
                title: "执行采集".to_string(),
                description: "点击任务右侧的「采集」按钮，等待采集完成，文章自动入库并完成清洗。".to_string(),
                action: "/keywords".to_string(),
            },
            QuickStartStep {
                step: 3,
                title: "发布文章".to_string(),
                description: "进入「发布管理」选择待发布文章，发布到 Hexo / WordPress / Typecho。".to_string(),
                action: "/publish".to_string(),
            },
        ],
    }
}

#[derive(Serialize)]
pub struct QuickStartGuide {
    pub steps: Vec<QuickStartStep>,
}

#[derive(Serialize)]
pub struct QuickStartStep {
    pub step: i32,
    pub title: String,
    pub description: String,
    pub action: String,
}

/// 更新日志
#[tauri::command]
pub fn get_changelog() -> String {
    manual::get_changelog()
}

/// 快捷键列表
#[tauri::command]
pub fn get_shortcuts() -> Vec<ShortcutItem> {
    vec![
        ShortcutItem { key: "Ctrl+Shift+K".into(), description: "打开关键词采集页面".into(), category: "导航".into() },
        ShortcutItem { key: "Ctrl+Shift+A".into(), description: "打开文章库".into(), category: "导航".into() },
        ShortcutItem { key: "Ctrl+Shift+P".into(), description: "打开发布管理".into(), category: "导航".into() },
        ShortcutItem { key: "Ctrl+Shift+H".into(), description: "打开帮助中心".into(), category: "导航".into() },
        ShortcutItem { key: "Ctrl+S".into(), description: "保存当前编辑的文章".into(), category: "编辑".into() },
        ShortcutItem { key: "Ctrl+F".into(), description: "搜索（列表页）".into(), category: "编辑".into() },
        ShortcutItem { key: "Esc".into(), description: "关闭弹窗".into(), category: "通用".into() },
    ]
}

#[derive(Serialize)]
pub struct ShortcutItem {
    pub key: String,
    pub description: String,
    pub category: String,
}

/// 错误码列表
#[tauri::command]
pub fn get_error_codes() -> Vec<ErrorCode> {
    vec![
        ErrorCode {
            code: "1001".into(),
            name: "CMS 连接失败".into(),
            description: "无法连接到 CMS 站点的 XML-RPC 接口".into(),
            causes: vec!["站点地址或 API 路径错误".into(), "网络不通或被防火墙拦截".into(), "站点禁用了 XML-RPC".into()],
            solutions: vec!["核对站点地址与 API 路径（WordPress 为 /xmlrpc.php）".into(), "在浏览器中访问该路径确认返回 XML".into(), "检查站点安全插件是否禁用了 XML-RPC".into()],
        },
        ErrorCode {
            code: "1002".into(),
            name: "CMS 认证失败".into(),
            description: "XML-RPC 返回认证错误（403 / fault）".into(),
            causes: vec!["用户名或密码错误".into(), "WordPress 需要使用应用密码而非登录密码".into(), "账户被限制 XML-RPC 权限".into()],
            solutions: vec!["WordPress 在「用户 → 个人资料 → 应用密码」生成新密码".into(), "确认站点使用 HTTPS".into(), "检查账户权限".into()],
        },
        ErrorCode {
            code: "2001".into(),
            name: "采集请求失败".into(),
            description: "目标页面请求失败或返回非 2xx 状态".into(),
            causes: vec!["目标站反爬（403 / 429）".into(), "网络超时".into(), "URL 无效".into()],
            solutions: vec!["降低采集频率（请求间隔 2-5 秒）".into(), "开启代理 IP 轮换".into(), "检查 URL 是否可访问".into()],
        },
        ErrorCode {
            code: "2002".into(),
            name: "正文提取失败".into(),
            description: "无法从页面中提取有效正文内容".into(),
            causes: vec!["页面为纯 JS 渲染（SPA）".into(), "正文选择器不匹配".into(), "页面结构特殊".into()],
            solutions: vec!["在采集源中配置自定义 CSS 选择器".into(), "使用站点适配支持的站点".into(), "查看帮助手册「自定义选择器」章节".into()],
        },
        ErrorCode {
            code: "3001".into(),
            name: "数据库写入失败".into(),
            description: "文章写入本地 SQLite 数据库失败".into(),
            causes: vec!["磁盘空间不足".into(), "数据库文件被占用".into()],
            solutions: vec!["检查磁盘空间".into(), "重启应用".into()],
        },
    ]
}

#[derive(Serialize)]
pub struct ErrorCode {
    pub code: String,
    pub name: String,
    pub description: String,
    pub causes: Vec<String>,
    pub solutions: Vec<String>,
}

/// 检查 GitHub 最新版本
#[tauri::command]
pub async fn check_update() -> UpdateInfo {
    let current_version = env!("CARGO_PKG_VERSION").to_string();
    let default = UpdateInfo {
        has_update: false,
        latest_version: current_version.clone(),
        current_version: current_version.clone(),
        download_url: "https://github.com/xfm0797/x-collector/releases".to_string(),
        changelog: String::new(),
    };

    let client = match reqwest::Client::builder()
        .user_agent("x-collector")
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(_) => return default,
    };
    let Ok(resp) = client
        .get("https://api.github.com/repos/xfm0797/x-collector/releases/latest")
        .send()
        .await
    else {
        return default;
    };
    let Ok(json) = resp.json::<serde_json::Value>().await else {
        return default;
    };

    let tag = json.get("tag_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let url = json
        .get("html_url")
        .and_then(|v| v.as_str())
        .unwrap_or("https://github.com/xfm0797/x-collector/releases")
        .to_string();
    let changelog = json
        .get("body")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    if tag.is_empty() {
        return default;
    }
    let latest = tag.trim_start_matches('v').to_string();
    UpdateInfo {
        has_update: latest != current_version,
        latest_version: latest,
        current_version,
        download_url: url,
        changelog,
    }
}

#[derive(Serialize)]
pub struct UpdateInfo {
    pub has_update: bool,
    pub latest_version: String,
    pub current_version: String,
    pub download_url: String,
    pub changelog: String,
}

/// 打开外部链接（浏览器）
#[tauri::command]
pub fn open_external_link(url: String) -> Result<(), String> {
    // 仅允许 http/https 链接，防止命令注入
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("仅支持打开 http/https 链接".to_string());
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/c", "start", "", &url])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(&url).spawn().map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(&url).spawn().map_err(|e| e.to_string())?;
    }
    Ok(())
}
