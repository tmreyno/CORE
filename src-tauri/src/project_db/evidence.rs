// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Evidence file, hash, and verification operations.

use super::database::ProjectDatabase;
use super::types::*;
use crate::common::{hash::is_valid_hash, HashAlgorithm};
use rusqlite::{params, Connection, OptionalExtension, Result as SqlResult};
use std::collections::HashMap;
use std::str::FromStr;

const MAX_HASH_SOURCE_REF_JSON_BYTES: usize = 1024 * 1024;
const MAX_HASH_TEXT_FIELD_BYTES: usize = 16 * 1024;
const MAX_EVIDENCE_TEXT_FIELD_BYTES: usize = 16 * 1024;

impl ProjectDatabase {
    // ========================================================================
    // Evidence File Operations
    // ========================================================================

    /// Insert or update an evidence file
    pub fn upsert_evidence_file(&self, file: &DbEvidenceFile) -> SqlResult<()> {
        validate_evidence_file_record(file)?;

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

    /// Batch insert or update evidence files in a single transaction.
    /// Much faster than calling upsert_evidence_file() individually for each file
    /// because it acquires the mutex lock once and wraps all INSERTs in a transaction.
    pub fn batch_upsert_evidence_files(&self, files: &[DbEvidenceFile]) -> SqlResult<usize> {
        if files.is_empty() {
            return Ok(0);
        }
        for file in files {
            validate_evidence_file_record(file)?;
        }

        let conn = self.conn.lock();
        conn.execute_batch("BEGIN IMMEDIATE")?;
        let mut count = 0usize;
        {
            let mut stmt = conn.prepare_cached(
                "INSERT INTO evidence_files (id, path, filename, container_type, total_size, segment_count, discovered_at, created, modified)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                 ON CONFLICT(path) DO UPDATE SET
                    filename = excluded.filename,
                    container_type = excluded.container_type,
                    total_size = excluded.total_size,
                    segment_count = excluded.segment_count,
                    created = COALESCE(excluded.created, evidence_files.created),
                    modified = COALESCE(excluded.modified, evidence_files.modified)",
            )?;
            for file in files {
                stmt.execute(params![
                    file.id,
                    file.path,
                    file.filename,
                    file.container_type,
                    file.total_size,
                    file.segment_count,
                    file.discovered_at,
                    file.created,
                    file.modified,
                ])?;
                count += 1;
            }
        }
        conn.execute_batch("COMMIT")?;
        Ok(count)
    }

    /// Get all evidence files
    pub fn get_evidence_files(&self) -> SqlResult<Vec<DbEvidenceFile>> {
        self.get_evidence_files_limited(None)
    }

    /// Get evidence files with an optional bounded limit.
    pub fn get_evidence_files_limited(&self, limit: Option<i64>) -> SqlResult<Vec<DbEvidenceFile>> {
        let conn = self.conn.lock();
        let limit = limit.unwrap_or(10_000).clamp(1, 100_000);
        let mut stmt = conn.prepare(
            "SELECT id, path, filename, container_type, total_size, segment_count, discovered_at, created, modified
             FROM evidence_files ORDER BY filename LIMIT ?1",
        )?;

        let rows = stmt.query_map(params![limit], |row| {
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
        validate_hash_record(hash)?;

        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO hashes (id, file_id, source_id, source_ref_json, algorithm, hash_value, computed_at, segment_index, segment_name, source)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                hash.id,
                hash.file_id,
                hash.source_id,
                hash.source_ref_json,
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
            "SELECT id, file_id, source_id, source_ref_json, algorithm, hash_value, computed_at, segment_index, segment_name, source
             FROM hashes WHERE file_id = ?1 ORDER BY computed_at DESC",
        )?;

        let rows = stmt.query_map(params![file_id], row_to_hash)?;

        rows.collect()
    }

    /// Get all hashes recorded for a source id.
    pub fn get_hashes_for_source(&self, source_id: &str) -> SqlResult<Vec<DbProjectHash>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, file_id, source_id, source_ref_json, algorithm, hash_value, computed_at, segment_index, segment_name, source
             FROM hashes WHERE source_id = ?1 ORDER BY computed_at DESC",
        )?;

        let rows = stmt.query_map(params![source_id], row_to_hash)?;

        rows.collect()
    }

