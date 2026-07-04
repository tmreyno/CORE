// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Processed database operations (AXIOM, Cellebrite, Autopsy snapshots).

use super::database::ProjectDatabase;
use super::types::*;
use rusqlite::{params, Result as SqlResult};

const MAX_PROCESSED_JSON_BYTES: usize = 1024 * 1024;
const MAX_PROCESSED_TEXT_FIELD_BYTES: usize = 16 * 1024;

impl ProjectDatabase {
    // ========================================================================
    // Processed Database Operations
    // ========================================================================

    /// Insert or update a processed database record
    pub fn upsert_processed_database(&self, db: &DbProcessedDatabase) -> SqlResult<()> {
        validate_processed_database_record(db)?;

        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO processed_databases (id, path, name, db_type, case_number, examiner, created_date, total_size, artifact_count, notes, registered_at, metadata_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
             ON CONFLICT(path) DO UPDATE SET
                name = excluded.name,
                db_type = excluded.db_type,
                case_number = excluded.case_number,
                examiner = excluded.examiner,
                created_date = excluded.created_date,
                total_size = excluded.total_size,
                artifact_count = excluded.artifact_count,
                notes = excluded.notes,
                metadata_json = excluded.metadata_json",
            params![
                db.id, db.path, db.name, db.db_type, db.case_number, db.examiner,
                db.created_date, db.total_size, db.artifact_count, db.notes,
                db.registered_at, db.metadata_json,
            ],
        )?;
        Ok(())
    }

