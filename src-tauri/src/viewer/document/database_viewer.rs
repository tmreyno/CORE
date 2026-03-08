// =============================================================================
// CORE-FFX - Forensic File Explorer
// Database Viewer - SQLite database inspection for forensic analysis
// =============================================================================

use rusqlite::{types::Value as SqlValue, Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use std::path::Path;

use super::error::{DocumentError, DocumentResult};

// =============================================================================
// Types
// =============================================================================

/// Overview information about a SQLite database
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseInfo {
    pub path: String,
    pub file_size: u64,
    pub page_count: i64,
    pub page_size: i64,
    pub tables: Vec<TableSummary>,
    pub views: Vec<String>,
    pub journal_mode: String,
    pub sqlite_version: String,
}

/// Summary of a table (name + row count)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableSummary {
    pub name: String,
    pub row_count: i64,
    pub column_count: usize,
    pub is_system: bool,
}

/// Column definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnInfo {
    pub index: usize,
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub default_value: Option<String>,
    pub is_primary_key: bool,
}

/// Full table schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableSchema {
    pub name: String,
    pub columns: Vec<ColumnInfo>,
    pub row_count: i64,
    pub create_sql: Option<String>,
    pub indexes: Vec<String>,
}

/// Paginated row data from a table
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableRows {
    pub table_name: String,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub total_count: i64,
    pub page: usize,
    pub page_size: usize,
    pub has_more: bool,
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Open a SQLite database in read-only immutable mode (forensic safe)
///
/// Uses `immutable=1` URI parameter to prevent SQLite from looking for
/// WAL/journal files which may not exist on extracted forensic evidence.
fn open_readonly(path: impl AsRef<Path>) -> DocumentResult<Connection> {
    let path = path.as_ref();

    // Try immutable mode first (skips WAL/journal, ideal for forensic files)
    let uri = format!("file:{}?immutable=1", path.to_string_lossy());
    if let Ok(conn) = Connection::open_with_flags(
        &uri,
        OpenFlags::SQLITE_OPEN_READ_ONLY
            | OpenFlags::SQLITE_OPEN_NO_MUTEX
            | OpenFlags::SQLITE_OPEN_URI,
    ) {
        return Ok(conn);
    }

    // Fall back to standard read-only (for databases with accessible journals)
    Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| DocumentError::Parse(format!("Failed to open database: {}", e)))
}

/// Format a SQLite value to a display string
fn format_sql_value(val: &SqlValue) -> String {
    match val {
        SqlValue::Null => "(NULL)".to_string(),
        SqlValue::Integer(i) => i.to_string(),
        SqlValue::Real(f) => format!("{}", f),
        SqlValue::Text(s) => {
            if s.len() > 256 {
                // Use char_indices to find a safe UTF-8 boundary near 256 chars
                let truncated: String = s.chars().take(256).collect();
                format!("{}... ({} chars)", truncated, s.chars().count())
            } else {
                s.clone()
            }
        }
        SqlValue::Blob(b) => {
            let hex: String = b
                .iter()
                .take(32)
                .map(|byte| format!("{:02x}", byte))
                .collect::<Vec<_>>()
                .join(" ");
            if b.len() > 32 {
                format!("[BLOB] {} ... ({} bytes)", hex, b.len())
            } else {
                format!("[BLOB] {} ({} bytes)", hex, b.len())
            }
        }
    }
}

/// Sanitize a table name for use in SQL queries (prevent injection)
fn sanitize_table_name(name: &str) -> DocumentResult<String> {
    // Only allow alphanumeric, underscores, and dots
    if name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '.' || c == '-')
    {
        Ok(format!("\"{}\"", name))
    } else {
        Err(DocumentError::Parse(format!(
            "Invalid table name: {}",
            name
        )))
    }
}

// =============================================================================
// Public API
// =============================================================================