    /// Summarize hashes by algorithm.
    pub fn summarize_hashes_by_algorithm(&self) -> SqlResult<Vec<DbHashAlgorithmSummary>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT algorithm,
                    COUNT(*) AS count,
                    COUNT(DISTINCT file_id) AS evidence_file_count,
                    COUNT(DISTINCT COALESCE(source_id, file_id || ':' || COALESCE(segment_name, 'whole'))) AS source_count,
                    MAX(computed_at) AS latest_computed_at
             FROM hashes
             GROUP BY algorithm
             ORDER BY algorithm",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(DbHashAlgorithmSummary {
                algorithm: row.get(0)?,
                count: row.get(1)?,
                evidence_file_count: row.get(2)?,
                source_count: row.get(3)?,
                latest_computed_at: row.get(4)?,
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
            "SELECT id, file_id, source_id, source_ref_json, algorithm, hash_value, computed_at, segment_index, segment_name, source
             FROM hashes h
             WHERE h.file_id = ?1
               AND h.algorithm = ?2
               AND h.segment_index IS NULL
               AND (
                   h.source_id IS NULL
                   OR h.source_id = (SELECT path FROM evidence_files WHERE id = ?1)
               )
             ORDER BY h.computed_at DESC LIMIT 1",
        )?;

        let mut rows = stmt.query(params![file_id, algorithm])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row_to_hash(row)?))
        } else {
            Ok(None)
        }
    }

    /// Get the latest hash for a source id/algorithm combination.
    pub fn get_latest_hash_for_source(
        &self,
        source_id: &str,
        algorithm: &str,
    ) -> SqlResult<Option<DbProjectHash>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, file_id, source_id, source_ref_json, algorithm, hash_value, computed_at, segment_index, segment_name, source
             FROM hashes WHERE source_id = ?1 AND algorithm = ?2 AND segment_index IS NULL
             ORDER BY computed_at DESC LIMIT 1",
        )?;

        let mut rows = stmt.query(params![source_id, algorithm])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row_to_hash(row)?))
        } else {
            Ok(None)
        }
    }

    /// Build a latest-hash lookup keyed by source id for one algorithm.
    pub fn latest_source_hash_map(&self, algorithm: &str) -> SqlResult<HashMap<String, String>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT source_id, hash_value
             FROM hashes
             WHERE source_id IS NOT NULL AND algorithm = ?1 AND segment_index IS NULL
             ORDER BY computed_at DESC",
        )?;

        let rows = stmt.query_map(params![algorithm], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut hashes = HashMap::new();
        for row in rows {
            let (source_id, hash_value) = row?;
            hashes.entry(source_id).or_insert(hash_value);
        }
        Ok(hashes)
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
             WHERE f.path = ?1
               AND h.algorithm = ?2
               AND h.segment_index IS NULL
               AND (h.source_id IS NULL OR h.source_id = f.path)
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
        validate_verification_record(v)?;

        let conn = self.conn.lock();
        validate_verification_hash_values(&conn, v)?;
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

    /// Summarize hash verification results by status.
    pub fn summarize_verifications_by_result(&self) -> SqlResult<Vec<DbVerificationResultSummary>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT result,
                    COUNT(*) AS count,
                    COUNT(DISTINCT hash_id) AS hash_count,
                    MAX(verified_at) AS latest_verified_at
             FROM verifications
             GROUP BY result
             ORDER BY result",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(DbVerificationResultSummary {
                result: row.get(0)?,
                count: row.get(1)?,
                hash_count: row.get(2)?,
                latest_verified_at: row.get(3)?,
            })
        })?;
        rows.collect()
    }
}

fn validate_evidence_file_record(file: &DbEvidenceFile) -> SqlResult<()> {
    validate_required_evidence_text_field("id", &file.id, &file.id)?;
    validate_required_evidence_text_field("path", &file.path, &file.id)?;
    validate_required_evidence_text_field("filename", &file.filename, &file.id)?;
    validate_required_evidence_text_field("container_type", &file.container_type, &file.id)?;
    validate_required_evidence_text_field("discovered_at", &file.discovered_at, &file.id)?;
    validate_optional_evidence_text_field("created", file.created.as_deref(), &file.id)?;
    validate_optional_evidence_text_field("modified", file.modified.as_deref(), &file.id)?;

    if file.total_size < 0 {
        return Err(evidence_validation_error(format!(
            "Evidence total_size cannot be negative for {}: {}",
            evidence_id_for_error(&file.id),
            file.total_size
        )));
    }
    if file.segment_count < 1 {
        return Err(evidence_validation_error(format!(
            "Evidence segment_count must be at least 1 for {}: {}",
            evidence_id_for_error(&file.id),
            file.segment_count
        )));
    }

    Ok(())
}

