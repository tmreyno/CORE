// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! SQLite database module for FFX persistent storage
//! 
//! Handles:
//! - Sessions (open directories/workspaces)
//! - Files (discovered evidence containers)
//! - Hashes (computed hash records with timestamps)
//! - Verifications (verification audit trail)
//! - UI state (open tabs, settings)

use rusqlite::{Connection, params, Result as SqlResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use parking_lot::Mutex;

/// Database connection wrapper for thread-safe access
pub struct Database {
    conn: Mutex<Connection>,
}

// ============================================================================
// Data Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub name: String,
    pub root_path: String,
    pub created_at: String,
    pub last_opened_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRecord {
    pub id: String,
    pub session_id: String,
    pub path: String,
    pub filename: String,
    pub container_type: String,
    pub total_size: i64,
    pub segment_count: i32,
    pub discovered_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashRecord {
    pub id: String,
    pub file_id: String,
    pub algorithm: String,
    pub hash_value: String,
    pub computed_at: String,
    pub segment_index: Option<i32>,  // NULL for full container hash
    pub segment_name: Option<String>,
    pub source: String,  // 'computed', 'stored', 'imported'
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationRecord {
    pub id: String,
    pub hash_id: String,
    pub verified_at: String,
    pub result: String,  // 'match', 'mismatch'
    pub expected_hash: String,
    pub actual_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenTabRecord {
    pub id: String,
    pub session_id: String,
    pub file_path: String,
    pub tab_order: i32,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub key: String,
    pub value: String,
}

// ============================================================================
// Database Implementation
// ============================================================================

impl Database {
    /// Initialize database at the given path, creating tables if needed
    pub fn new(db_path: &PathBuf) -> SqlResult<Self> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        
        let conn = Connection::open(db_path)?;
        let db = Database {
            conn: Mutex::new(conn),
        };
        db.init_schema()?;
        Ok(db)
    }
    
    /// Create all tables if they don't exist
    fn init_schema(&self) -> SqlResult<()> {
        let conn = self.conn.lock();
        
        conn.execute_batch(r#"
            -- Sessions (open directories/workspaces)
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                root_path TEXT NOT NULL UNIQUE,
                created_at TEXT NOT NULL,
                last_opened_at TEXT NOT NULL
            );
            
            -- Files (discovered evidence containers)
            CREATE TABLE IF NOT EXISTS files (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                path TEXT NOT NULL,
                filename TEXT NOT NULL,
                container_type TEXT NOT NULL,
                total_size INTEGER NOT NULL,
                segment_count INTEGER NOT NULL DEFAULT 1,
                discovered_at TEXT NOT NULL,
                FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE,
                UNIQUE(session_id, path)
            );
            
            -- Hashes (computed hash records - immutable audit trail)
            CREATE TABLE IF NOT EXISTS hashes (
                id TEXT PRIMARY KEY,
                file_id TEXT NOT NULL,
                algorithm TEXT NOT NULL,
                hash_value TEXT NOT NULL,
                computed_at TEXT NOT NULL,
                segment_index INTEGER,
                segment_name TEXT,
                source TEXT NOT NULL DEFAULT 'computed',
                FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
            );
            
            -- Verifications (verification audit trail)
            CREATE TABLE IF NOT EXISTS verifications (
                id TEXT PRIMARY KEY,
                hash_id TEXT NOT NULL,
                verified_at TEXT NOT NULL,
                result TEXT NOT NULL,
                expected_hash TEXT NOT NULL,
                actual_hash TEXT NOT NULL,
                FOREIGN KEY (hash_id) REFERENCES hashes(id) ON DELETE CASCADE
            );
            
            -- Open tabs (UI state per session)
            CREATE TABLE IF NOT EXISTS open_tabs (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                file_path TEXT NOT NULL,
                tab_order INTEGER NOT NULL,
                is_active INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE,
                UNIQUE(session_id, file_path)
            );
            
            -- App settings (key-value store)
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            
            -- Indexes for common queries
            CREATE INDEX IF NOT EXISTS idx_files_session ON files(session_id);
            CREATE INDEX IF NOT EXISTS idx_files_path ON files(path);
            CREATE INDEX IF NOT EXISTS idx_hashes_file ON hashes(file_id);
            CREATE INDEX IF NOT EXISTS idx_hashes_algorithm ON hashes(algorithm);
            CREATE INDEX IF NOT EXISTS idx_verifications_hash ON verifications(hash_id);
            CREATE INDEX IF NOT EXISTS idx_tabs_session ON open_tabs(session_id);
        "#)?;
        
        Ok(())
    }
    
    // ========================================================================
    // Session Operations
    // ========================================================================
    
    pub fn create_session(&self, id: &str, name: &str, root_path: &str) -> SqlResult<Session> {
        let conn = self.conn.lock();
        let now = chrono::Utc::now().to_rfc3339();
        
        conn.execute(
            "INSERT INTO sessions (id, name, root_path, created_at, last_opened_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, name, root_path, now, now],
        )?;
        
        Ok(Session {
            id: id.to_string(),
            name: name.to_string(),
            root_path: root_path.to_string(),
            created_at: now.clone(),
            last_opened_at: now,
        })
    }
    
    pub fn get_session_by_path(&self, root_path: &str) -> SqlResult<Option<Session>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, name, root_path, created_at, last_opened_at FROM sessions WHERE root_path = ?1"
        )?;
        
        let mut rows = stmt.query(params![root_path])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Session {
                id: row.get(0)?,
                name: row.get(1)?,
                root_path: row.get(2)?,
                created_at: row.get(3)?,
                last_opened_at: row.get(4)?,
            }))
        } else {
            Ok(None)
        }
    }
    
    pub fn get_or_create_session(&self, root_path: &str) -> SqlResult<Session> {
        if let Some(session) = self.get_session_by_path(root_path)? {
            // Update last opened
            self.update_session_last_opened(&session.id)?;
            return Ok(session);
        }
        
        // Create new session
        let id = uuid::Uuid::new_v4().to_string();
        let name = std::path::Path::new(root_path)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".to_string());
        
        self.create_session(&id, &name, root_path)
    }
    
    pub fn update_session_last_opened(&self, session_id: &str) -> SqlResult<()> {
        let conn = self.conn.lock();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE sessions SET last_opened_at = ?1 WHERE id = ?2",
            params![now, session_id],
        )?;
        Ok(())
    }
    
    pub fn get_recent_sessions(&self, limit: i32) -> SqlResult<Vec<Session>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, name, root_path, created_at, last_opened_at 
             FROM sessions 
             ORDER BY last_opened_at DESC 
             LIMIT ?1"
        )?;
        
        let rows = stmt.query_map(params![limit], |row| {
            Ok(Session {
                id: row.get(0)?,
                name: row.get(1)?,
                root_path: row.get(2)?,
                created_at: row.get(3)?,
                last_opened_at: row.get(4)?,
            })
        })?;
        
        rows.collect()
    }
    
    pub fn get_last_session(&self) -> SqlResult<Option<Session>> {
        let sessions = self.get_recent_sessions(1)?;
        Ok(sessions.into_iter().next())
    }
    
    // ========================================================================
    // File Operations
    // ========================================================================
    
    pub fn upsert_file(&self, file: &FileRecord) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO files (id, session_id, path, filename, container_type, total_size, segment_count, discovered_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(session_id, path) DO UPDATE SET
                filename = excluded.filename,
                container_type = excluded.container_type,
                total_size = excluded.total_size,
                segment_count = excluded.segment_count",
            params![
                file.id, file.session_id, file.path, file.filename,
                file.container_type, file.total_size, file.segment_count, file.discovered_at
            ],
        )?;
        Ok(())
    }
    
    pub fn get_files_for_session(&self, session_id: &str) -> SqlResult<Vec<FileRecord>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, session_id, path, filename, container_type, total_size, segment_count, discovered_at
             FROM files WHERE session_id = ?1 ORDER BY filename"
        )?;
        
        let rows = stmt.query_map(params![session_id], |row| {
            Ok(FileRecord {
                id: row.get(0)?,
                session_id: row.get(1)?,
                path: row.get(2)?,
                filename: row.get(3)?,
                container_type: row.get(4)?,
                total_size: row.get(5)?,
                segment_count: row.get(6)?,
                discovered_at: row.get(7)?,
            })
        })?;
        
        rows.collect()
    }
    
    pub fn get_file_by_path(&self, session_id: &str, path: &str) -> SqlResult<Option<FileRecord>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, session_id, path, filename, container_type, total_size, segment_count, discovered_at
             FROM files WHERE session_id = ?1 AND path = ?2"
        )?;
        
        let mut rows = stmt.query(params![session_id, path])?;
        if let Some(row) = rows.next()? {
            Ok(Some(FileRecord {
                id: row.get(0)?,
                session_id: row.get(1)?,
                path: row.get(2)?,
                filename: row.get(3)?,
                container_type: row.get(4)?,
                total_size: row.get(5)?,
                segment_count: row.get(6)?,
                discovered_at: row.get(7)?,
            }))
        } else {
            Ok(None)
        }
    }
    
    // ========================================================================
    // Hash Operations
    // ========================================================================
    
    pub fn insert_hash(&self, hash: &HashRecord) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO hashes (id, file_id, algorithm, hash_value, computed_at, segment_index, segment_name, source)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                hash.id, hash.file_id, hash.algorithm, hash.hash_value,
                hash.computed_at, hash.segment_index, hash.segment_name, hash.source
            ],
        )?;
        Ok(())
    }
    
    pub fn get_hashes_for_file(&self, file_id: &str) -> SqlResult<Vec<HashRecord>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, file_id, algorithm, hash_value, computed_at, segment_index, segment_name, source
             FROM hashes WHERE file_id = ?1 ORDER BY computed_at DESC"
        )?;
        
        let rows = stmt.query_map(params![file_id], |row| {
            Ok(HashRecord {
                id: row.get(0)?,
                file_id: row.get(1)?,
                algorithm: row.get(2)?,
                hash_value: row.get(3)?,
                computed_at: row.get(4)?,
                segment_index: row.get(5)?,
                segment_name: row.get(6)?,
                source: row.get(7)?,
            })
        })?;
        
        rows.collect()
    }
    
    pub fn get_latest_hash(&self, file_id: &str, algorithm: &str, segment_index: Option<i32>) -> SqlResult<Option<HashRecord>> {
        let conn = self.conn.lock();
        
        let sql = if segment_index.is_some() {
            "SELECT id, file_id, algorithm, hash_value, computed_at, segment_index, segment_name, source
             FROM hashes WHERE file_id = ?1 AND algorithm = ?2 AND segment_index = ?3
             ORDER BY computed_at DESC LIMIT 1"
        } else {
            "SELECT id, file_id, algorithm, hash_value, computed_at, segment_index, segment_name, source
             FROM hashes WHERE file_id = ?1 AND algorithm = ?2 AND segment_index IS NULL
             ORDER BY computed_at DESC LIMIT 1"
        };
        
        let mut stmt = conn.prepare(sql)?;
        
        let mut rows = if let Some(idx) = segment_index {
            stmt.query(params![file_id, algorithm, idx])?
        } else {
            stmt.query(params![file_id, algorithm])?
        };
        
        if let Some(row) = rows.next()? {
            Ok(Some(HashRecord {
                id: row.get(0)?,
                file_id: row.get(1)?,
                algorithm: row.get(2)?,
                hash_value: row.get(3)?,
                computed_at: row.get(4)?,
                segment_index: row.get(5)?,
                segment_name: row.get(6)?,
                source: row.get(7)?,
            }))
        } else {
            Ok(None)
        }
    }
    
    /// Look up the latest known SHA-256 hash for a file by its source path.
    /// Joins files → hashes to find the most recent hash across all sessions.
    /// Returns (hash_value, source) or None if no hash is stored.
    pub fn lookup_known_hash_by_path(&self, path: &str) -> SqlResult<Option<(String, String)>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT h.hash_value, h.source
             FROM hashes h
             INNER JOIN files f ON h.file_id = f.id
             WHERE f.path = ?1 AND h.algorithm = 'SHA-256' AND h.segment_index IS NULL
             ORDER BY h.computed_at DESC
             LIMIT 1"
        )?;
        
        let mut rows = stmt.query(params![path])?;
        
        if let Some(row) = rows.next()? {
            Ok(Some((row.get(0)?, row.get(1)?)))
        } else {
            Ok(None)
        }
    }
    
    // ========================================================================
    // Verification Operations
    // ========================================================================
    
    pub fn insert_verification(&self, verification: &VerificationRecord) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO verifications (id, hash_id, verified_at, result, expected_hash, actual_hash)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                verification.id, verification.hash_id, verification.verified_at,
                verification.result, verification.expected_hash, verification.actual_hash
            ],
        )?;
        Ok(())
    }
    
    pub fn get_verifications_for_file(&self, file_id: &str) -> SqlResult<Vec<VerificationRecord>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT v.id, v.hash_id, v.verified_at, v.result, v.expected_hash, v.actual_hash
             FROM verifications v
             JOIN hashes h ON v.hash_id = h.id
             WHERE h.file_id = ?1
             ORDER BY v.verified_at DESC"
        )?;
        
        let rows = stmt.query_map(params![file_id], |row| {
            Ok(VerificationRecord {
                id: row.get(0)?,
                hash_id: row.get(1)?,
                verified_at: row.get(2)?,
                result: row.get(3)?,
                expected_hash: row.get(4)?,
                actual_hash: row.get(5)?,
            })
        })?;
        
        rows.collect()
    }
    
    // ========================================================================
    // Open Tabs Operations
    // ========================================================================
    
    pub fn save_open_tabs(&self, session_id: &str, tabs: &[OpenTabRecord]) -> SqlResult<()> {
        let conn = self.conn.lock();
        
        // Clear existing tabs for session
        conn.execute("DELETE FROM open_tabs WHERE session_id = ?1", params![session_id])?;
        
        // Insert new tabs
        for tab in tabs {
            conn.execute(
                "INSERT INTO open_tabs (id, session_id, file_path, tab_order, is_active)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![tab.id, session_id, tab.file_path, tab.tab_order, tab.is_active as i32],
            )?;
        }
        
        Ok(())
    }
    
    pub fn get_open_tabs(&self, session_id: &str) -> SqlResult<Vec<OpenTabRecord>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, session_id, file_path, tab_order, is_active
             FROM open_tabs WHERE session_id = ?1 ORDER BY tab_order"
        )?;
        
        let rows = stmt.query_map(params![session_id], |row| {
            let is_active: i32 = row.get(4)?;
            Ok(OpenTabRecord {
                id: row.get(0)?,
                session_id: row.get(1)?,
                file_path: row.get(2)?,
                tab_order: row.get(3)?,
                is_active: is_active != 0,
            })
        })?;
        
        rows.collect()
    }
    
    // ========================================================================
    // Settings Operations
    // ========================================================================
    
    pub fn set_setting(&self, key: &str, value: &str) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }
    
    pub fn get_setting(&self, key: &str) -> SqlResult<Option<String>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
        
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }
}

