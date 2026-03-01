// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Evidence file, hash, and verification operations.

use super::database::ProjectDatabase;
use super::types::*;
use rusqlite::{params, Result as SqlResult};

impl ProjectDatabase {
    // ========================================================================
    // Evidence File Operations
    // ========================================================================

    /// Insert or update an evidence file
    pub fn upsert_evidence_file(&self, file: &DbEvidenceFile) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO evidence_files (id, path, filename, container_type, total_size, segment_count, discovered_at, created, modified)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(path) DO UPDATE SET
                filename = excluded.filename,
                container_type = excluded.container_type,
                total_size = excluded.total_size,
                segment_count = excluded.segment_count,
                created = COALESCE(excluded.created, evidence_files.created),
                modified = COALESCE(excluded.modified, evidence_files.modified)",
            params![
                file.id,
                file.path,
                file.filename,
                file.container_type,
                file.total_size,
                file.segment_count,
                file.discovered_at,
                file.created,
                file.modified,
            ],
        )?;
        Ok(())
    }

    /// Get all evidence files
    pub fn get_evidence_files(&self) -> SqlResult<Vec<DbEvidenceFile>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, path, filename, container_type, total_size, segment_count, discovered_at, created, modified
             FROM evidence_files ORDER BY filename",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(DbEvidenceFile {
                id: row.get(0)?,
                path: row.get(1)?,
                filename: row.get(2)?,
                container_type: row.get(3)?,
                total_size: row.get(4)?,
                segment_count: row.get(5)?,
                discovered_at: row.get(6)?,
                created: row.get(7)?,
                modified: row.get(8)?,
            })
        })?;

        rows.collect()
    }

    /// Get an evidence file by path
    pub fn get_evidence_file_by_path(&self, path: &str) -> SqlResult<Option<DbEvidenceFile>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, path, filename, container_type, total_size, segment_count, discovered_at, created, modified
             FROM evidence_files WHERE path = ?1",
        )?;

        let mut rows = stmt.query(params![path])?;
        if let Some(row) = rows.next()? {
            Ok(Some(DbEvidenceFile {
                id: row.get(0)?,
                path: row.get(1)?,
                filename: row.get(2)?,
                container_type: row.get(3)?,
                total_size: row.get(4)?,
                segment_count: row.get(5)?,
                discovered_at: row.get(6)?,
                created: row.get(7)?,
                modified: row.get(8)?,
            }))
        } else {
            Ok(None)
        }
    }

    // ========================================================================
    // Hash Operations
    // ========================================================================

    /// Insert a hash record (immutable — no updates)
    pub fn insert_hash(&self, hash: &DbProjectHash) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO hashes (id, file_id, algorithm, hash_value, computed_at, segment_index, segment_name, source)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                hash.id,
                hash.file_id,
                hash.algorithm,
                hash.hash_value,
                hash.computed_at,
                hash.segment_index,
                hash.segment_name,
                hash.source,
            ],
        )?;
        Ok(())
    }

    /// Get all hashes for an evidence file
    pub fn get_hashes_for_file(&self, file_id: &str) -> SqlResult<Vec<DbProjectHash>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, file_id, algorithm, hash_value, computed_at, segment_index, segment_name, source
             FROM hashes WHERE file_id = ?1 ORDER BY computed_at DESC",
        )?;

        let rows = stmt.query_map(params![file_id], |row| {
            Ok(DbProjectHash {
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

    /// Get the latest hash for a file/algorithm combination
    pub fn get_latest_hash(
        &self,
        file_id: &str,
        algorithm: &str,
    ) -> SqlResult<Option<DbProjectHash>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, file_id, algorithm, hash_value, computed_at, segment_index, segment_name, source
             FROM hashes WHERE file_id = ?1 AND algorithm = ?2 AND segment_index IS NULL
             ORDER BY computed_at DESC LIMIT 1",
        )?;

        let mut rows = stmt.query(params![file_id, algorithm])?;
        if let Some(row) = rows.next()? {
            Ok(Some(DbProjectHash {
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

    /// Look up latest known hash for a file by path
    pub fn lookup_hash_by_path(
        &self,
        path: &str,
        algorithm: &str,
    ) -> SqlResult<Option<(String, String)>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT h.hash_value, h.source
             FROM hashes h
             INNER JOIN evidence_files f ON h.file_id = f.id
             WHERE f.path = ?1 AND h.algorithm = ?2 AND h.segment_index IS NULL
             ORDER BY h.computed_at DESC
             LIMIT 1",
        )?;

        let mut rows = stmt.query(params![path, algorithm])?;
        if let Some(row) = rows.next()? {
            Ok(Some((row.get(0)?, row.get(1)?)))
        } else {
            Ok(None)
        }
    }

    // ========================================================================
    // Verification Operations
    // ========================================================================

    /// Insert a verification record (immutable)
    pub fn insert_verification(&self, v: &DbProjectVerification) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO verifications (id, hash_id, verified_at, result, expected_hash, actual_hash)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![v.id, v.hash_id, v.verified_at, v.result, v.expected_hash, v.actual_hash],
        )?;
        Ok(())
    }

    /// Get verifications for a specific hash
    pub fn get_verifications_for_hash(
        &self,
        hash_id: &str,
    ) -> SqlResult<Vec<DbProjectVerification>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, hash_id, verified_at, result, expected_hash, actual_hash
             FROM verifications WHERE hash_id = ?1 ORDER BY verified_at DESC",
        )?;

        let rows = stmt.query_map(params![hash_id], |row| {
            Ok(DbProjectVerification {
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
}