    /// Get all processed databases
    pub fn get_processed_databases(&self) -> SqlResult<Vec<DbProcessedDatabase>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, path, name, db_type, case_number, examiner, created_date, total_size, artifact_count, notes, registered_at, metadata_json
             FROM processed_databases ORDER BY name",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(DbProcessedDatabase {
                id: row.get(0)?,
                path: row.get(1)?,
                name: row.get(2)?,
                db_type: row.get(3)?,
                case_number: row.get(4)?,
                examiner: row.get(5)?,
                created_date: row.get(6)?,
                total_size: row.get(7)?,
                artifact_count: row.get(8)?,
                notes: row.get(9)?,
                registered_at: row.get(10)?,
                metadata_json: row.get(11)?,
            })
        })?;

        rows.collect()
    }

    /// Get a processed database by path
    pub fn get_processed_database_by_path(
        &self,
        path: &str,
    ) -> SqlResult<Option<DbProcessedDatabase>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, path, name, db_type, case_number, examiner, created_date, total_size, artifact_count, notes, registered_at, metadata_json
             FROM processed_databases WHERE path = ?1",
        )?;

        let mut rows = stmt.query(params![path])?;
        if let Some(row) = rows.next()? {
            Ok(Some(DbProcessedDatabase {
                id: row.get(0)?,
                path: row.get(1)?,
                name: row.get(2)?,
                db_type: row.get(3)?,
                case_number: row.get(4)?,
                examiner: row.get(5)?,
                created_date: row.get(6)?,
                total_size: row.get(7)?,
                artifact_count: row.get(8)?,
                notes: row.get(9)?,
                registered_at: row.get(10)?,
                metadata_json: row.get(11)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Delete a processed database and all related records (cascades)
    pub fn delete_processed_database(&self, id: &str) -> SqlResult<()> {
        let conn = self.conn.lock();
        conn.execute(
            "DELETE FROM artifact_categories WHERE processed_db_id = ?1",
            params![id],
        )?;
        conn.execute(
            "DELETE FROM processed_db_metrics WHERE processed_db_id = ?1",
            params![id],
        )?;
        conn.execute(
            "DELETE FROM processed_db_integrity WHERE processed_db_id = ?1",
            params![id],
        )?;
        let axiom_ids: Vec<String> = {
            let mut stmt =
                conn.prepare("SELECT id FROM axiom_case_info WHERE processed_db_id = ?1")?;
            let rows = stmt.query_map(params![id], |row| row.get(0))?;
            rows.filter_map(|r| r.ok()).collect()
        };
        for axiom_id in &axiom_ids {
            conn.execute(
                "DELETE FROM axiom_evidence_sources WHERE axiom_case_id = ?1",
                params![axiom_id],
            )?;
            conn.execute(
                "DELETE FROM axiom_search_results WHERE axiom_case_id = ?1",
                params![axiom_id],
            )?;
        }
        conn.execute(
            "DELETE FROM axiom_case_info WHERE processed_db_id = ?1",
            params![id],
        )?;
        conn.execute("DELETE FROM processed_databases WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ========================================================================
    // Processed DB Integrity Operations
    // ========================================================================

    /// Insert or update an integrity record
    pub fn upsert_processed_db_integrity(&self, i: &DbProcessedDbIntegrity) -> SqlResult<()> {
        validate_processed_db_integrity_record(i)?;

        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO processed_db_integrity (id, processed_db_id, file_path, file_size, baseline_hash, baseline_timestamp, current_hash, current_hash_timestamp, status, changes_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(id) DO UPDATE SET
                file_size = excluded.file_size,
                current_hash = excluded.current_hash,
                current_hash_timestamp = excluded.current_hash_timestamp,
                status = excluded.status,
                changes_json = excluded.changes_json",
            params![
                i.id, i.processed_db_id, i.file_path, i.file_size,
                i.baseline_hash, i.baseline_timestamp,
                i.current_hash, i.current_hash_timestamp,
                i.status, i.changes_json,
            ],
        )?;
        Ok(())
    }

    /// Get integrity records for a processed database
    pub fn get_processed_db_integrity(
        &self,
        processed_db_id: &str,
    ) -> SqlResult<Vec<DbProcessedDbIntegrity>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, processed_db_id, file_path, file_size, baseline_hash, baseline_timestamp, current_hash, current_hash_timestamp, status, changes_json
             FROM processed_db_integrity WHERE processed_db_id = ?1",
        )?;

        let rows = stmt.query_map(params![processed_db_id], |row| {
            Ok(DbProcessedDbIntegrity {
                id: row.get(0)?,
                processed_db_id: row.get(1)?,
                file_path: row.get(2)?,
                file_size: row.get(3)?,
                baseline_hash: row.get(4)?,
                baseline_timestamp: row.get(5)?,
                current_hash: row.get(6)?,
                current_hash_timestamp: row.get(7)?,
                status: row.get(8)?,
                changes_json: row.get(9)?,
            })
        })?;

        rows.collect()
    }

    // ========================================================================
    // Processed DB Metrics Operations
    // ========================================================================

    /// Insert or update metrics for a processed database
    pub fn upsert_processed_db_metrics(&self, m: &DbProcessedDbMetrics) -> SqlResult<()> {
        validate_processed_db_metrics_record(m)?;

        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO processed_db_metrics (id, processed_db_id, total_scans, last_scan_date, total_jobs, last_job_date, total_notes, total_tagged_items, total_users, user_names_json, captured_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
             ON CONFLICT(id) DO UPDATE SET
                total_scans = excluded.total_scans,
                last_scan_date = excluded.last_scan_date,
                total_jobs = excluded.total_jobs,
                last_job_date = excluded.last_job_date,
                total_notes = excluded.total_notes,
                total_tagged_items = excluded.total_tagged_items,
                total_users = excluded.total_users,
                user_names_json = excluded.user_names_json,
                captured_at = excluded.captured_at",
            params![
                m.id, m.processed_db_id, m.total_scans, m.last_scan_date,
                m.total_jobs, m.last_job_date, m.total_notes, m.total_tagged_items,
                m.total_users, m.user_names_json, m.captured_at,
            ],
        )?;
        Ok(())
    }

    /// Get metrics for a processed database
    pub fn get_processed_db_metrics(
        &self,
        processed_db_id: &str,
    ) -> SqlResult<Option<DbProcessedDbMetrics>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, processed_db_id, total_scans, last_scan_date, total_jobs, last_job_date, total_notes, total_tagged_items, total_users, user_names_json, captured_at
             FROM processed_db_metrics WHERE processed_db_id = ?1
             ORDER BY captured_at DESC LIMIT 1",
        )?;

        let mut rows = stmt.query(params![processed_db_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(DbProcessedDbMetrics {
                id: row.get(0)?,
                processed_db_id: row.get(1)?,
                total_scans: row.get(2)?,
                last_scan_date: row.get(3)?,
                total_jobs: row.get(4)?,
                last_job_date: row.get(5)?,
                total_notes: row.get(6)?,
                total_tagged_items: row.get(7)?,
                total_users: row.get(8)?,
                user_names_json: row.get(9)?,
                captured_at: row.get(10)?,
            }))
        } else {
            Ok(None)
        }
    }

    // ========================================================================
    // AXIOM Case Info Operations
    // ========================================================================

    /// Insert or update AXIOM case information
    pub fn upsert_axiom_case_info(&self, a: &DbAxiomCaseInfo) -> SqlResult<()> {
        validate_axiom_case_info_record(a)?;

        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO axiom_case_info (id, processed_db_id, case_name, case_number, case_type, description, examiner, agency, axiom_version, search_start, search_end, search_duration, search_outcome, output_folder, total_artifacts, case_path, captured_at, keyword_info_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)
             ON CONFLICT(id) DO UPDATE SET
                case_name = excluded.case_name,
                case_number = excluded.case_number,
                case_type = excluded.case_type,
                description = excluded.description,
                examiner = excluded.examiner,
                agency = excluded.agency,
                axiom_version = excluded.axiom_version,
                search_start = excluded.search_start,
                search_end = excluded.search_end,
                search_duration = excluded.search_duration,
                search_outcome = excluded.search_outcome,
                output_folder = excluded.output_folder,
                total_artifacts = excluded.total_artifacts,
                case_path = excluded.case_path,
                captured_at = excluded.captured_at,
                keyword_info_json = excluded.keyword_info_json",
            params![
                a.id, a.processed_db_id, a.case_name, a.case_number,
                a.case_type, a.description, a.examiner, a.agency,
                a.axiom_version, a.search_start, a.search_end,
                a.search_duration, a.search_outcome, a.output_folder,
                a.total_artifacts, a.case_path, a.captured_at, a.keyword_info_json,
            ],
        )?;
        Ok(())
    }

    /// Get AXIOM case info for a processed database
    pub fn get_axiom_case_info(&self, processed_db_id: &str) -> SqlResult<Option<DbAxiomCaseInfo>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, processed_db_id, case_name, case_number, case_type, description, examiner, agency, axiom_version, search_start, search_end, search_duration, search_outcome, output_folder, total_artifacts, case_path, captured_at, keyword_info_json
             FROM axiom_case_info WHERE processed_db_id = ?1
             ORDER BY captured_at DESC LIMIT 1",
        )?;

        let mut rows = stmt.query(params![processed_db_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(DbAxiomCaseInfo {
                id: row.get(0)?,
                processed_db_id: row.get(1)?,
                case_name: row.get(2)?,
                case_number: row.get(3)?,
                case_type: row.get(4)?,
                description: row.get(5)?,
                examiner: row.get(6)?,
                agency: row.get(7)?,
                axiom_version: row.get(8)?,
                search_start: row.get(9)?,
                search_end: row.get(10)?,
                search_duration: row.get(11)?,
                search_outcome: row.get(12)?,
                output_folder: row.get(13)?,
                total_artifacts: row.get(14)?,
                case_path: row.get(15)?,
                captured_at: row.get(16)?,
                keyword_info_json: row.get(17)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Get all AXIOM case info records
    pub fn get_all_axiom_case_info(&self) -> SqlResult<Vec<DbAxiomCaseInfo>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, processed_db_id, case_name, case_number, case_type, description, examiner, agency, axiom_version, search_start, search_end, search_duration, search_outcome, output_folder, total_artifacts, case_path, captured_at, keyword_info_json
             FROM axiom_case_info ORDER BY captured_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(DbAxiomCaseInfo {
                id: row.get(0)?,
                processed_db_id: row.get(1)?,
                case_name: row.get(2)?,
                case_number: row.get(3)?,
                case_type: row.get(4)?,
                description: row.get(5)?,
                examiner: row.get(6)?,
                agency: row.get(7)?,
                axiom_version: row.get(8)?,
                search_start: row.get(9)?,
                search_end: row.get(10)?,
                search_duration: row.get(11)?,
                search_outcome: row.get(12)?,
                output_folder: row.get(13)?,
                total_artifacts: row.get(14)?,
                case_path: row.get(15)?,
                captured_at: row.get(16)?,
                keyword_info_json: row.get(17)?,
            })
        })?;

        rows.collect()
    }

    // ========================================================================
    // AXIOM Evidence Source Operations
    // ========================================================================

    /// Insert an AXIOM evidence source
    pub fn insert_axiom_evidence_source(&self, s: &DbAxiomEvidenceSource) -> SqlResult<()> {
        validate_axiom_evidence_source_record(s)?;

        let conn = self.conn.lock();
        conn.execute(
            "INSERT OR IGNORE INTO axiom_evidence_sources (id, axiom_case_id, name, evidence_number, source_type, path, hash, size, acquired, search_types_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                s.id, s.axiom_case_id, s.name, s.evidence_number,
                s.source_type, s.path, s.hash, s.size, s.acquired, s.search_types_json,
            ],
        )?;
        Ok(())
    }

    /// Get evidence sources for an AXIOM case
    pub fn get_axiom_evidence_sources(
        &self,
        axiom_case_id: &str,
    ) -> SqlResult<Vec<DbAxiomEvidenceSource>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, axiom_case_id, name, evidence_number, source_type, path, hash, size, acquired, search_types_json
             FROM axiom_evidence_sources WHERE axiom_case_id = ?1",
        )?;

        let rows = stmt.query_map(params![axiom_case_id], |row| {
            Ok(DbAxiomEvidenceSource {
                id: row.get(0)?,
                axiom_case_id: row.get(1)?,
                name: row.get(2)?,
                evidence_number: row.get(3)?,
                source_type: row.get(4)?,
                path: row.get(5)?,
                hash: row.get(6)?,
                size: row.get(7)?,
                acquired: row.get(8)?,
                search_types_json: row.get(9)?,
            })
        })?;

        rows.collect()
    }

    // ========================================================================
    // AXIOM Search Result Operations
    // ========================================================================

    /// Insert an AXIOM search result
    pub fn insert_axiom_search_result(&self, r: &DbAxiomSearchResult) -> SqlResult<()> {
        validate_axiom_search_result_record(r)?;

        let conn = self.conn.lock();
        conn.execute(
            "INSERT OR IGNORE INTO axiom_search_results (id, axiom_case_id, artifact_type, hit_count)
             VALUES (?1, ?2, ?3, ?4)",
            params![r.id, r.axiom_case_id, r.artifact_type, r.hit_count],
        )?;
        Ok(())
    }

    /// Get search results for an AXIOM case
    pub fn get_axiom_search_results(
        &self,
        axiom_case_id: &str,
    ) -> SqlResult<Vec<DbAxiomSearchResult>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, axiom_case_id, artifact_type, hit_count
             FROM axiom_search_results WHERE axiom_case_id = ?1 ORDER BY hit_count DESC",
        )?;

        let rows = stmt.query_map(params![axiom_case_id], |row| {
            Ok(DbAxiomSearchResult {
                id: row.get(0)?,
                axiom_case_id: row.get(1)?,
                artifact_type: row.get(2)?,
                hit_count: row.get(3)?,
            })
        })?;

        rows.collect()
    }

    // ========================================================================
    // Artifact Category Operations
    // ========================================================================

    /// Insert or replace artifact categories for a processed database
    pub fn upsert_artifact_categories(&self, categories: &[DbArtifactCategory]) -> SqlResult<()> {
        for category in categories {
            validate_artifact_category_record(category)?;
        }

        let conn = self.conn.lock();
        for c in categories {
            conn.execute(
                "INSERT INTO artifact_categories (id, processed_db_id, category, artifact_type, count)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(id) DO UPDATE SET
                    count = excluded.count",
                params![c.id, c.processed_db_id, c.category, c.artifact_type, c.count],
            )?;
        }
        Ok(())
    }

    /// Get artifact categories for a processed database
    pub fn get_artifact_categories(
        &self,
        processed_db_id: &str,
    ) -> SqlResult<Vec<DbArtifactCategory>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, processed_db_id, category, artifact_type, count
             FROM artifact_categories WHERE processed_db_id = ?1 ORDER BY category, artifact_type",
        )?;

        let rows = stmt.query_map(params![processed_db_id], |row| {
            Ok(DbArtifactCategory {
                id: row.get(0)?,
                processed_db_id: row.get(1)?,
                category: row.get(2)?,
                artifact_type: row.get(3)?,
                count: row.get(4)?,
            })
        })?;

        rows.collect()
    }
}

