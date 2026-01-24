// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Incremental Container Indexing - Cache container metadata for fast access

use std::path::{Path, PathBuf};
use rusqlite::{Connection, params};
use tracing::{debug, info};

/// Cached index entry for a file within a container
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexEntry {
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
    pub modified_time: Option<i64>,
    pub hash: Option<String>,
}

/// Container index summary (lightweight)
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexSummary {
    pub container_path: String,
    pub total_files: usize,
    pub total_dirs: usize,
    pub total_size: u64,
    pub indexed_at: i64,
    pub is_complete: bool,
}

/// Cache statistics
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheStats {
    pub total_containers: usize,
    pub total_entries: usize,
    pub db_size_bytes: u64,
}

/// Incremental index cache manager (uses connection-per-operation for thread safety)
#[derive(Clone, Debug)]
pub struct IndexCache {
    db_path: PathBuf,
}

impl IndexCache {
    pub fn new(db_path: impl AsRef<Path>) -> Result<Self, String> {
        let db_path = db_path.as_ref().to_path_buf();
        let conn = Self::open_connection(&db_path)?;
        Self::create_schema(&conn)?;
        drop(conn);
        info!(db_path = %db_path.display(), "Index cache initialized");
        Ok(Self { db_path })
    }
    
    fn open_connection(db_path: &Path) -> Result<Connection, String> {
        Connection::open(db_path).map_err(|e| format!("Failed to open database: {}", e))
    }
    
