use rusqlite::Connection;

/// 在应用数据目录初始化 SQLite 数据库
pub fn init(data_dir: &std::path::Path) -> Result<Connection, Box<dyn std::error::Error>> {
    std::fs::create_dir_all(data_dir)?;
    let db_path = data_dir.join("x-collector.db");
    let conn = Connection::open(&db_path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    super::migrations::run(&conn)?;
    Ok(conn)
}
