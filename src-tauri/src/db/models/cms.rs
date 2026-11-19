use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};

use super::now;

#[derive(Debug, Serialize, Clone)]
pub struct CmsConnection {
    pub id: i64,
    pub name: String,
    pub cms_type: String,
    pub site_url: String,
    pub api_path: Option<String>,
    pub username: String,
    pub default_category: Option<String>,
    pub default_status: String,
    pub tag_mapping: Option<String>,
    pub enabled: i64,
    pub has_password: bool,
    pub last_used: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct CmsConnectionInput {
    pub id: Option<i64>,
    pub name: String,
    pub cms_type: Option<String>,
    pub site_url: String,
    pub api_path: Option<String>,
    pub username: String,
    pub password: Option<String>,
    pub default_category: Option<String>,
    pub default_status: Option<String>,
    pub tag_mapping: Option<serde_json::Value>,
    pub enabled: Option<i64>,
}

impl CmsConnection {
    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        let password: String = row.get("password")?;
        Ok(CmsConnection {
            id: row.get("id")?,
            name: row.get("name")?,
            cms_type: row.get("cms_type")?,
            site_url: row.get("site_url")?,
            api_path: row.get("api_path")?,
            username: row.get("username")?,
            default_category: row.get("default_category")?,
            default_status: row.get("default_status")?,
            tag_mapping: row.get("tag_mapping")?,
            enabled: row.get("enabled")?,
            has_password: !password.is_empty(),
            last_used: row.get("last_used")?,
            created_at: row.get("created_at")?,
        })
    }
}

pub struct CmsRepo;

impl CmsRepo {
    pub fn list(conn: &Connection) -> rusqlite::Result<Vec<CmsConnection>> {
        let mut stmt = conn.prepare("SELECT * FROM cms_connections ORDER BY created_at DESC")?;
        let rows = stmt.query_map([], CmsConnection::from_row)?;
        let connections = rows.collect::<Result<Vec<_>, _>>()?;
        Ok(connections)
    }

    pub fn get(conn: &Connection, id: i64) -> rusqlite::Result<CmsConnection> {
        conn.query_row("SELECT * FROM cms_connections WHERE id = ?1", params![id], |r| {
            CmsConnection::from_row(r)
        })
    }

    /// 读取（解密后的）密码，供发布模块使用
    pub fn get_password(conn: &Connection, id: i64) -> rusqlite::Result<String> {
        let encrypted: String =
            conn.query_row("SELECT password FROM cms_connections WHERE id = ?1", params![id], |r| r.get(0))?;
        Ok(crate::utils::hash::decrypt_password(&encrypted))
    }

    pub fn save(conn: &Connection, input: &CmsConnectionInput) -> rusqlite::Result<CmsConnection> {
        let tag_mapping = input
            .tag_mapping
            .as_ref()
            .map(|v| serde_json::to_string(v).unwrap_or_default());
        let cms_type = input.cms_type.as_deref().unwrap_or("wordpress");
        let default_api = match cms_type {
            "wordpress" => "/xmlrpc.php",
            "typecho" => "/action/xmlrpc",
            "zblog" => "/zb_system/xml-rpc/index.php",
            _ => "",
        };

        if let Some(id) = input.id {
            conn.execute(
                "UPDATE cms_connections SET name = ?1, cms_type = ?2, site_url = ?3,
                 api_path = COALESCE(?4, api_path), username = ?5,
                 password = COALESCE(?6, password), default_category = COALESCE(?7, default_category),
                 default_status = COALESCE(?8, default_status), tag_mapping = COALESCE(?9, tag_mapping),
                 enabled = COALESCE(?10, enabled) WHERE id = ?11",
                params![
                    input.name,
                    cms_type,
                    input.site_url,
                    input.api_path.as_deref().filter(|s| !s.is_empty()).or(Some(default_api)),
                    input.username,
                    input.password.as_deref().filter(|s| !s.is_empty()).map(crate::utils::hash::encrypt_password),
                    input.default_category,
                    input.default_status.as_deref().unwrap_or("publish"),
                    tag_mapping,
                    input.enabled.unwrap_or(1),
                    id,
                ],
            )?;
            Self::get(conn, id)
        } else {
            conn.execute(
                "INSERT INTO cms_connections (name, cms_type, site_url, api_path, username, password, default_category, default_status, tag_mapping, enabled, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    input.name,
                    cms_type,
                    input.site_url,
                    input.api_path.as_deref().filter(|s| !s.is_empty()).or(Some(default_api)),
                    input.username,
                    input.password.as_deref().map(crate::utils::hash::encrypt_password).unwrap_or_default(),
                    input.default_category,
                    input.default_status.as_deref().unwrap_or("publish"),
                    tag_mapping,
                    input.enabled.unwrap_or(1),
                    now(),
                ],
            )?;
            Self::get(conn, conn.last_insert_rowid())
        }
    }

    pub fn delete(conn: &Connection, ids: &[i64]) -> rusqlite::Result<()> {
        for id in ids {
            conn.execute("DELETE FROM cms_connections WHERE id = ?1", params![id])?;
        }
        Ok(())
    }

    pub fn set_last_used(conn: &Connection, id: i64) -> rusqlite::Result<()> {
        conn.execute(
            "UPDATE cms_connections SET last_used = ?1 WHERE id = ?2",
            params![now(), id],
        )?;
        Ok(())
    }
}