// ============================================================================
// Global Database Instance
// ============================================================================

use std::sync::OnceLock;

static DB: OnceLock<Database> = OnceLock::new();

/// Get the global database instance, initializing if needed
pub fn get_db() -> &'static Database {
    DB.get_or_init(|| {
        let app_data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("com.ffxcheck.app");
        let db_path = app_data_dir.join("ffx.db");
        
        tracing::info!("Initializing database at: {:?}", db_path);
        
        Database::new(&db_path).expect("Failed to initialize database")
    })
}

/// Initialize database with a custom path (for testing or app-provided path)
pub fn init_db(db_path: PathBuf) -> SqlResult<()> {
    let db = Database::new(&db_path)?;
    let _ = DB.set(db);
    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Create a test database in a temporary directory
    fn create_test_db() -> (Database, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Database::new(&db_path).expect("Failed to create test database");
        (db, temp_dir)
    }

    #[test]
    fn test_database_initialization() {
        let (db, _temp) = create_test_db();
        // Database should be usable
        let sessions = db.get_recent_sessions(10).unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn test_session_struct_fields() {
        let session = Session {
            id: "session-123".to_string(),
            name: "Test Session".to_string(),
            root_path: "/path/to/case".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            last_opened_at: "2024-01-02T00:00:00Z".to_string(),
        };
        assert_eq!(session.id, "session-123");
        assert_eq!(session.name, "Test Session");
        assert_eq!(session.root_path, "/path/to/case");
    }

    #[test]
    fn test_file_record_struct_fields() {
        let file = FileRecord {
            id: "file-123".to_string(),
            session_id: "session-123".to_string(),
            path: "/evidence/disk.E01".to_string(),
            filename: "disk.E01".to_string(),
            container_type: "E01".to_string(),
            total_size: 1024 * 1024 * 1024, // 1 GB
            segment_count: 5,
            discovered_at: "2024-01-01T00:00:00Z".to_string(),
        };
        assert_eq!(file.container_type, "E01");
        assert_eq!(file.segment_count, 5);
        assert_eq!(file.total_size, 1073741824);
    }

    #[test]
    fn test_hash_record_struct_fields() {
        let hash = HashRecord {
            id: "hash-123".to_string(),
            file_id: "file-123".to_string(),
            algorithm: "SHA-256".to_string(),
            hash_value: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(),
            computed_at: "2024-01-01T00:00:00Z".to_string(),
            segment_index: None,
            segment_name: None,
            source: "computed".to_string(),
        };
        assert_eq!(hash.algorithm, "SHA-256");
        assert_eq!(hash.source, "computed");
        assert!(hash.segment_index.is_none());
    }

    #[test]
    fn test_hash_record_with_segment() {
        let hash = HashRecord {
            id: "hash-456".to_string(),
            file_id: "file-123".to_string(),
            algorithm: "MD5".to_string(),
            hash_value: "d41d8cd98f00b204e9800998ecf8427e".to_string(),
            computed_at: "2024-01-01T00:00:00Z".to_string(),
            segment_index: Some(0),
            segment_name: Some("disk.E01".to_string()),
            source: "stored".to_string(),
        };
        assert_eq!(hash.segment_index, Some(0));
        assert_eq!(hash.segment_name, Some("disk.E01".to_string()));
        assert_eq!(hash.source, "stored");
    }

    #[test]
    fn test_verification_record_struct_fields() {
        let verification = VerificationRecord {
            id: "verify-123".to_string(),
            hash_id: "hash-123".to_string(),
            verified_at: "2024-01-01T12:00:00Z".to_string(),
            result: "match".to_string(),
            expected_hash: "abc123".to_string(),
            actual_hash: "abc123".to_string(),
        };
        assert_eq!(verification.result, "match");
        assert_eq!(verification.expected_hash, verification.actual_hash);
    }

    #[test]
    fn test_verification_record_mismatch() {
        let verification = VerificationRecord {
            id: "verify-456".to_string(),
            hash_id: "hash-456".to_string(),
            verified_at: "2024-01-01T12:00:00Z".to_string(),
            result: "mismatch".to_string(),
            expected_hash: "abc123".to_string(),
            actual_hash: "def456".to_string(),
        };
        assert_eq!(verification.result, "mismatch");
        assert_ne!(verification.expected_hash, verification.actual_hash);
    }

    #[test]
    fn test_open_tab_record_struct_fields() {
        let tab = OpenTabRecord {
            id: "tab-123".to_string(),
            session_id: "session-123".to_string(),
            file_path: "/evidence/disk.E01".to_string(),
            tab_order: 0,
            is_active: true,
        };
        assert!(tab.is_active);
        assert_eq!(tab.tab_order, 0);
    }

    #[test]
    fn test_app_settings_struct_fields() {
        let setting = AppSettings {
            key: "theme".to_string(),
            value: "dark".to_string(),
        };
        assert_eq!(setting.key, "theme");
        assert_eq!(setting.value, "dark");
    }

    #[test]
    fn test_create_session() {
        let (db, _temp) = create_test_db();
        let session = db.create_session("test-id", "Test Session", "/path/to/case").unwrap();
        
        assert_eq!(session.id, "test-id");
        assert_eq!(session.name, "Test Session");
        assert_eq!(session.root_path, "/path/to/case");
        assert!(!session.created_at.is_empty());
    }

    #[test]
    fn test_get_session_by_path() {
        let (db, _temp) = create_test_db();
        
        // Initially no session
        let result = db.get_session_by_path("/path/to/case").unwrap();
        assert!(result.is_none());
        
        // Create session
        db.create_session("test-id", "Test", "/path/to/case").unwrap();
        
        // Now should find it
        let result = db.get_session_by_path("/path/to/case").unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "Test");
    }

    #[test]
    fn test_get_or_create_session_creates_new() {
        let (db, _temp) = create_test_db();
        
        let session = db.get_or_create_session("/path/to/new/case").unwrap();
        assert_eq!(session.root_path, "/path/to/new/case");
        assert_eq!(session.name, "case"); // Last path component
    }

    #[test]
    fn test_get_or_create_session_returns_existing() {
        let (db, _temp) = create_test_db();
        
        // Create first session
        let session1 = db.get_or_create_session("/path/to/case").unwrap();
        let id1 = session1.id.clone();
        
        // Get again - should return same session
        let session2 = db.get_or_create_session("/path/to/case").unwrap();
        assert_eq!(session2.id, id1);
    }

    #[test]
    fn test_get_recent_sessions() {
        let (db, _temp) = create_test_db();
        
        db.create_session("s1", "Session 1", "/path/1").unwrap();
        db.create_session("s2", "Session 2", "/path/2").unwrap();
        db.create_session("s3", "Session 3", "/path/3").unwrap();
        
        let sessions = db.get_recent_sessions(2).unwrap();
        assert_eq!(sessions.len(), 2);
    }

    #[test]
    fn test_get_last_session() {
        let (db, _temp) = create_test_db();
        
        // No sessions initially
        let result = db.get_last_session().unwrap();
        assert!(result.is_none());
        
        // Create session
        db.create_session("s1", "Session 1", "/path/1").unwrap();
        
        let result = db.get_last_session().unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "Session 1");
    }

    #[test]
    fn test_upsert_file() {
        let (db, _temp) = create_test_db();
        
        db.create_session("s1", "Session", "/path").unwrap();
        
        let file = FileRecord {
            id: "f1".to_string(),
            session_id: "s1".to_string(),
            path: "/evidence/disk.E01".to_string(),
            filename: "disk.E01".to_string(),
            container_type: "E01".to_string(),
            total_size: 1024,
            segment_count: 1,
            discovered_at: "2024-01-01T00:00:00Z".to_string(),
        };
        
        db.upsert_file(&file).unwrap();
        
        let files = db.get_files_for_session("s1").unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].filename, "disk.E01");
    }

    #[test]
    fn test_upsert_file_updates_existing() {
        let (db, _temp) = create_test_db();
        
        db.create_session("s1", "Session", "/path").unwrap();
        
        let file1 = FileRecord {
            id: "f1".to_string(),
            session_id: "s1".to_string(),
            path: "/evidence/disk.E01".to_string(),
            filename: "disk.E01".to_string(),
            container_type: "E01".to_string(),
            total_size: 1024,
            segment_count: 1,
            discovered_at: "2024-01-01T00:00:00Z".to_string(),
        };
        
        db.upsert_file(&file1).unwrap();
        
        // Update with same path - different values
        let file2 = FileRecord {
            id: "f2".to_string(), // Different ID
            session_id: "s1".to_string(),
            path: "/evidence/disk.E01".to_string(), // Same path
            filename: "disk.E01".to_string(),
            container_type: "E01".to_string(),
            total_size: 2048, // Updated size
            segment_count: 5, // Updated count
            discovered_at: "2024-01-02T00:00:00Z".to_string(),
        };
        
        db.upsert_file(&file2).unwrap();
        
        let files = db.get_files_for_session("s1").unwrap();
        assert_eq!(files.len(), 1); // Still only one file
        assert_eq!(files[0].total_size, 2048);
        assert_eq!(files[0].segment_count, 5);
    }

    #[test]
    fn test_get_file_by_path() {
        let (db, _temp) = create_test_db();
        
        db.create_session("s1", "Session", "/path").unwrap();
        
        let file = FileRecord {
            id: "f1".to_string(),
            session_id: "s1".to_string(),
            path: "/evidence/disk.E01".to_string(),
            filename: "disk.E01".to_string(),
            container_type: "E01".to_string(),
            total_size: 1024,
            segment_count: 1,
            discovered_at: "2024-01-01T00:00:00Z".to_string(),
        };
        
        db.upsert_file(&file).unwrap();
        
        let result = db.get_file_by_path("s1", "/evidence/disk.E01").unwrap();
        assert!(result.is_some());
        
        let result = db.get_file_by_path("s1", "/evidence/other.E01").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_insert_and_get_hashes() {
        let (db, _temp) = create_test_db();
        
        db.create_session("s1", "Session", "/path").unwrap();
        
        let file = FileRecord {
            id: "f1".to_string(),
            session_id: "s1".to_string(),
            path: "/evidence/disk.E01".to_string(),
            filename: "disk.E01".to_string(),
            container_type: "E01".to_string(),
            total_size: 1024,
            segment_count: 1,
            discovered_at: "2024-01-01T00:00:00Z".to_string(),
        };
        db.upsert_file(&file).unwrap();
        
        let hash = HashRecord {
            id: "h1".to_string(),
            file_id: "f1".to_string(),
            algorithm: "SHA-256".to_string(),
            hash_value: "abc123".to_string(),
            computed_at: "2024-01-01T00:00:00Z".to_string(),
            segment_index: None,
            segment_name: None,
            source: "computed".to_string(),
        };
        
        db.insert_hash(&hash).unwrap();
        
        let hashes = db.get_hashes_for_file("f1").unwrap();
        assert_eq!(hashes.len(), 1);
        assert_eq!(hashes[0].algorithm, "SHA-256");
    }

    #[test]
    fn test_get_latest_hash() {
        let (db, _temp) = create_test_db();
        
        db.create_session("s1", "Session", "/path").unwrap();
        
        let file = FileRecord {
            id: "f1".to_string(),
            session_id: "s1".to_string(),
            path: "/evidence/disk.E01".to_string(),
            filename: "disk.E01".to_string(),
            container_type: "E01".to_string(),
            total_size: 1024,
            segment_count: 1,
            discovered_at: "2024-01-01T00:00:00Z".to_string(),
        };
        db.upsert_file(&file).unwrap();
        
        // Insert older hash
        let hash1 = HashRecord {
            id: "h1".to_string(),
            file_id: "f1".to_string(),
            algorithm: "SHA-256".to_string(),
            hash_value: "old_hash".to_string(),
            computed_at: "2024-01-01T00:00:00Z".to_string(),
            segment_index: None,
            segment_name: None,
            source: "computed".to_string(),
        };
        db.insert_hash(&hash1).unwrap();
        
        // Insert newer hash
        let hash2 = HashRecord {
            id: "h2".to_string(),
            file_id: "f1".to_string(),
            algorithm: "SHA-256".to_string(),
            hash_value: "new_hash".to_string(),
            computed_at: "2024-01-02T00:00:00Z".to_string(),
            segment_index: None,
            segment_name: None,
            source: "computed".to_string(),
        };
        db.insert_hash(&hash2).unwrap();
        
        let latest = db.get_latest_hash("f1", "SHA-256", None).unwrap();
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().hash_value, "new_hash");
    }

    #[test]
    fn test_insert_and_get_verifications() {
        let (db, _temp) = create_test_db();
        
        db.create_session("s1", "Session", "/path").unwrap();
        
        let file = FileRecord {
            id: "f1".to_string(),
            session_id: "s1".to_string(),
            path: "/evidence/disk.E01".to_string(),
            filename: "disk.E01".to_string(),
            container_type: "E01".to_string(),
            total_size: 1024,
            segment_count: 1,
            discovered_at: "2024-01-01T00:00:00Z".to_string(),
        };
        db.upsert_file(&file).unwrap();
        
        let hash = HashRecord {
            id: "h1".to_string(),
            file_id: "f1".to_string(),
            algorithm: "SHA-256".to_string(),
            hash_value: "abc123".to_string(),
            computed_at: "2024-01-01T00:00:00Z".to_string(),
            segment_index: None,
            segment_name: None,
            source: "computed".to_string(),
        };
        db.insert_hash(&hash).unwrap();
        
        let verification = VerificationRecord {
            id: "v1".to_string(),
            hash_id: "h1".to_string(),
            verified_at: "2024-01-01T12:00:00Z".to_string(),
            result: "match".to_string(),
            expected_hash: "abc123".to_string(),
            actual_hash: "abc123".to_string(),
        };
        db.insert_verification(&verification).unwrap();
        
        let verifications = db.get_verifications_for_file("f1").unwrap();
        assert_eq!(verifications.len(), 1);
        assert_eq!(verifications[0].result, "match");
    }

    #[test]
    fn test_save_and_get_open_tabs() {
        let (db, _temp) = create_test_db();
        
        db.create_session("s1", "Session", "/path").unwrap();
        
        let tabs = vec![
            OpenTabRecord {
                id: "t1".to_string(),
                session_id: "s1".to_string(),
                file_path: "/evidence/disk.E01".to_string(),
                tab_order: 0,
                is_active: true,
            },
            OpenTabRecord {
                id: "t2".to_string(),
                session_id: "s1".to_string(),
                file_path: "/evidence/other.E01".to_string(),
                tab_order: 1,
                is_active: false,
            },
        ];
        
        db.save_open_tabs("s1", &tabs).unwrap();
        
        let retrieved = db.get_open_tabs("s1").unwrap();
        assert_eq!(retrieved.len(), 2);
        assert!(retrieved[0].is_active);
        assert!(!retrieved[1].is_active);
    }

    #[test]
    fn test_save_tabs_replaces_existing() {
        let (db, _temp) = create_test_db();
        
        db.create_session("s1", "Session", "/path").unwrap();
        
        // Save initial tabs
        let tabs1 = vec![
            OpenTabRecord {
                id: "t1".to_string(),
                session_id: "s1".to_string(),
                file_path: "/evidence/disk1.E01".to_string(),
                tab_order: 0,
                is_active: true,
            },
        ];
        db.save_open_tabs("s1", &tabs1).unwrap();
        
        // Save new tabs (should replace)
        let tabs2 = vec![
            OpenTabRecord {
                id: "t2".to_string(),
                session_id: "s1".to_string(),
                file_path: "/evidence/disk2.E01".to_string(),
                tab_order: 0,
                is_active: true,
            },
        ];
        db.save_open_tabs("s1", &tabs2).unwrap();
        
        let retrieved = db.get_open_tabs("s1").unwrap();
        assert_eq!(retrieved.len(), 1);
        assert_eq!(retrieved[0].file_path, "/evidence/disk2.E01");
    }

    #[test]
    fn test_settings() {
        let (db, _temp) = create_test_db();
        
        // Initially no setting
        let result = db.get_setting("theme").unwrap();
        assert!(result.is_none());
        
        // Set a setting
        db.set_setting("theme", "dark").unwrap();
        
        let result = db.get_setting("theme").unwrap();
        assert_eq!(result, Some("dark".to_string()));
        
        // Update setting
        db.set_setting("theme", "light").unwrap();
        
        let result = db.get_setting("theme").unwrap();
        assert_eq!(result, Some("light".to_string()));
    }
}
