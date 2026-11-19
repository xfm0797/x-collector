use rusqlite::Connection;

/// 连接级 PRAGMA：WAL 提升读写并发，NORMAL 降低同步开销，缓存 16MB
const PRAGMAS: &str = "
PRAGMA journal_mode=WAL;
PRAGMA synchronous=NORMAL;
PRAGMA foreign_keys=ON;
PRAGMA busy_timeout=5000;
PRAGMA temp_store=MEMORY;
PRAGMA cache_size=-16000;
";

/// 在应用数据目录初始化 SQLite 数据库
pub fn init(data_dir: &std::path::Path) -> Result<Connection, Box<dyn std::error::Error>> {
    std::fs::create_dir_all(data_dir)?;
    let db_path = data_dir.join("x-collector.db");
    let conn = Connection::open(&db_path)?;
    conn.execute_batch(PRAGMAS)?;
    super::migrations::run(&conn)?;
    Ok(conn)
}