fn validate_processed_database_record(db: &DbProcessedDatabase) -> SqlResult<()> {
    validate_required_processed_text_field("id", &db.id, &db.id)?;
    validate_required_processed_text_field("path", &db.path, &db.id)?;
    validate_required_processed_text_field("name", &db.name, &db.id)?;
    validate_required_processed_text_field("db_type", &db.db_type, &db.id)?;
    validate_optional_processed_text_field("case_number", db.case_number.as_deref(), &db.id)?;
    validate_optional_processed_text_field("examiner", db.examiner.as_deref(), &db.id)?;
    validate_optional_processed_text_field("created_date", db.created_date.as_deref(), &db.id)?;
    validate_optional_processed_text_field("notes", db.notes.as_deref(), &db.id)?;
    validate_required_processed_text_field("registered_at", &db.registered_at, &db.id)?;
    validate_optional_processed_json_field("metadata_json", db.metadata_json.as_deref(), &db.id)?;
    validate_nonnegative_i64("total_size", db.total_size, &db.id)?;
    validate_optional_nonnegative_i64("artifact_count", db.artifact_count, &db.id)
}

fn validate_processed_db_integrity_record(i: &DbProcessedDbIntegrity) -> SqlResult<()> {
    validate_required_processed_text_field("id", &i.id, &i.id)?;
    validate_required_processed_text_field("processed_db_id", &i.processed_db_id, &i.id)?;
    validate_required_processed_text_field("file_path", &i.file_path, &i.id)?;
    validate_nonnegative_i64("file_size", i.file_size, &i.id)?;
    validate_required_processed_text_field("baseline_hash", &i.baseline_hash, &i.id)?;
    validate_processed_hex_digest("baseline_hash", &i.baseline_hash, &i.id)?;
    validate_required_processed_text_field("baseline_timestamp", &i.baseline_timestamp, &i.id)?;
    if let Some(current_hash) = &i.current_hash {
        validate_required_processed_text_field("current_hash", current_hash, &i.id)?;
        validate_processed_hex_digest("current_hash", current_hash, &i.id)?;
    }
    validate_optional_processed_text_field(
        "current_hash_timestamp",
        i.current_hash_timestamp.as_deref(),
        &i.id,
    )?;
    validate_required_processed_text_field("status", &i.status, &i.id)?;
    validate_optional_processed_json_field("changes_json", i.changes_json.as_deref(), &i.id)
}