/// Get overview information about a SQLite database
pub fn get_database_info(path: impl AsRef<Path>) -> DocumentResult<DatabaseInfo> {
    let path = path.as_ref();
    let conn = open_readonly(path)?;

    // Get file size
    let file_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);

    // Get page count and size
    let page_count: i64 = conn
        .query_row("PRAGMA page_count", [], |r| r.get(0))
        .unwrap_or(0);
    let page_size: i64 = conn
        .query_row("PRAGMA page_size", [], |r| r.get(0))
        .unwrap_or(4096);

    // Get journal mode
    let journal_mode: String = conn
        .query_row("PRAGMA journal_mode", [], |r| r.get(0))
        .unwrap_or_else(|_| "unknown".to_string());

    // Get SQLite version
    let sqlite_version: String = conn
        .query_row("SELECT sqlite_version()", [], |r| r.get(0))
        .unwrap_or_else(|_| "unknown".to_string());

    // Get list of tables
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
        .map_err(|e| DocumentError::Parse(format!("Failed to list tables: {}", e)))?;

    let table_names: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| DocumentError::Parse(format!("Failed to read table names: {}", e)))?
        .filter_map(|r| r.ok())
        .collect();

    // Get table summaries with row counts and column counts
    let mut tables = Vec::new();
    for name in &table_names {
        let safe_name = sanitize_table_name(name)?;
        let row_count: i64 = conn
            .query_row(&format!("SELECT COUNT(*) FROM {}", safe_name), [], |r| {
                r.get(0)
            })
            .unwrap_or(0);

        // Get column count via PRAGMA
        let column_count = conn
            .prepare(&format!("PRAGMA table_info({})", safe_name))
            .map(|mut s| {
                s.query_map([], |_| Ok(()))
                    .map(|rows| rows.count())
                    .unwrap_or(0)
            })
            .unwrap_or(0);

        tables.push(TableSummary {
            is_system: name.starts_with("sqlite_"),
            name: name.clone(),
            row_count,
            column_count,
        });
    }

    // Get views
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='view' ORDER BY name")
        .map_err(|e| DocumentError::Parse(format!("Failed to list views: {}", e)))?;

    let views: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| DocumentError::Parse(format!("Failed to read view names: {}", e)))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(DatabaseInfo {
        path: path.to_string_lossy().to_string(),
        file_size,
        page_count,
        page_size,
        tables,
        views,
        journal_mode,
        sqlite_version,
    })
}

/// Get the schema of a specific table
pub fn get_table_schema(
    db_path: impl AsRef<Path>,
    table_name: &str,
) -> DocumentResult<TableSchema> {
    let db_path = db_path.as_ref();
    let conn = open_readonly(db_path)?;
    let safe_name = sanitize_table_name(table_name)?;

    // Get columns via PRAGMA table_info
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({})", safe_name))
        .map_err(|e| DocumentError::Parse(format!("Failed to get table info: {}", e)))?;

    let columns: Vec<ColumnInfo> = stmt
        .query_map([], |row| {
            Ok(ColumnInfo {
                index: row.get::<_, i32>(0)? as usize,
                name: row.get(1)?,
                data_type: row.get::<_, String>(2).unwrap_or_else(|_| "".to_string()),
                nullable: row.get::<_, i32>(3).unwrap_or(1) == 0, // notnull=0 means nullable
                default_value: row.get::<_, Option<String>>(4).unwrap_or(None),
                is_primary_key: row.get::<_, i32>(5).unwrap_or(0) != 0,
            })
        })
        .map_err(|e| DocumentError::Parse(format!("Failed to read column info: {}", e)))?
        .filter_map(|r| r.ok())
        .collect();

    // Get row count
    let row_count: i64 = conn
        .query_row(&format!("SELECT COUNT(*) FROM {}", safe_name), [], |r| {
            r.get(0)
        })
        .unwrap_or(0);

    // Get CREATE TABLE statement
    let create_sql: Option<String> = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name=?1",
            [table_name],
            |r| r.get(0),
        )
        .ok();

    // Get indexes
    let mut idx_stmt = conn
        .prepare(&format!("PRAGMA index_list({})", safe_name))
        .map_err(|e| DocumentError::Parse(format!("Failed to list indexes: {}", e)))?;

    let indexes: Vec<String> = idx_stmt
        .query_map([], |row| row.get(1))
        .map_err(|e| DocumentError::Parse(format!("Failed to read indexes: {}", e)))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(TableSchema {
        name: table_name.to_string(),
        columns,
        row_count,
        create_sql,
        indexes,
    })
}

