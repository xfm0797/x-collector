use rand::prelude::*;

/// 句子改写引擎：连接词插入、语序调整、长句拆分

const CONNECTIVES: &[&str] = &["然而", "因此", "值得注意的是", "此外", "与此同时", "换句话说", "具体而言", "总的来说"];

/// 按句号/问号/叹号切分句子（保留结尾标点）
pub fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        current.push(ch);
        if matches!(ch, '。' | '！' | '？' | ';' | '；') {
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() {
                sentences.push(trimmed);
            }
            current.clear();
        }
    }
    let rest = current.trim().to_string();
    if !rest.is_empty() {
        sentences.push(rest);
    }
    sentences
}

/// 句子改写：按比例对句子做变换
pub fn rewrite_sentences(text: &str, ratio: f64, seed: u64) -> String {
    let mut rng = StdRng::seed_from_u64(seed);
    let sentences = split_sentences(text);
    let total = sentences.len();
    if total == 0 {
        return text.to_string();
    }

    let rewrite_count = ((total as f64) * ratio).round() as usize;
    let mut indices: Vec<usize> = (0..total).collect();
    indices.shuffle(&mut rng);
    let rewrite_set: std::collections::HashSet<usize> =
        indices.into_iter().take(rewrite_count).collect();

    sentences
        .into_iter()
        .enumerate()
        .map(|(i, s)| {
            if rewrite_set.contains(&i) {
                rewrite_one(&mut rng, &s)
            } else {
                s
            }
        })
        .collect::<Vec<_>>()
        .join("")
}

/// 单句变换：连接词插入 / 逗号子句换序 / 长句拆分
fn rewrite_one(rng: &mut StdRng, sentence: &str) -> String {
    let strategy = rng.gen_range(0..3);
    match strategy {
        // 1. 句首插入连接词
        0 => {
            let conn = CONNECTIVES[rng.gen_range(0..CONNECTIVES.len())];
            if CONNECTIVES.iter().any(|c| sentence.starts_with(c)) || sentence.chars().count() < 12 {
                sentence.to_string()
            } else {
                format!("{}，{}", conn, sentence)
            }
        }
        // 2. 逗号分隔的子句换序（首子句与其他子句交换，仅适用于 2-4 个子句）
        1 => {
            let parts: Vec<&str> = sentence.split('，').collect();
            if parts.len() >= 2 && parts.len() <= 4 && parts.iter().all(|p| p.chars().count() >= 4) {
                let mut shuffled = parts.clone();
                shuffled.shuffle(rng);
                shuffled.join("，")
            } else {
                sentence.to_string()
            }
        }
        // 3. 超长句拆分：把中间某个逗号改为句号
        _ => {
            let char_count = sentence.chars().count();
            if char_count > 60 {
                let commas: Vec<usize> = sentence
                    .match_indices('，')
                    .map(|(i, _)| i)
                    .collect();
                if commas.len() >= 3 {
                    // 挑中间位置的逗号改为句号
                    let mid = commas[commas.len() / 2];
                    let mut s = sentence.to_string();
                    s.replace_range(mid..mid + '，'.len_utf8(), "。");
                    s
                } else {
                    sentence.to_string()
                }
            } else {
                sentence.to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split() {
        let s = split_sentences("这是第一句。这是第二句！没有结尾");
        assert_eq!(s.len(), 3);
    }
}