fn validate_processed_db_metrics_record(m: &DbProcessedDbMetrics) -> SqlResult<()> {
    validate_required_processed_text_field("id", &m.id, &m.id)?;
    validate_required_processed_text_field("processed_db_id", &m.processed_db_id, &m.id)?;
    validate_nonnegative_i32("total_scans", m.total_scans, &m.id)?;
    validate_optional_processed_text_field("last_scan_date", m.last_scan_date.as_deref(), &m.id)?;
    validate_nonnegative_i32("total_jobs", m.total_jobs, &m.id)?;
    validate_optional_processed_text_field("last_job_date", m.last_job_date.as_deref(), &m.id)?;
    validate_nonnegative_i32("total_notes", m.total_notes, &m.id)?;
    validate_nonnegative_i32("total_tagged_items", m.total_tagged_items, &m.id)?;
    validate_nonnegative_i32("total_users", m.total_users, &m.id)?;
    validate_optional_processed_json_field("user_names_json", m.user_names_json.as_deref(), &m.id)?;
    validate_required_processed_text_field("captured_at", &m.captured_at, &m.id)
}

fn validate_axiom_case_info_record(a: &DbAxiomCaseInfo) -> SqlResult<()> {
    validate_required_processed_text_field("id", &a.id, &a.id)?;
    validate_required_processed_text_field("processed_db_id", &a.processed_db_id, &a.id)?;
    validate_required_processed_text_field("case_name", &a.case_name, &a.id)?;
    validate_optional_processed_text_field("case_number", a.case_number.as_deref(), &a.id)?;
    validate_optional_processed_text_field("case_type", a.case_type.as_deref(), &a.id)?;
    validate_optional_processed_text_field("description", a.description.as_deref(), &a.id)?;
    validate_optional_processed_text_field("examiner", a.examiner.as_deref(), &a.id)?;
    validate_optional_processed_text_field("agency", a.agency.as_deref(), &a.id)?;
    validate_optional_processed_text_field("axiom_version", a.axiom_version.as_deref(), &a.id)?;
    validate_optional_processed_text_field("search_start", a.search_start.as_deref(), &a.id)?;
    validate_optional_processed_text_field("search_end", a.search_end.as_deref(), &a.id)?;
    validate_optional_processed_text_field("search_duration", a.search_duration.as_deref(), &a.id)?;
    validate_optional_processed_text_field("search_outcome", a.search_outcome.as_deref(), &a.id)?;
    validate_optional_processed_text_field("output_folder", a.output_folder.as_deref(), &a.id)?;
    validate_nonnegative_i64("total_artifacts", a.total_artifacts, &a.id)?;
    validate_optional_processed_text_field("case_path", a.case_path.as_deref(), &a.id)?;
    validate_required_processed_text_field("captured_at", &a.captured_at, &a.id)?;
    validate_optional_processed_json_field(
        "keyword_info_json",
        a.keyword_info_json.as_deref(),
        &a.id,
    )
}

