use rand::prelude::*;
use serde::{Deserialize, Serialize};

use super::sentence_rewriter;
use super::synonym_dict;

/// 伪原创选项
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RewriteOptions {
    pub enabled: bool,
    /// light / medium / heavy
    pub intensity: String,
    /// 同义词替换比例 10-50（百分比）
    pub synonym_ratio: i32,
    /// 句子改写比例 10-50（百分比）
    pub sentence_ratio: i32,
    pub paragraph_shuffle: bool,
    pub rewrite_ends: bool,
    pub keywords: Vec<String>,
}

impl Default for RewriteOptions {
    fn default() -> Self {
        Self {
            enabled: false,
            intensity: "medium".to_string(),
            synonym_ratio: 30,
            sentence_ratio: 20,
            paragraph_shuffle: false,
            rewrite_ends: false,
            keywords: vec![],
        }
    }
}

/// 强度对应的比例系数（在用户设置基础上按强度缩放）
fn intensity_factor(intensity: &str) -> f64 {
    match intensity {
        "light" => 0.7,
        "heavy" => 1.3,
        _ => 1.0,
    }
}

/// 首段模板库
const OPENING_TEMPLATES: &[&str] = &[
    "在当今快速发展的技术环境中，{topic}是许多开发者关注的焦点。",
    "随着行业需求的不断演进，理解{topic}变得愈发重要。",
    "本文将围绕{topic}展开，梳理其中的关键要点与实践经验。",
    "如果你正在学习{topic}，这篇文章或许能为你提供清晰的思路。",
];

/// 尾段模板库
const CLOSING_TEMPLATES: &[&str] = &[
    "总的来说，掌握{topic}需要理论与实践的结合，希望本文对你有所帮助。",
    "以上便是关于{topic}的核心内容，欢迎在实际项目中尝试应用。",
    "关于{topic}仍有更多值得深入探索的空间，期待与你继续交流。",
    "如果本文对你有所启发，不妨收藏起来以便日后查阅。",
];

/// Markdown 伪原创主流程：跳过代码块 / 表格 / 标题，仅改写正文段落
pub fn rewrite_markdown(md: &str, opts: &RewriteOptions, custom_dict: &[Vec<String>]) -> String {
    if !opts.enabled {
        return md.to_string();
    }

    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(42);

    let factor = intensity_factor(&opts.intensity);
    let synonym_ratio = (opts.synonym_ratio as f64 / 100.0 * factor).clamp(0.05, 0.8);
    let sentence_ratio = (opts.sentence_ratio as f64 / 100.0 * factor).clamp(0.05, 0.8);

    let blocks = split_blocks(md);

    // 段落重排（保留首尾段）
    let paragraph_indices: Vec<usize> = blocks
        .iter()
        .enumerate()
        .filter(|(_, b)| is_paragraph(b))
        .map(|(i, _)| i)
        .collect();
    let shuffle_map = if opts.paragraph_shuffle && paragraph_indices.len() > 3 {
        let mut rng = StdRng::seed_from_u64(seed);
        // 保留首尾，中间打乱
        let mid = &paragraph_indices[1..paragraph_indices.len() - 1];
        let mut shuffled_mid = mid.to_vec();
        shuffled_mid.shuffle(&mut rng);
        let mut map = std::collections::HashMap::new();
        for (old, new) in mid.iter().zip(shuffled_mid.iter()) {
            map.insert(*old, *new);
        }
        map
    } else {
        std::collections::HashMap::new()
    };

    let mut result: Vec<String> = Vec::new();
    for (i, block) in blocks.iter().enumerate() {
        if is_paragraph(block) {
            let source = match shuffle_map.get(&i) {
                Some(&src) => &blocks[src],
                None => block,
            };
            let mut rewritten = rewrite_paragraph(source, synonym_ratio, sentence_ratio, seed + i as u64, custom_dict);
            rewritten = insert_keywords(&rewritten, &opts.keywords, seed + i as u64);
            // 首尾段重写
            if opts.rewrite_ends {
                let paragraph_pos = result.iter().filter(|b| is_paragraph(b)).count();
                let total_paragraphs = paragraph_indices.len();
                if paragraph_pos == 0 {
                    rewritten = rewrite_end_paragraph(&rewritten, OPENING_TEMPLATES, seed);
                } else if paragraph_pos + 1 == total_paragraphs {
                    rewritten = rewrite_end_paragraph(&rewritten, CLOSING_TEMPLATES, seed);
                }
            }
            result.push(rewritten);
        } else {
            result.push(block.clone());
        }
    }

    result.join("\n\n")
}