fn validate_required_evidence_text_field(
    field_name: &str,
    value: &str,
    evidence_id: &str,
) -> SqlResult<()> {
    if value.trim().is_empty() {
        return Err(evidence_validation_error(format!(
            "Evidence {field_name} cannot be blank for {}",
            evidence_id_for_error(evidence_id)
        )));
    }

    validate_evidence_text_field_size(field_name, value, evidence_id)
}

fn validate_optional_evidence_text_field(
    field_name: &str,
    value: Option<&str>,
    evidence_id: &str,
) -> SqlResult<()> {
    let Some(value) = value else {
        return Ok(());
    };

    if value.trim().is_empty() {
        return Err(evidence_validation_error(format!(
            "Evidence {field_name} cannot be blank for {}",
            evidence_id_for_error(evidence_id)
        )));
    }

    validate_evidence_text_field_size(field_name, value, evidence_id)
}

fn validate_evidence_text_field_size(
    field_name: &str,
    value: &str,
    evidence_id: &str,
) -> SqlResult<()> {
    if value.len() > MAX_EVIDENCE_TEXT_FIELD_BYTES {
        return Err(evidence_validation_error(format!(
            "Evidence {field_name} exceeds {MAX_EVIDENCE_TEXT_FIELD_BYTES} bytes for {}",
            evidence_id_for_error(evidence_id)
        )));
    }

    Ok(())
}

fn evidence_id_for_error(evidence_id: &str) -> &str {
    if evidence_id.trim().is_empty() {
        "<blank>"
    } else {
        evidence_id
    }
}

fn evidence_validation_error(message: String) -> rusqlite::Error {
    rusqlite::Error::InvalidParameterName(message)
}

fn validate_hash_record(hash: &DbProjectHash) -> SqlResult<()> {
    validate_required_hash_text_field("id", &hash.id, &hash.id)?;
    validate_required_hash_text_field("file_id", &hash.file_id, &hash.id)?;
    validate_optional_hash_text_field("source_id", hash.source_id.as_deref(), &hash.id)?;
    validate_required_hash_text_field("algorithm", &hash.algorithm, &hash.id)?;
    validate_required_hash_text_field("hash_value", &hash.hash_value, &hash.id)?;
    validate_required_hash_text_field("computed_at", &hash.computed_at, &hash.id)?;
    validate_optional_hash_text_field("segment_name", hash.segment_name.as_deref(), &hash.id)?;
    validate_required_hash_text_field("source", &hash.source, &hash.id)?;

    let algorithm = HashAlgorithm::from_str(&hash.algorithm).map_err(|e| {
        hash_validation_error(format!(
            "Invalid hash algorithm for {}: {}",
            hash_record_id_for_error(&hash.id),
            e
        ))
    })?;

    if hash.algorithm != algorithm.name() {
        return Err(hash_validation_error(format!(
            "Hash algorithm must use canonical name {} for {}",
            algorithm.name(),
            hash_record_id_for_error(&hash.id)
        )));
    }

    if !is_valid_hash(hash.hash_value.trim(), algorithm) {
        return Err(hash_validation_error(format!(
            "Hash value is not a valid {} digest for {}",
            algorithm.name(),
            hash_record_id_for_error(&hash.id)
        )));
    }

    if hash.segment_index.is_some_and(|index| index < 0) {
        return Err(hash_validation_error(format!(
            "Hash segment_index cannot be negative for {}",
            hash_record_id_for_error(&hash.id)
        )));
    }

    if let Some(source_ref_json) = &hash.source_ref_json {
        validate_hash_json_field("source_ref_json", source_ref_json)?;
        if source_ref_json.len() > MAX_HASH_SOURCE_REF_JSON_BYTES {
            return Err(hash_validation_error(format!(
                "Hash source_ref_json exceeds {MAX_HASH_SOURCE_REF_JSON_BYTES} bytes for {}",
                hash_record_id_for_error(&hash.id)
            )));
        }
    }

    Ok(())
}