fn validate_axiom_evidence_source_record(s: &DbAxiomEvidenceSource) -> SqlResult<()> {
    validate_required_processed_text_field("id", &s.id, &s.id)?;
    validate_required_processed_text_field("axiom_case_id", &s.axiom_case_id, &s.id)?;
    validate_required_processed_text_field("name", &s.name, &s.id)?;
    validate_optional_processed_text_field("evidence_number", s.evidence_number.as_deref(), &s.id)?;
    validate_required_processed_text_field("source_type", &s.source_type, &s.id)?;
    validate_optional_processed_text_field("path", s.path.as_deref(), &s.id)?;
    validate_optional_processed_text_field("hash", s.hash.as_deref(), &s.id)?;
    validate_optional_nonnegative_i64("size", s.size, &s.id)?;
    validate_optional_processed_text_field("acquired", s.acquired.as_deref(), &s.id)?;
    validate_optional_processed_json_field(
        "search_types_json",
        s.search_types_json.as_deref(),
        &s.id,
    )
}

fn validate_axiom_search_result_record(r: &DbAxiomSearchResult) -> SqlResult<()> {
    validate_required_processed_text_field("id", &r.id, &r.id)?;
    validate_required_processed_text_field("axiom_case_id", &r.axiom_case_id, &r.id)?;
    validate_required_processed_text_field("artifact_type", &r.artifact_type, &r.id)?;
    validate_nonnegative_i64("hit_count", r.hit_count, &r.id)
}

