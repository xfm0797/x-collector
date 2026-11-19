use rusqlite::{params, Connection};

use crate::domain::dedup::title_similarity;

use super::now;

pub struct DedupRepo;

impl DedupRepo {
    pub fn url_exists(conn: &Connection, url_hash: &str) -> bool {
        conn.query_row("SELECT 1 FROM dedup_cache WHERE url_hash = ?1", params![url_hash], |_| Ok(()))
            .is_ok()
    }

    pub fn fingerprint_exists(conn: &Connection, fingerprint: &str) -> bool {
        conn.query_row(
            "SELECT 1 FROM dedup_cache WHERE fingerprint = ?1",
            params![fingerprint],
            |_| Ok(()),
        )
        .is_ok()
    }

    /// 返回与给定标题相似（超过阈值）的已有标题数量
    pub fn find_similar_title(conn: &Connection, title: &str, threshold: f64) -> Vec<String> {
        let Ok(mut stmt) = conn.prepare("SELECT title FROM articles") else {
            return vec![];
        };
        let Ok(titles) = stmt.query_map([], |r| r.get::<_, String>(0)) else {
            return vec![];
        };
        let mut similar = vec![];
        for t in titles.flatten() {
            let sim = title_similarity(&t, title);
            if sim >= threshold {
                similar.push(t);
            }
        }
        similar
    }

    pub fn insert(conn: &Connection, url_hash: &str, title_hash: &str, fingerprint: &str) -> rusqlite::Result<()> {
        conn.execute(
            "INSERT OR IGNORE INTO dedup_cache (url_hash, title_hash, fingerprint, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![url_hash, title_hash, fingerprint, now()],
        )?;
        Ok(())
    }
}