/// Query rows from a table with pagination
pub fn query_table_rows(
    db_path: impl AsRef<Path>,
    table_name: &str,
    page: usize,
    page_size: usize,
) -> DocumentResult<TableRows> {
    let db_path = db_path.as_ref();
    let conn = open_readonly(db_path)?;
    let safe_name = sanitize_table_name(table_name)?;

    // Clamp page_size
    let page_size = page_size.clamp(1, 500);
    let offset = page * page_size;

    // Get total count
    let total_count: i64 = conn
        .query_row(&format!("SELECT COUNT(*) FROM {}", safe_name), [], |r| {
            r.get(0)
        })
        .unwrap_or(0);

    // Get column names
    let stmt = conn
        .prepare(&format!("SELECT * FROM {} LIMIT 0", safe_name))
        .map_err(|e| DocumentError::Parse(format!("Failed to prepare query: {}", e)))?;

    let columns: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();
    drop(stmt);

    // Query rows with pagination
    let sql = format!("SELECT * FROM {} LIMIT ?1 OFFSET ?2", safe_name);
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| DocumentError::Parse(format!("Failed to prepare paginated query: {}", e)))?;

    let col_count = columns.len();
    let mut rows: Vec<Vec<String>> = Vec::new();

    let row_iter = stmt
        .query_map(rusqlite::params![page_size as i64, offset as i64], |row| {
            let mut row_data = Vec::with_capacity(col_count);
            for i in 0..col_count {
                let val: SqlValue = row.get(i).unwrap_or(SqlValue::Null);
                row_data.push(format_sql_value(&val));
            }
            Ok(row_data)
        })
        .map_err(|e| DocumentError::Parse(format!("Failed to query rows: {}", e)))?;

    for row_data in row_iter.flatten() {
        rows.push(row_data);
    }

    Ok(TableRows {
        table_name: table_name.to_string(),
        columns,
        rows,
        total_count,
        page,
        page_size,
        has_more: (offset + page_size) < total_count as usize,
    })
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a temp SQLite DB for testing
    fn create_test_db() -> (tempfile::NamedTempFile, String) {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_string_lossy().to_string();

        let conn = Connection::open(tmp.path()).unwrap();
        conn.execute_batch(
            "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, email TEXT, age INTEGER);
             INSERT INTO users VALUES (1, 'Alice', 'alice@test.com', 30);
             INSERT INTO users VALUES (2, 'Bob', 'bob@test.com', 25);
             INSERT INTO users VALUES (3, 'Charlie', 'charlie@test.com', 35);
             CREATE TABLE logs (id INTEGER PRIMARY KEY, message TEXT, level TEXT);
             INSERT INTO logs VALUES (1, 'System started', 'info');
             CREATE VIEW active_users AS SELECT * FROM users WHERE age > 20;
             CREATE INDEX idx_users_name ON users(name);",
        )
        .unwrap();

        (tmp, path)
    }

    #[test]
    fn test_get_database_info() {
        let (_tmp, path) = create_test_db();
        let info = get_database_info(&path).unwrap();

        assert_eq!(info.tables.len(), 2);
        assert!(info.views.contains(&"active_users".to_string()));
        assert!(info.page_count > 0);
        assert!(info.page_size > 0);
        assert!(!info.sqlite_version.is_empty());

        // Find users table
        let users = info.tables.iter().find(|t| t.name == "users").unwrap();
        assert_eq!(users.row_count, 3);
        assert_eq!(users.column_count, 4);
    }

    #[test]
    fn test_get_table_schema() {
        let (_tmp, path) = create_test_db();
        let schema = get_table_schema(&path, "users").unwrap();

        assert_eq!(schema.name, "users");
        assert_eq!(schema.columns.len(), 4);
        assert_eq!(schema.row_count, 3);
        assert!(schema.create_sql.is_some());

        // Check column info
        let id_col = schema.columns.iter().find(|c| c.name == "id").unwrap();
        assert!(id_col.is_primary_key);
        assert_eq!(id_col.data_type, "INTEGER");

        let name_col = schema.columns.iter().find(|c| c.name == "name").unwrap();
        assert_eq!(name_col.data_type, "TEXT");

        // Check indexes
        assert!(schema.indexes.contains(&"idx_users_name".to_string()));
    }

    #[test]
    fn test_query_table_rows() {
        let (_tmp, path) = create_test_db();
        let rows = query_table_rows(&path, "users", 0, 10).unwrap();

        assert_eq!(rows.table_name, "users");
        assert_eq!(rows.columns.len(), 4);
        assert_eq!(rows.rows.len(), 3);
        assert_eq!(rows.total_count, 3);
        assert_eq!(rows.page, 0);
        assert!(!rows.has_more);

        // Check first row
        assert_eq!(rows.rows[0][0], "1"); // id
        assert_eq!(rows.rows[0][1], "Alice"); // name
    }

    #[test]
    fn test_query_table_rows_pagination() {
        let (_tmp, path) = create_test_db();

        // Page 0 with size 2
        let page0 = query_table_rows(&path, "users", 0, 2).unwrap();
        assert_eq!(page0.rows.len(), 2);
        assert!(page0.has_more);
        assert_eq!(page0.rows[0][1], "Alice");
        assert_eq!(page0.rows[1][1], "Bob");

        // Page 1 with size 2
        let page1 = query_table_rows(&path, "users", 1, 2).unwrap();
        assert_eq!(page1.rows.len(), 1);
        assert!(!page1.has_more);
        assert_eq!(page1.rows[0][1], "Charlie");
    }

    #[test]
    fn test_format_sql_value_types() {
        assert_eq!(format_sql_value(&SqlValue::Null), "(NULL)");
        assert_eq!(format_sql_value(&SqlValue::Integer(42)), "42");
        assert_eq!(format_sql_value(&SqlValue::Real(1.23)), "1.23");
        assert_eq!(
            format_sql_value(&SqlValue::Text("hello".to_string())),
            "hello"
        );

        let blob = SqlValue::Blob(vec![0xDE, 0xAD, 0xBE, 0xEF]);
        let formatted = format_sql_value(&blob);
        assert!(formatted.contains("[BLOB]"));
        assert!(formatted.contains("de ad be ef"));
    }

    #[test]
    fn test_format_sql_value_long_text() {
        let long_text = "a".repeat(500);
        let formatted = format_sql_value(&SqlValue::Text(long_text));
        assert!(formatted.contains("..."));
        assert!(formatted.contains("500 chars"));
    }

    #[test]
    fn test_sanitize_table_name_valid() {
        assert!(sanitize_table_name("users").is_ok());
        assert!(sanitize_table_name("my_table").is_ok());
        assert!(sanitize_table_name("table123").is_ok());
        assert!(sanitize_table_name("schema.table").is_ok());
    }

    #[test]
    fn test_sanitize_table_name_invalid() {
        assert!(sanitize_table_name("users; DROP TABLE").is_err());
        assert!(sanitize_table_name("table'name").is_err());
        assert!(sanitize_table_name("table\"name").is_err());
    }

    #[test]
    fn test_get_database_info_nonexistent() {
        let result = get_database_info("/nonexistent/file.db");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_table_schema_nonexistent_table() {
        let (_tmp, path) = create_test_db();
        let result = get_table_schema(&path, "nonexistent_table");
        // Should succeed but with empty columns (PRAGMA table_info returns nothing)
        assert!(result.is_ok());
        let schema = result.unwrap();
        assert_eq!(schema.columns.len(), 0);
    }

    #[test]
    fn test_empty_database() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let conn = Connection::open(tmp.path()).unwrap();
        conn.execute_batch("SELECT 1").unwrap(); // Initialize DB
        drop(conn);

        let info = get_database_info(tmp.path()).unwrap();
        assert_eq!(info.tables.len(), 0);
        assert_eq!(info.views.len(), 0);
    }

    #[test]
    fn test_database_with_blob_data() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let conn = Connection::open(tmp.path()).unwrap();
        conn.execute_batch(
            "CREATE TABLE blobs (id INTEGER PRIMARY KEY, data BLOB);
             INSERT INTO blobs VALUES (1, X'DEADBEEF');",
        )
        .unwrap();
        drop(conn);

        let rows = query_table_rows(tmp.path(), "blobs", 0, 10).unwrap();
        assert_eq!(rows.rows.len(), 1);
        assert!(rows.rows[0][1].contains("[BLOB]"));
    }

    #[test]
    fn test_database_with_null_values() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let conn = Connection::open(tmp.path()).unwrap();
        conn.execute_batch(
            "CREATE TABLE nullable (id INTEGER, value TEXT);
             INSERT INTO nullable VALUES (1, NULL);
             INSERT INTO nullable VALUES (NULL, 'test');",
        )
        .unwrap();
        drop(conn);

        let rows = query_table_rows(tmp.path(), "nullable", 0, 10).unwrap();
        assert_eq!(rows.rows.len(), 2);
        assert_eq!(rows.rows[0][1], "(NULL)");
        assert_eq!(rows.rows[1][0], "(NULL)");
    }
}