fn validate_artifact_category_record(c: &DbArtifactCategory) -> SqlResult<()> {
    validate_required_processed_text_field("id", &c.id, &c.id)?;
    validate_required_processed_text_field("processed_db_id", &c.processed_db_id, &c.id)?;
    validate_required_processed_text_field("category", &c.category, &c.id)?;
    validate_required_processed_text_field("artifact_type", &c.artifact_type, &c.id)?;
    validate_nonnegative_i64("count", c.count, &c.id)
}

fn validate_required_processed_text_field(
    field_name: &str,
    value: &str,
    record_id: &str,
) -> SqlResult<()> {
    if value.trim().is_empty() {
        return Err(processed_validation_error(format!(
            "Processed {field_name} cannot be blank for {}",
            processed_id_for_error(record_id)
        )));
    }

    validate_processed_text_field_size(field_name, value, record_id)
}

fn validate_optional_processed_text_field(
    field_name: &str,
    value: Option<&str>,
    record_id: &str,
) -> SqlResult<()> {
    let Some(value) = value else {
        return Ok(());
    };

    if value.trim().is_empty() {
        return Err(processed_validation_error(format!(
            "Processed {field_name} cannot be blank for {}",
            processed_id_for_error(record_id)
        )));
    }

    validate_processed_text_field_size(field_name, value, record_id)
}

