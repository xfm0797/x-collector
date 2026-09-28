pub mod cms_client;
pub mod hexo_exporter;
pub mod typecho_publisher;
pub mod wordpress_publisher;
pub mod zblog_publisher;

/// 轻量 Markdown → HTML 转换（用于 CMS 发布正文）
pub fn markdown_to_html(md: &str) -> String {
    let mut html = String::new();
    let mut in_code = false;
    let mut in_list = false;
    let mut in_quote = false;
    let mut paragraph: Vec<String> = Vec::new();

    let flush_paragraph = |paragraph: &mut Vec<String>, html: &mut String| {
        if !paragraph.is_empty() {
            html.push_str("<p>");
            html.push_str(&paragraph.join("\n"));
            html.push_str("</p>\n");
            paragraph.clear();
        }
    };
    let flush_list = |in_list: &mut bool, html: &mut String| {
        if *in_list {
            html.push_str("</ul>\n");
            *in_list = false;
        }
    };
    let flush_quote = |in_quote: &mut bool, html: &mut String| {
        if *in_quote {
            html.push_str("</blockquote>\n");
            *in_quote = false;
        }
    };

    for line in md.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("```") {
            flush_paragraph(&mut paragraph, &mut html);
            flush_list(&mut in_list, &mut html);
            flush_quote(&mut in_quote, &mut html);
            if in_code {
                html.push_str("</code></pre>\n");
                in_code = false;
            } else {
                let lang = trimmed.trim_start_matches('`').trim();
                html.push_str(&format!("<pre><code class=\"language-{}\">", esc(lang)));
                in_code = true;
            }
            continue;
        }
        if in_code {
            html.push_str(&esc(line));
            html.push('\n');
            continue;
        }

        // 标题
        if let Some(rest) = trimmed.strip_prefix("### ") {
            flush_paragraph(&mut paragraph, &mut html);
            html.push_str(&format!("<h3>{}</h3>\n", inline(rest)));
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("## ") {
            flush_paragraph(&mut paragraph, &mut html);
            html.push_str(&format!("<h2>{}</h2>\n", inline(rest)));
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("# ") {
            flush_paragraph(&mut paragraph, &mut html);
            html.push_str(&format!("<h1>{}</h1>\n", inline(rest)));
            continue;
        }
        // 水平线
        if trimmed == "---" || trimmed == "***" {
            flush_paragraph(&mut paragraph, &mut html);
            html.push_str("<hr/>\n");
            continue;
        }
        // 引用
        if let Some(rest) = trimmed.strip_prefix("> ") {
            flush_paragraph(&mut paragraph, &mut html);
            flush_list(&mut in_list, &mut html);
            if !in_quote {
                html.push_str("<blockquote>\n");
                in_quote = true;
            }
            html.push_str(&inline(rest));
            html.push_str("<br/>\n");
            continue;
        }
        // 无序列表
        if let Some(rest) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")) {
            flush_paragraph(&mut paragraph, &mut html);
            flush_quote(&mut in_quote, &mut html);
            if !in_list {
                html.push_str("<ul>\n");
                in_list = true;
            }
            html.push_str(&format!("<li>{}</li>\n", inline(rest)));
            continue;
        }
        // 空行
        if trimmed.is_empty() {
            flush_paragraph(&mut paragraph, &mut html);
            flush_list(&mut in_list, &mut html);
            flush_quote(&mut in_quote, &mut html);
            continue;
        }
        // 普通段落
        paragraph.push(inline(trimmed));
    }

    flush_paragraph(&mut paragraph, &mut html);
    flush_list(&mut in_list, &mut html);
    flush_quote(&mut in_quote, &mut html);
    if in_code {
        html.push_str("</code></pre>\n");
    }
    html
}

/// 行内元素：粗体 / 斜体 / 行内代码 / 链接 / 图片
fn inline(text: &str) -> String {
    use once_cell::sync::Lazy;
    use regex::Regex;
    static IMG_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"!\[([^\]]*)\]\(([^)]+)\)").unwrap());
    static LINK_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap());
    static BOLD_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\*\*([^*]+)\*\*").unwrap());
    static ITALIC_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\*([^*]+)\*").unwrap());
    static CODE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"`([^`]+)`").unwrap());

    let s = IMG_RE.replace_all(text, |c: &regex::Captures| {
        format!("<img src=\"{}\" alt=\"{}\"/>", esc(&c[2]), esc(&c[1]))
    });
    let s = LINK_RE.replace_all(&s, |c: &regex::Captures| {
        format!("<a href=\"{}\">{}</a>", esc(&c[2]), esc(&c[1]))
    });
    let s = BOLD_RE.replace_all(&s, "<strong>$1</strong>");
    let s = ITALIC_RE.replace_all(&s, "<em>$1</em>");
    let s = CODE_RE.replace_all(&s, |c: &regex::Captures| format!("<code>{}</code>", esc(&c[1])));
    s.to_string()
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_md_to_html() {
        let html = markdown_to_html("# 标题\n\n段落 **加粗** 文本\n\n```rust\nfn a(){}\n```\n");
        assert!(html.contains("<h1>标题</h1>"));
        assert!(html.contains("<strong>加粗</strong>"));
        assert!(html.contains("language-rust"));
    }
}