fn validate_verification_record(v: &DbProjectVerification) -> SqlResult<()> {
    validate_required_hash_text_field("id", &v.id, &v.id)?;
    validate_required_hash_text_field("hash_id", &v.hash_id, &v.id)?;
    validate_required_hash_text_field("verified_at", &v.verified_at, &v.id)?;
    validate_required_hash_text_field("result", &v.result, &v.id)?;
    validate_required_hash_text_field("expected_hash", &v.expected_hash, &v.id)?;
    validate_required_hash_text_field("actual_hash", &v.actual_hash, &v.id)
}

fn validate_verification_hash_values(
    conn: &Connection,
    v: &DbProjectVerification,
) -> SqlResult<()> {
    let algorithm = conn
        .query_row(
            "SELECT algorithm FROM hashes WHERE id = ?1",
            params![v.hash_id],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .ok_or_else(|| {
            hash_validation_error(format!(
                "Verification references unknown hash {} for {}",
                v.hash_id,
                hash_record_id_for_error(&v.id)
            ))
        })?;
    let algorithm = HashAlgorithm::from_str(&algorithm).map_err(|e| {
        hash_validation_error(format!(
            "Invalid verification hash algorithm for {}: {}",
            hash_record_id_for_error(&v.id),
            e
        ))
    })?;

    if !is_valid_hash(v.expected_hash.trim(), algorithm) {
        return Err(hash_validation_error(format!(
            "Verification expected_hash is not a valid {} digest for {}",
            algorithm.name(),
            hash_record_id_for_error(&v.id)
        )));
    }
    if !is_valid_hash(v.actual_hash.trim(), algorithm) {
        return Err(hash_validation_error(format!(
            "Verification actual_hash is not a valid {} digest for {}",
            algorithm.name(),
            hash_record_id_for_error(&v.id)
        )));
    }

    Ok(())
}

fn validate_required_hash_text_field(
    field_name: &str,
    value: &str,
    record_id: &str,
) -> SqlResult<()> {
    if value.trim().is_empty() {
        return Err(hash_validation_error(format!(
            "Hash {field_name} cannot be blank for {}",
            hash_record_id_for_error(record_id)
        )));
    }

    validate_hash_text_field_size(field_name, value, record_id)
}

fn validate_optional_hash_text_field(
    field_name: &str,
    value: Option<&str>,
    record_id: &str,
) -> SqlResult<()> {
    let Some(value) = value else {
        return Ok(());
    };

    if value.trim().is_empty() {
        return Err(hash_validation_error(format!(
            "Hash {field_name} cannot be blank for {}",
            hash_record_id_for_error(record_id)
        )));
    }

    validate_hash_text_field_size(field_name, value, record_id)
}

fn validate_hash_text_field_size(field_name: &str, value: &str, record_id: &str) -> SqlResult<()> {
    if value.len() > MAX_HASH_TEXT_FIELD_BYTES {
        return Err(hash_validation_error(format!(
            "Hash {field_name} exceeds {MAX_HASH_TEXT_FIELD_BYTES} bytes for {}",
            hash_record_id_for_error(record_id)
        )));
    }

    Ok(())
}

fn validate_hash_json_field(field_name: &str, value: &str) -> SqlResult<()> {
    serde_json::from_str::<serde_json::Value>(value)
        .map(|_| ())
        .map_err(|e| hash_validation_error(format!("Invalid hash {field_name}: {e}")))
}

fn hash_record_id_for_error(record_id: &str) -> &str {
    if record_id.trim().is_empty() {
        "<blank>"
    } else {
        record_id
    }
}

fn hash_validation_error(message: String) -> rusqlite::Error {
    rusqlite::Error::InvalidParameterName(message)
}

fn row_to_hash(row: &rusqlite::Row<'_>) -> SqlResult<DbProjectHash> {
    Ok(DbProjectHash {
        id: row.get(0)?,
        file_id: row.get(1)?,
        source_id: row.get(2)?,
        source_ref_json: row.get(3)?,
        algorithm: row.get(4)?,
        hash_value: row.get(5)?,
        computed_at: row.get(6)?,
        segment_index: row.get(7)?,
        segment_name: row.get(8)?,
        source: row.get(9)?,
    })
}
