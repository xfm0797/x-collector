use rust_embed::RustEmbed;
use serde::Serialize;

#[derive(RustEmbed)]
#[folder = "../docs/manual/"]
struct ManualAssets;

#[derive(Serialize, Clone)]
pub struct ManualSection {
    pub id: String,
    pub title: String,
    pub path: String,
    pub children: Vec<ManualSection>,
    pub level: u32,
}

#[derive(Serialize)]
pub struct ManualContent {
    pub path: String,
    pub title: String,
    pub content: String,
    pub prev: Option<NavItem>,
    pub next: Option<NavItem>,
}

#[derive(Serialize, Clone)]
pub struct NavItem {
    pub title: String,
    pub path: String,
}

/// 一级章节顺序（用于上一页 / 下一页导航）
const SECTION_ORDER: &[(&str, &str)] = &[
    ("index", "手册首页"),
    ("quickstart", "快速开始"),
    ("keyword-collect", "关键词采集"),
    ("source-manager", "采集源管理"),
    ("content-cleaning", "内容清洗"),
    ("rewrite", "伪原创"),
    ("publish", "发布配置"),
    ("cms-config", "CMS 连接配置"),
    ("settings", "系统设置"),
    ("troubleshooting", "故障排除"),
];

fn section(id: &str, title: &str, children: Vec<(&str, &str)>) -> ManualSection {
    ManualSection {
        id: id.to_string(),
        title: title.to_string(),
        path: id.to_string(),
        children: children
            .into_iter()
            .map(|(cid, ctitle)| ManualSection {
                id: cid.to_string(),
                title: ctitle.to_string(),
                path: format!("{}#{}", id, cid),
                children: vec![],
                level: 2,
            })
            .collect(),
        level: 1,
    }
}

/// 获取手册目录结构
pub fn get_index() -> Vec<ManualSection> {
    vec![
        section("index", "手册首页", vec![]),
        section(
            "quickstart",
            "快速开始",
            vec![
                ("install", "安装与启动"),
                ("first-config", "首次配置向导"),
                ("first-task", "第一条采集任务"),
            ],
        ),
        section(
            "keyword-collect",
            "关键词采集",
            vec![
                ("add-keyword", "添加关键词"),
                ("search-engine", "搜索引擎配置"),
                ("match-rules", "匹配规则设置"),
                ("schedule", "定时任务配置"),
                ("dedup", "去重策略说明"),
            ],
        ),
        section(
            "source-manager",
            "采集源管理",
            vec![
                ("rss", "RSS 订阅源"),
                ("single-page", "单页采集"),
                ("sitemap", "站点地图采集"),
                ("custom-selectors", "自定义选择器"),
            ],
        ),
        section(
            "content-cleaning",
            "内容清洗",
            vec![
                ("ad-filter", "广告过滤"),
                ("images", "图片处理"),
                ("format", "格式优化"),
                ("custom-rules", "自定义规则"),
            ],
        ),
        section(
            "rewrite",
            "伪原创",
            vec![
                ("synonyms", "同义词替换"),
                ("sentences", "句子改写"),
                ("paragraphs", "段落重排"),
                ("custom-dict", "自定义词库"),
            ],
        ),
        section(
            "publish",
            "发布配置",
            vec![
                ("hexo", "Hexo 导出"),
                ("wordpress", "WordPress 发布"),
                ("typecho", "Typecho 发布"),
                ("zblog", "Z-Blog 发布"),
            ],
        ),
        section(
            "cms-config",
            "CMS 连接配置",
            vec![("connections", "连接管理"), ("security", "密码与安全")],
        ),
        section(
            "settings",
            "系统设置",
            vec![
                ("collect-settings", "采集设置"),
                ("proxy-settings", "代理设置"),
                ("anti-crawl", "反爬设置"),
            ],
        ),
        section(
            "troubleshooting",
            "故障排除",
            vec![
                ("error-codes", "常见错误码"),
                ("network", "网络问题"),
                ("parse", "解析问题"),
                ("publish-fail", "发布失败处理"),
            ],
        ),
    ]
}

/// 获取手册章节内容
pub fn get_section(path: &str) -> Option<ManualContent> {
    let base = path.split('#').next().unwrap_or("index");
    let filename = format!("{}.md", base);
    let asset = ManualAssets::get(&filename)?;
    let content = String::from_utf8_lossy(asset.data.as_ref()).to_string();
    let title = extract_title(&content);

    let (prev, next) = get_nav(base);
    Some(ManualContent {
        path: base.to_string(),
        title,
        content,
        prev,
        next,
    })
}

/// 提取 Markdown 一级标题作为章节标题
fn extract_title(content: &str) -> String {
    content
        .lines()
        .find(|l| l.starts_with("# "))
        .map(|l| l.trim_start_matches("# ").trim().to_string())
        .unwrap_or_default()
}

/// 获取上下页导航
fn get_nav(current: &str) -> (Option<NavItem>, Option<NavItem>) {
    let pos = SECTION_ORDER.iter().position(|(p, _)| *p == current);
    match pos {
        Some(i) => {
            let prev = if i > 0 {
                Some(NavItem {
                    title: SECTION_ORDER[i - 1].1.to_string(),
                    path: SECTION_ORDER[i - 1].0.to_string(),
                })
            } else {
                None
            };
            let next = if i < SECTION_ORDER.len() - 1 {
                Some(NavItem {
                    title: SECTION_ORDER[i + 1].1.to_string(),
                    path: SECTION_ORDER[i + 1].0.to_string(),
                })
            } else {
                None
            };
            (prev, next)
        }
        None => (None, None),
    }
}

/// 获取全部手册资源（用于全文搜索）
pub fn all_sections() -> Vec<(String, String)> {
    let mut result = Vec::new();
    for (path, title) in SECTION_ORDER {
        let filename = format!("{}.md", path);
        if let Some(asset) = ManualAssets::get(&filename) {
            let content = String::from_utf8_lossy(asset.data.as_ref()).to_string();
            result.push((title.to_string(), content));
        }
    }
    result
}

/// 内嵌 CHANGELOG
#[derive(RustEmbed)]
#[folder = "../docs/"]
struct RootDocs;

pub fn get_changelog() -> String {
    RootDocs::get("CHANGELOG.md")
        .map(|a| String::from_utf8_lossy(a.data.as_ref()).to_string())
        .unwrap_or_else(|| "# 暂无更新日志".to_string())
}
