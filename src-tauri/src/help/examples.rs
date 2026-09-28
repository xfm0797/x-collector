use rust_embed::RustEmbed;
use serde::Serialize;

#[derive(RustEmbed)]
#[folder = "../docs/examples/"]
struct ExampleAssets;

#[derive(Serialize, Clone)]
pub struct ConfigExample {
    pub id: String,
    pub platform: String,
    pub title: String,
    pub description: String,
    pub content: String,
    pub notes: Vec<String>,
}

/// 示例元数据（id → 平台 / 标题 / 描述 / 注意事项）
fn metadata(id: &str) -> (&'static str, &'static str, &'static str, Vec<&'static str>) {
    match id {
        "wordpress-config" => (
            "wordpress",
            "WordPress 自动发布配置",
            "通过 XML-RPC 接口将文章自动发布到 WordPress 站点。",
            vec![
                "推荐使用「应用密码」而非登录密码：用户 → 个人资料 → 应用密码 生成",
                "API 路径通常为 /xmlrpc.php",
                "确保站点已启用 XML-RPC 协议（部分安全插件可能默认禁用）",
                "站点必须使用 HTTPS，应用密码仅在 HTTPS 下生效",
            ],
        ),
        "typecho-config" => (
            "typecho",
            "Typecho 自动发布配置",
            "通过 XML-RPC 接口将文章自动发布到 Typecho 博客。",
            vec![
                "API 路径为 /action/xmlrpc",
                "需要在 Typecho 后台开启 XML-RPC 扩展（默认开启）",
                "用户名为登录用户名，密码为登录密码",
            ],
        ),
        "hexo-config" => (
            "hexo",
            "Hexo 导出配置",
            "将采集的文章导出为带 Front-matter 的 Markdown 文件，放入 Hexo 博客。",
            vec![
                "post_dir 建议指向 Hexo 的 source/_posts 目录",
                "导出后执行 hexo generate 即可生成静态页面",
                "文件命名采用「日期-标题.md」格式，与 Hexo 约定一致",
                "Front-matter 自动包含 title/date/tags/categories 与原文 source_url",
            ],
        ),
        "zblog-config" => (
            "zblog",
            "Z-Blog 自动发布配置",
            "通过 XML-RPC 接口将文章自动发布到 Z-BlogPHP 站点。",
            vec![
                "API 路径为 /zb_system/xml-rpc/index.php",
                "需要在 Z-Blog 后台开启「XML-RPC 发布协议」插件",
            ],
        ),
        "keyword-examples" => (
            "keyword",
            "关键词采集任务示例",
            "常见的关键词采集任务配置示例，可参考快速搭建自己的采集矩阵。",
            vec![
                "max_pages 控制采集深度（1-10 页），越大越全但耗时越长",
                "site_limit 限定站点范围，如 csdn.net",
                "interval_minutes 大于 0 时按间隔自动轮询采集",
            ],
        ),
        "rewrite-examples" => (
            "rewrite",
            "伪原创配置示例",
            "不同强度的伪原创配置组合示例。",
            vec![
                "同义词替换比例建议 20%-40%，过高会影响可读性",
                "代码块与表格不会被改写，可放心采集技术文章",
                "可在设置中导入自定义同义词词库",
            ],
        ),
        _ => ("custom", "自定义配置", "自定义配置示例。", vec![]),
    }
}

/// 获取配置示例列表（可按平台过滤）
pub fn get_examples(platform: Option<&str>) -> Vec<ConfigExample> {
    let mut result = Vec::new();
    for file in ExampleAssets::iter() {
        let id = file.to_string();
        let id = id.trim_end_matches(".json");
        let (plat, title, description, notes) = metadata(id);
        if let Some(p) = platform.filter(|p| !p.is_empty() && *p != "all") {
            if plat != p {
                continue;
            }
        }
        if let Some(asset) = ExampleAssets::get(&format!("{}.json", id)) {
            let content = pretty_json(&String::from_utf8_lossy(asset.data.as_ref()));
            result.push(ConfigExample {
                id: id.to_string(),
                platform: plat.to_string(),
                title: title.to_string(),
                description: description.to_string(),
                content,
                notes: notes.into_iter().map(|s| s.to_string()).collect(),
            });
        }
    }
    result
}

/// JSON 美化（便于展示）
fn pretty_json(raw: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(raw) {
        Ok(v) => serde_json::to_string_pretty(&v).unwrap_or_else(|_| raw.to_string()),
        Err(_) => raw.to_string(),
    }
}