/// 改写单个段落：同义词替换 + 句子改写
fn rewrite_paragraph(
    text: &str,
    synonym_ratio: f64,
    sentence_ratio: f64,
    seed: u64,
    custom_dict: &[Vec<String>],
) -> String {
    let mut out = synonym_dict::replace_synonyms(text, synonym_ratio, seed, custom_dict);
    out = sentence_rewriter::rewrite_sentences(&out, sentence_ratio, seed);
    out
}

/// 自然插入关键词：在段落中部合适位置插入一次
fn insert_keywords(text: &str, keywords: &[String], seed: u64) -> String {
    if keywords.is_empty() {
        return text.to_string();
    }
    let mut rng = StdRng::seed_from_u64(seed);
    let sentences = sentence_rewriter::split_sentences(text);
    let len = sentences.len();
    if len < 3 {
        return text.to_string();
    }
    let kw = &keywords[rng.gen_range(0..keywords.len())];
    // 检查是否已包含该关键词
    if text.contains(kw.as_str()) {
        return text.to_string();
    }
    let pos = rng.gen_range(1..len);
    let mut result: Vec<String> = Vec::new();
    for (i, s) in sentences.iter().enumerate() {
        if i == pos && s.chars().count() > 20 {
            // 在该句第二个逗号后插入关键词短语
            let parts: Vec<&str> = s.splitn(3, '，').collect();
            if parts.len() == 3 {
                result.push(format!("{}，{}（涉及{}）", parts[0], parts[1], kw));
                continue;
            }
        }
        result.push(s.clone());
    }
    result.join("")
}

/// 用模板重写首/尾段
fn rewrite_end_paragraph(original: &str, templates: &[&str], seed: u64) -> String {
    let mut rng = StdRng::seed_from_u64(seed);
    let template = templates[rng.gen_range(0..templates.len())];
    // 从原段落提取主题（取出现频率较高的名词性短语，简化为前 12 字）
    let topic: String = original
        .chars()
        .filter(|c| !c.is_whitespace())
        .take(12)
        .collect();
    let rewritten = template.replace("{topic}", &topic);
    // 保留原文核心内容（截断至 200 字）附于模板之后
    let original_tail: String = original.chars().take(200).collect();
    format!("{}\n\n{}", rewritten, original_tail)
}

/// 将 Markdown 切分为块（段落 / 代码块 / 表格 / 标题 / 列表），代码块整体保留
fn split_blocks(md: &str) -> Vec<String> {
    let mut blocks: Vec<String> = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    let mut in_code = false;

    for line in md.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            if !in_code && !current.is_empty() {
                blocks.push(current.join("\n"));
                current.clear();
            }
            in_code = !in_code;
            current.push(line);
            if !in_code {
                blocks.push(current.join("\n"));
                current.clear();
            }
            continue;
        }
        if in_code {
            current.push(line);
            continue;
        }
        if trimmed.is_empty() {
            if !current.is_empty() {
                blocks.push(current.join("\n"));
                current.clear();
            }
        } else {
            current.push(line);
        }
    }
    if !current.is_empty() {
        blocks.push(current.join("\n"));
    }
    blocks
}

/// 判断是否为可改写的正文段落（非标题 / 非列表 / 非表格 / 非代码 / 非图片行）
fn is_paragraph(block: &str) -> bool {
    let first = block.lines().next().unwrap_or("");
    let trimmed = first.trim_start();
    !(trimmed.starts_with('#')
        || trimmed.starts_with('|')
        || trimmed.starts_with("- ")
        || trimmed.starts_with("* ")
        || trimmed.starts_with("> ")
        || trimmed.starts_with("```")
        || trimmed.starts_with('!')
        || trimmed.starts_with("---")
        || trimmed.chars().count() < 20)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rewrite_preserves_code() {
        let md = "这是一个足够长的中文段落，用于测试伪原创功能是否正常工作。\n\n```rust\nfn main() {}\n```\n\n另一个足够长的中文段落，其中包含一些可以替换的词汇，例如方法和工具。";
        let opts = RewriteOptions {
            enabled: true,
            intensity: "heavy".to_string(),
            synonym_ratio: 100,
            sentence_ratio: 100,
            ..Default::default()
        };
        let out = rewrite_markdown(md, &opts, &[]);
        assert!(out.contains("```rust"));
        assert!(out.contains("fn main() {}"));
    }
}
