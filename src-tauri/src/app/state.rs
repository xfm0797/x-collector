use std::sync::{Mutex, MutexGuard};

use rusqlite::Connection;

use super::error::to_err;

/// 全局应用状态：SQLite 连接
pub struct AppState {
    pub db: Mutex<Connection>,
}

impl AppState {
    pub fn new(conn: Connection) -> Self {
        Self { db: Mutex::new(conn) }
    }

    /// 获取数据库连接；锁中毒时返回可读错误
    pub fn conn(&self) -> Result<MutexGuard<'_, Connection>, String> {
        self.db.lock().map_err(|e| format!("数据库连接锁定失败：{}", to_err(e)))
    }
}