    fn create_schema(conn: &Connection) -> Result<(), String> {
        conn.execute("CREATE TABLE IF NOT EXISTS container_index (id INTEGER PRIMARY KEY, container_path TEXT UNIQUE, indexed_at INTEGER, total_files INTEGER, total_dirs INTEGER, total_size INTEGER, is_complete INTEGER)", []).map_err(|e| format!("Schema error: {}", e))?;
        conn.execute("CREATE TABLE IF NOT EXISTS index_entries (id INTEGER PRIMARY KEY, container_id INTEGER, entry_path TEXT, size INTEGER, is_dir INTEGER, modified_time INTEGER, hash TEXT)", []).map_err(|e| format!("Schema error: {}", e))?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_container_path ON container_index(container_path)", []).map_err(|e| format!("Index error: {}", e))?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_entry_container ON index_entries(container_id)", []).map_err(|e| format!("Index error: {}", e))?;
        Ok(())
    }
    
    pub fn has_index(&self, container_path: &str) -> Result<bool, String> {
        let conn = Self::open_connection(&self.db_path)?;
        let meta = std::fs::metadata(container_path).map_err(|e| format!("Stat error: {}", e))?;
        let mtime = meta.modified().map_err(|e| format!("Mtime error: {}", e))?.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
        let mut stmt = conn.prepare("SELECT indexed_at FROM container_index WHERE container_path = ? AND is_complete = 1").map_err(|e| format!("Query error: {}", e))?;
        let result = stmt.query_row(params![container_path], |row| { let indexed_at: i64 = row.get(0)?; Ok(indexed_at >= mtime) });
        match result { Ok(valid) => Ok(valid), Err(rusqlite::Error::QueryReturnedNoRows) => Ok(false), Err(e) => Err(format!("Query error: {}", e)) }
    }
    
    pub fn get_summary(&self, container_path: &str) -> Result<Option<IndexSummary>, String> {
        let conn = Self::open_connection(&self.db_path)?;
        let mut stmt = conn.prepare("SELECT container_path, total_files, total_dirs, total_size, indexed_at, is_complete FROM container_index WHERE container_path = ?").map_err(|e| format!("Query error: {}", e))?;
        let result = stmt.query_row(params![container_path], |row| Ok(IndexSummary { container_path: row.get(0)?, total_files: row.get::<_, i64>(1)? as usize, total_dirs: row.get::<_, i64>(2)? as usize, total_size: row.get::<_, i64>(3)? as u64, indexed_at: row.get(4)?, is_complete: row.get::<_, i64>(5)? != 0 }));
        match result { Ok(summary) => Ok(Some(summary)), Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None), Err(e) => Err(format!("Query error: {}", e)) }
    }
    
    pub fn store_index(&self, container_path: &str, entries: &[IndexEntry], is_complete: bool) -> Result<(), String> {
        let mut conn = Self::open_connection(&self.db_path)?;
        let tx = conn.transaction().map_err(|e| format!("Transaction error: {}", e))?;
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
        let total_files = entries.iter().filter(|e| !e.is_dir).count();
        let total_dirs = entries.iter().filter(|e| e.is_dir).count();
        let total_size: u64 = entries.iter().map(|e| e.size).sum();
        tx.execute("INSERT OR REPLACE INTO container_index (container_path, indexed_at, total_files, total_dirs, total_size, is_complete) VALUES (?, ?, ?, ?, ?, ?)", params![container_path, now, total_files as i64, total_dirs as i64, total_size as i64, if is_complete { 1 } else { 0 }]).map_err(|e| format!("Insert error: {}", e))?;
        let container_id: i64 = tx.query_row("SELECT id FROM container_index WHERE container_path = ?", params![container_path], |row| row.get(0)).map_err(|e| format!("Query error: {}", e))?;
        tx.execute("DELETE FROM index_entries WHERE container_id = ?", params![container_id]).map_err(|e| format!("Delete error: {}", e))?;
        let mut stmt = tx.prepare("INSERT INTO index_entries (container_id, entry_path, size, is_dir, modified_time, hash) VALUES (?, ?, ?, ?, ?, ?)").map_err(|e| format!("Prepare error: {}", e))?;
        for entry in entries { stmt.execute(params![container_id, &entry.path, entry.size as i64, if entry.is_dir { 1 } else { 0 }, entry.modified_time, &entry.hash]).map_err(|e| format!("Insert error: {}", e))?; }
        drop(stmt);
        tx.commit().map_err(|e| format!("Commit error: {}", e))?;
        debug!(container = %container_path, entries = entries.len(), "Index stored");
        Ok(())
    }
    
    pub fn load_index(&self, container_path: &str) -> Result<Vec<IndexEntry>, String> {
        let conn = Self::open_connection(&self.db_path)?;
        let container_id: i64 = conn.query_row("SELECT id FROM container_index WHERE container_path = ?", params![container_path], |row| row.get(0)).map_err(|e| match e { rusqlite::Error::QueryReturnedNoRows => "Not indexed".to_string(), _ => format!("Query error: {}", e) })?;
        let mut stmt = conn.prepare("SELECT entry_path, size, is_dir, modified_time, hash FROM index_entries WHERE container_id = ?").map_err(|e| format!("Query error: {}", e))?;
        let entries = stmt.query_map(params![container_id], |row| Ok(IndexEntry { path: row.get(0)?, size: row.get::<_, i64>(1)? as u64, is_dir: row.get::<_, i64>(2)? != 0, modified_time: row.get(3)?, hash: row.get(4)? })).map_err(|e| format!("Query error: {}", e))?.collect::<Result<Vec<_>, _>>().map_err(|e| format!("Collect error: {}", e))?;
        debug!(container = %container_path, count = entries.len(), "Index loaded");
        Ok(entries)
    }
    
    pub fn invalidate(&self, container_path: &str) -> Result<(), String> {
        let conn = Self::open_connection(&self.db_path)?;
        conn.execute("DELETE FROM container_index WHERE container_path = ?", params![container_path]).map_err(|e| format!("Delete error: {}", e))?;
        debug!(container = %container_path, "Index invalidated");
        Ok(())
    }
    
    pub fn get_stats(&self) -> Result<CacheStats, String> {
        let conn = Self::open_connection(&self.db_path)?;
        let total_containers: i64 = conn.query_row("SELECT COUNT(*) FROM container_index", [], |row| row.get(0)).unwrap_or(0);
        let total_entries: i64 = conn.query_row("SELECT COUNT(*) FROM index_entries", [], |row| row.get(0)).unwrap_or(0);
        let db_size = std::fs::metadata(&self.db_path).map(|m| m.len()).unwrap_or(0);
        Ok(CacheStats { total_containers: total_containers as usize, total_entries: total_entries as usize, db_size_bytes: db_size })
    }
    
    pub fn clear_all(&self) -> Result<(), String> {
        let conn = Self::open_connection(&self.db_path)?;
        conn.execute("DELETE FROM index_entries", []).map_err(|e| format!("Clear error: {}", e))?;
        conn.execute("DELETE FROM container_index", []).map_err(|e| format!("Clear error: {}", e))?;
        info!("Cache cleared");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cache_create() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_index.db");
        let _ = std::fs::remove_file(&db_path);
        let cache = IndexCache::new(&db_path).unwrap();
        assert!(db_path.exists());
        let stats = cache.get_stats().unwrap();
        assert_eq!(stats.total_containers, 0);
    }
    
    #[test]
    fn test_store_and_load() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_store_load.db");
        let _ = std::fs::remove_file(&db_path);
        let cache = IndexCache::new(&db_path).unwrap();
        let entries = vec![IndexEntry { path: "/file.txt".to_string(), size: 1024, is_dir: false, modified_time: Some(123456), hash: Some("abc123".to_string()) }];
        cache.store_index("test.ad1", &entries, true).unwrap();
        let summary = cache.get_summary("test.ad1").unwrap().unwrap();
        assert_eq!(summary.total_files, 1);
        let loaded = cache.load_index("test.ad1").unwrap();
        assert_eq!(loaded.len(), 1);
    }
}