fn validate_processed_text_field_size(
    field_name: &str,
    value: &str,
    record_id: &str,
) -> SqlResult<()> {
    if value.len() > MAX_PROCESSED_TEXT_FIELD_BYTES {
        return Err(processed_validation_error(format!(
            "Processed {field_name} exceeds {MAX_PROCESSED_TEXT_FIELD_BYTES} bytes for {}",
            processed_id_for_error(record_id)
        )));
    }

    Ok(())
}

fn validate_optional_processed_json_field(
    field_name: &str,
    value: Option<&str>,
    record_id: &str,
) -> SqlResult<()> {
    let Some(value) = value else {
        return Ok(());
    };

    serde_json::from_str::<serde_json::Value>(value)
        .map(|_| ())
        .map_err(|e| processed_validation_error(format!("Invalid processed {field_name}: {e}")))?;
    if value.len() > MAX_PROCESSED_JSON_BYTES {
        return Err(processed_validation_error(format!(
            "Processed {field_name} exceeds {MAX_PROCESSED_JSON_BYTES} bytes for {}",
            processed_id_for_error(record_id)
        )));
    }

    Ok(())
}

fn validate_nonnegative_i64(field_name: &str, value: i64, record_id: &str) -> SqlResult<()> {
    if value < 0 {
        return Err(processed_validation_error(format!(
            "Processed {field_name} cannot be negative for {}: {value}",
            processed_id_for_error(record_id)
        )));
    }

    Ok(())
}

fn validate_optional_nonnegative_i64(
    field_name: &str,
    value: Option<i64>,
    record_id: &str,
) -> SqlResult<()> {
    if let Some(value) = value {
        validate_nonnegative_i64(field_name, value, record_id)?;
    }

    Ok(())
}

fn validate_nonnegative_i32(field_name: &str, value: i32, record_id: &str) -> SqlResult<()> {
    if value < 0 {
        return Err(processed_validation_error(format!(
            "Processed {field_name} cannot be negative for {}: {value}",
            processed_id_for_error(record_id)
        )));
    }

    Ok(())
}

fn validate_processed_hex_digest(field_name: &str, value: &str, record_id: &str) -> SqlResult<()> {
    let value = value.trim();
    if !matches!(value.len(), 8 | 16 | 32 | 40 | 64 | 128)
        || !value.chars().all(|ch| ch.is_ascii_hexdigit())
    {
        return Err(processed_validation_error(format!(
            "Processed {field_name} is not a valid hex digest for {}",
            processed_id_for_error(record_id)
        )));
    }

    Ok(())
}

fn processed_id_for_error(record_id: &str) -> &str {
    if record_id.trim().is_empty() {
        "<blank>"
    } else {
        record_id
    }
}

fn processed_validation_error(message: String) -> rusqlite::Error {
    rusqlite::Error::InvalidParameterName(message)
}
