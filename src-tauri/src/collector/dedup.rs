/// 文章去重：URL 哈希 + 标题相似度 + 内容指纹

/// 归一化标题：去除空白与标点、转小写
fn normalize_title(title: &str) -> Vec<char> {
    title
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

/// 标题相似度（0.0 - 1.0）：基于归一化编辑距离
pub fn title_similarity(a: &str, b: &str) -> f64 {
    let ca = normalize_title(a);
    let cb = normalize_title(b);
    if ca.is_empty() && cb.is_empty() {
        return 1.0;
    }
    if ca.is_empty() || cb.is_empty() {
        return 0.0;
    }
    let dist = levenshtein(&ca, &cb);
    let max_len = ca.len().max(cb.len()) as f64;
    1.0 - (dist as f64 / max_len)
}

/// 字符级编辑距离（动态规划）
fn levenshtein(a: &[char], b: &[char]) -> usize {
    let (m, n) = (a.len(), b.len());
    let mut prev: Vec<usize> = (0..=n).collect();
    let mut cur: Vec<usize> = vec![0; n + 1];
    for i in 1..=m {
        cur[0] = i;
        for j in 1..=n {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[n]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_similarity() {
        // 追加年份后缀仍有较高相似度
        assert!(title_similarity("Rust 入门指南", "Rust 入门指南（2024）") > 0.5);
        // 完全不同的标题相似度低
        assert!(title_similarity("Rust 教程", "Python 爬虫实战") < 0.3);
        assert_eq!(title_similarity("a", "a"), 1.0);
    }
}
