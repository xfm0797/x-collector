//! 采集后的 Markdown 规范化：去除重复标题、不可见字符、冗余空行等

use once_cell::sync::Lazy;
use regex::Regex;

/// 空链接文本（`[](url)` / `[ ](url)`）
static EMPTY_LINK_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[\s*\]\([^)]*\)").unwrap());
/// 整行为一张图片
static IMAGE_LINE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*!\[[^\]]*\]\([^)]*\)\s*$").unwrap());

/// 对采集到的 Markdown 做规范化处理
pub fn normalize_markdown(md: &str, title: &str) -> String {
    // 换行与不可见字符
    let text = md
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace(['\u{200b}', '\u{200c}', '\u{200d}', '\u{feff}'], "")
        .replace('\u{00a0}', " ")
        .replace('\u{3000}', " ");

    let mut lines: Vec<String> = text.lines().map(|l| l.trim_end().to_string()).collect();

    remove_duplicate_title(&mut lines, title);

    // 清理空链接残留
    for line in &mut lines {
        if EMPTY_LINK_RE.is_match(line) {
            *line = EMPTY_LINK_RE.replace_all(line, "").trim().to_string();
        }
    }

    // 连续重复的图片行只保留一张
    lines.dedup_by(|a, b| {
        IMAGE_LINE_RE.is_match(a) && IMAGE_LINE_RE.is_match(b) && a.trim() == b.trim()
    });

    // 合并连续空行，去掉首尾空行
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut blank_run = 0usize;
    for line in lines {
        if line.trim().is_empty() {
            blank_run += 1;
            if blank_run > 1 {
                continue;
            }
            out.push(String::new());
        } else {
            blank_run = 0;
            out.push(line);
        }
    }
    while out.first().map(|l| l.trim().is_empty()).unwrap_or(false) {
        out.remove(0);
    }
    while out.last().map(|l| l.trim().is_empty()).unwrap_or(false) {
        out.pop();
    }

    close_unclosed_fence(out.join("\n"))
}

/// 去掉与文章标题重复的首个一级标题
fn remove_duplicate_title(lines: &mut Vec<String>, title: &str) {
    let title = normalize_for_compare(title);
    if title.is_empty() {
        return;
    }
    let Some(pos) = lines.iter().position(|l| !l.trim().is_empty()) else {
        return;
    };
    let Some(heading) = lines[pos].trim().strip_prefix("# ") else {
        return;
    };
    if normalize_for_compare(heading) != title {
        return;
    }
    lines.remove(pos);
    if lines.get(pos).map(|l| l.trim().is_empty()).unwrap_or(false) {
        lines.remove(pos);
    }
}

/// 比较标题时忽略空白与大小写
fn normalize_for_compare(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect()
}

/// 代码围栏缺失结束标记时补齐，避免后续内容被渲染成代码
fn close_unclosed_fence(mut text: String) -> String {
    let fences = text
        .lines()
        .filter(|l| l.trim_start().starts_with("```"))
        .count();
    if fences % 2 == 1 {
        if !text.ends_with('\n') {
            text.push('\n');
        }
        text.push_str("```");
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_duplicate_title_and_blank_lines() {
        let md = "\n# 我的标题\n\n\n\n正文第一段。\n\n\n正文第二段。\n";
        let out = normalize_markdown(md, "我的标题");
        assert_eq!(out, "正文第一段。\n\n正文第二段。");
    }

    #[test]
    fn keeps_different_heading() {
        let md = "# 小标题\n\n正文";
        let out = normalize_markdown(md, "文章标题");
        assert!(out.starts_with("# 小标题"));
    }

    #[test]
    fn strips_invisible_chars_and_empty_links() {
        let md = "正文\u{200b}内容 [ ](https://example.com) 继续\u{00a0}阅读";
        let out = normalize_markdown(md, "");
        assert_eq!(out, "正文内容  继续 阅读");
    }

    #[test]
    fn dedupes_repeated_images() {
        let md = "![图](https://example.com/a.png)\n![图](https://example.com/a.png)\n\n正文";
        let out = normalize_markdown(md, "");
        assert_eq!(out, "![图](https://example.com/a.png)\n\n正文");
    }

    #[test]
    fn closes_unclosed_code_fence() {
        let md = "说明\n\n```rust\nfn main() {}\n";
        let out = normalize_markdown(md, "");
        assert!(out.ends_with("```"));
        assert_eq!(out.matches("```").count(), 2);
    }
}
