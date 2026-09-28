/// 生成文件名安全的 slug：保留中英文与数字
pub fn slugify(text: &str) -> String {
    let mut result = String::new();
    let mut last_dash = false;
    for c in text.chars() {
        if c.is_ascii_alphanumeric() || ('\u{4e00}'..='\u{9fff}').contains(&c) {
            result.push(c.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash && !result.is_empty() {
            result.push('-');
            last_dash = true;
        }
    }
    let trimmed = result.trim_matches('-').to_string();
    let limited: String = trimmed.chars().take(80).collect();
    if limited.is_empty() {
        "untitled".to_string()
    } else {
        limited
    }
}

/// Hexo 文件名：YYYYMMDD-title.md
pub fn hexo_filename(title: &str, date: &str) -> String {
    let d = if date.len() >= 10 {
        date[..10].replace('-', "")
    } else {
        chrono::Local::now().format("%Y%m%d").to_string()
    };
    format!("{}-{}.md", d, slugify(title))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World! 测试"), "hello-world-测试");
        assert_eq!(slugify("///"), "untitled");
    }
}
