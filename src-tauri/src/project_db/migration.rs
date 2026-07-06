// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! One-time migration from .cffx FFXProject into .ffxdb database.

use super::database::ProjectDatabase;
use crate::common::{hash::is_valid_hash, HashAlgorithm};
use rusqlite::{params, Result as SqlResult};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::str::FromStr;
use tracing::{info, warn};

impl ProjectDatabase {
    // ========================================================================
    // Migration from FFXProject (one-time import from .cffx)
    // ========================================================================

    /// Migrate data from an FFXProject struct into this database.
    /// Used when opening a project that has data in the .cffx but no .ffxdb yet.
    /// This is idempotent — it uses INSERT OR IGNORE to avoid duplicates.
    pub fn migrate_from_project(&self, project: &crate::project::FFXProject) -> SqlResult<()> {
        let migration_time = chrono::Utc::now().to_rfc3339();
        info!(
            "Migrating project '{}' data to .ffxdb ({} activities, {} sessions, {} users)",
            project.name,
            project.activity_log.len(),
            project.sessions.len(),
            project.users.len(),
        );

        let conn = self.conn.lock();

        // --- Users ---
        for u in &project.users {
            conn.execute(
                "INSERT OR IGNORE INTO users (username, display_name, hostname, first_access, last_access)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![u.username, u.display_name, u.hostname, u.first_access, u.last_access],
            )?;
        }

        // --- Sessions ---
        for s in &project.sessions {
            conn.execute(
                "INSERT OR IGNORE INTO sessions (session_id, user, started_at, ended_at, duration_seconds, hostname, app_version, summary)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    s.session_id, s.user, s.started_at, s.ended_at,
                    s.duration_seconds.map(|d| d as i64),
                    s.hostname, s.app_version, s.summary,
                ],
            )?;
        }

        // --- Activity Log ---
        for a in &project.activity_log {
            let details_json = a
                .details
                .as_ref()
                .and_then(|d| serde_json::to_string(d).ok());
            conn.execute(
                "INSERT OR IGNORE INTO activity_log (id, timestamp, user, category, action, description, file_path, details)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    a.id, a.timestamp, a.user, a.category, a.action,
                    a.description, a.file_path, details_json,
                ],
            )?;
        }

        // --- Bookmarks ---
        for b in &project.bookmarks {
            let context_json = b
                .context
                .as_ref()
                .and_then(|c| serde_json::to_string(c).ok());
            conn.execute(
                "INSERT OR IGNORE INTO bookmarks (id, target_type, target_path, name, created_by, created_at, color, notes, context)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    b.id, b.target_type, b.target_path, b.name,
                    b.created_by, b.created_at, b.color, b.notes, context_json,
                ],
            )?;
        }

        // --- Notes ---
        for n in &project.notes {
            conn.execute(
                "INSERT OR IGNORE INTO notes (id, target_type, target_path, title, content, created_by, created_at, modified_at, priority)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    n.id, n.target_type, n.target_path, n.title, n.content,
                    n.created_by, n.created_at, n.modified_at, n.priority,
                ],
            )?;
        }

        // --- Tags ---
        for t in &project.tags {
            conn.execute(
                "INSERT OR IGNORE INTO tags (id, name, color, description, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![t.id, t.name, t.color, t.description, t.created_at],
            )?;
        }

        // --- Reports ---
        for r in &project.reports {
            let config_json = r
                .config
                .as_ref()
                .and_then(|c| serde_json::to_string(c).ok());
            conn.execute(
                "INSERT OR IGNORE INTO reports (id, title, report_type, format, output_path, generated_at, generated_by, status, error, config)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    r.id, r.title, r.report_type, r.format, r.output_path,
                    r.generated_at, r.generated_by, r.status, r.error, config_json,
                ],
            )?;
        }

        // --- Saved Searches ---
        for s in &project.saved_searches {
            conn.execute(
                "INSERT OR IGNORE INTO saved_searches (id, name, query, search_type, is_regex, case_sensitive, scope, created_at, use_count, last_used)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    s.id, s.name, s.query, s.search_type,
                    s.is_regex as i32, s.case_sensitive as i32,
                    s.scope, s.created_at, s.use_count, s.last_used,
                ],
            )?;
        }

        // --- Evidence Files from cache ---
        let mut evidence_file_ids_by_path = HashMap::new();
        if let Some(ref cache) = project.evidence_cache {
            for f in &cache.discovered_files {
                let id = f.path.clone();
                evidence_file_ids_by_path.insert(f.path.clone(), id.clone());
                conn.execute(
                    "INSERT OR IGNORE INTO evidence_files (id, path, filename, container_type, total_size, segment_count, discovered_at, created, modified)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                    params![
                        id, f.path, f.filename, f.container_type, f.size as i64,
                        f.segment_count as i32,
                        cache.cached_at.clone(),
                        f.created, f.modified,
                    ],
                )?;
            }
        }

        migrate_cached_hashes(
            &conn,
            project,
            &mut evidence_file_ids_by_path,
            &migration_time,
        )?;

        // --- Case Documents from cache ---
        if let Some(ref cache) = project.case_documents_cache {
            let discovered_at = non_empty_or(&cache.cached_at, &migration_time);
            for d in &cache.documents {
                let id = case_document_id(&d.document_type, &d.path);
                let document_type = non_empty_or(&d.document_type, "unknown");
                let format = non_empty_or(&d.format, "unknown");
                conn.execute(
                    "INSERT OR IGNORE INTO case_documents (id, path, filename, document_type, size, format, case_number, evidence_id, modified, discovered_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                    params![
                        id,
                        d.path,
                        d.filename,
                        document_type,
                        d.size as i64,
                        format,
                        d.case_number,
                        d.evidence_id,
                        d.modified,
                        discovered_at,
                    ],
                )?;
            }
        }

        // Record migration timestamp
        conn.execute(
            "INSERT OR REPLACE INTO schema_meta (key, value) VALUES ('migrated_from_cffx', ?1)",
            params![&migration_time],
        )?;

        // --- Processed Databases ---
        let now_str = migration_time.clone();
        let pd_state = &project.processed_databases;

        // Register each loaded processed database path
        for (idx, loaded_path) in pd_state.loaded_paths.iter().enumerate() {
            let db_id = format!("pdb_{}", idx);
            let display_name = std::path::Path::new(loaded_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_string();

            // Try to extract metadata from cached_metadata if available
            let (db_type, case_number, examiner, total_size, artifact_count, metadata_json) =
                if let Some(ref meta_map) = pd_state.cached_metadata {
                    if let Some(meta_val) = meta_map.get(loaded_path) {
                        let db_type = meta_val
                            .get("db_type")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown")
                            .to_string();
                        let case_number = meta_val
                            .get("case_number")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        let examiner = meta_val
                            .get("examiner")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        let total_size = meta_val
                            .get("total_size")
                            .and_then(|v| v.as_i64())
                            .unwrap_or(0);
                        let artifact_count =
                            meta_val.get("artifact_count").and_then(|v| v.as_i64());
                        let metadata_json = serde_json::to_string(meta_val).ok();
                        (
                            db_type,
                            case_number,
                            examiner,
                            total_size,
                            artifact_count,
                            metadata_json,
                        )
                    } else {
                        ("Unknown".to_string(), None, None, 0i64, None, None)
                    }
                } else {
                    ("Unknown".to_string(), None, None, 0i64, None, None)
                };

            conn.execute(
                "INSERT OR IGNORE INTO processed_databases (id, path, name, db_type, case_number, examiner, total_size, artifact_count, registered_at, metadata_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    db_id, loaded_path, display_name, db_type,
                    case_number, examiner, total_size, artifact_count,
                    now_str, metadata_json,
                ],
            )?;

            // Migrate integrity records for this path
            if let Some(integrity) = pd_state.integrity.get(loaded_path) {
                let integrity_id = format!("pdi_{}_{}", idx, 0);
                let changes_json = if integrity.changes.is_empty() {
                    None
                } else {
                    serde_json::to_string(&integrity.changes).ok()
                };
                conn.execute(
                    "INSERT OR IGNORE INTO processed_db_integrity (id, processed_db_id, file_path, file_size, baseline_hash, baseline_timestamp, current_hash, current_hash_timestamp, status, changes_json)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                    params![
                        integrity_id, db_id, &integrity.path,
                        integrity.file_size as i64, &integrity.baseline_hash,
                        &integrity.baseline_timestamp, &integrity.current_hash,
                        &integrity.current_hash_timestamp, &integrity.status, changes_json,
                    ],
                )?;

                // Migrate work metrics if present
                if let Some(ref metrics) = integrity.metrics {
                    let metrics_id = format!("pdm_{}", idx);
                    let user_names_json = serde_json::to_string(&metrics.user_names).ok();
                    conn.execute(
                        "INSERT OR IGNORE INTO processed_db_metrics (id, processed_db_id, total_scans, last_scan_date, total_jobs, last_job_date, total_notes, total_tagged_items, total_users, user_names_json, captured_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                        params![
                            metrics_id, db_id,
                            metrics.total_scans as i32, &metrics.last_scan_date,
                            metrics.total_jobs as i32, &metrics.last_job_date,
                            metrics.total_notes as i32, metrics.total_tagged_items as i32,
                            metrics.total_users as i32, user_names_json, &now_str,
                        ],
                    )?;
                }
            }

            // Migrate cached AXIOM case info
            if let Some(ref axiom_map) = pd_state.cached_axiom_case_info {
                if let Some(axiom_val) = axiom_map.get(loaded_path) {
                    let axiom_id = format!("axc_{}", idx);
                    let case_name = axiom_val
                        .get("case_name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown")
                        .to_string();
                    let case_number = axiom_val
                        .get("case_number")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let case_type = axiom_val
                        .get("case_type")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let description = axiom_val
                        .get("description")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let examiner = axiom_val
                        .get("examiner")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let agency = axiom_val
                        .get("agency")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let axiom_version = axiom_val
                        .get("axiom_version")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let search_start = axiom_val
                        .get("search_start")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let search_end = axiom_val
                        .get("search_end")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let search_duration = axiom_val
                        .get("search_duration")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let search_outcome = axiom_val
                        .get("search_outcome")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let output_folder = axiom_val
                        .get("output_folder")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let total_artifacts = axiom_val
                        .get("total_artifacts")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);
                    let case_path = axiom_val
                        .get("case_path")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let keyword_info_json = axiom_val.get("keyword_info").map(|v| v.to_string());

                    conn.execute(
                        "INSERT OR IGNORE INTO axiom_case_info (id, processed_db_id, case_name, case_number, case_type, description, examiner, agency, axiom_version, search_start, search_end, search_duration, search_outcome, output_folder, total_artifacts, case_path, captured_at, keyword_info_json)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
                        params![
                            axiom_id, db_id, case_name, case_number,
                            case_type, description, examiner, agency,
                            axiom_version, search_start, search_end,
                            search_duration, search_outcome, output_folder,
                            total_artifacts, case_path, now_str, keyword_info_json,
                        ],
                    )?;

                    // Migrate AXIOM evidence sources
                    if let Some(sources) =
                        axiom_val.get("evidence_sources").and_then(|v| v.as_array())
                    {
                        for (si, source) in sources.iter().enumerate() {
                            let src_id = format!("axs_{}_{}", idx, si);
                            let name = source
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let evidence_number = source
                                .get("evidence_number")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());
                            let source_type = source
                                .get("source_type")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown")
                                .to_string();
                            let path = source
                                .get("path")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());
                            let hash = source
                                .get("hash")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());
                            let size = source.get("size").and_then(|v| v.as_i64());
                            let acquired = source
                                .get("acquired")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());
                            let search_types_json =
                                source.get("search_types").map(|v| v.to_string());

                            conn.execute(
                                "INSERT OR IGNORE INTO axiom_evidence_sources (id, axiom_case_id, name, evidence_number, source_type, path, hash, size, acquired, search_types_json)
                                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                                params![src_id, axiom_id, name, evidence_number, source_type, path, hash, size, acquired, search_types_json],
                            )?;
                        }
                    }

                    // Migrate AXIOM search results
                    if let Some(results) =
                        axiom_val.get("search_results").and_then(|v| v.as_array())
                    {
                        for (ri, result) in results.iter().enumerate() {
                            let res_id = format!("axr_{}_{}", idx, ri);
                            let artifact_type = result
                                .get("artifact_type")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let hit_count = result
                                .get("hit_count")
                                .and_then(|v| v.as_i64())
                                .unwrap_or(0);

                            conn.execute(
                                "INSERT OR IGNORE INTO axiom_search_results (id, axiom_case_id, artifact_type, hit_count)
                                 VALUES (?1, ?2, ?3, ?4)",
                                params![res_id, axiom_id, artifact_type, hit_count],
                            )?;
                        }
                    }
                }
            }

            // Migrate cached artifact categories
            if let Some(ref cat_map) = pd_state.cached_artifact_categories {
                if let Some(cats) = cat_map.get(loaded_path) {
                    for (ci, cat) in cats.iter().enumerate() {
                        let cat_id = format!("cat_{}_{}", idx, ci);
                        let category = cat
                            .get("category")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let artifact_type = cat
                            .get("artifact_type")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let count = cat.get("count").and_then(|v| v.as_i64()).unwrap_or(0);

                        conn.execute(
                            "INSERT OR IGNORE INTO artifact_categories (id, processed_db_id, category, artifact_type, count)
                             VALUES (?1, ?2, ?3, ?4, ?5)",
                            params![cat_id, db_id, category, artifact_type, count],
                        )?;
                    }
                }
            }
        }

        info!(
            "Migration complete for project '{}' (including {} processed databases)",
            project.name,
            pd_state.loaded_paths.len()
        );
        Ok(())
    }
}

fn migrate_cached_hashes(
    conn: &rusqlite::Connection,
    project: &crate::project::FFXProject,
    evidence_file_ids_by_path: &mut HashMap<String, String>,
    migration_time: &str,
) -> SqlResult<()> {
    let mut seen = HashSet::new();

    if let Some(ref cache) = project.evidence_cache {
        for (file_path, hash) in &cache.computed_hashes {
            migrate_hash_record(
                conn,
                evidence_file_ids_by_path,
                &mut seen,
                file_path,
                &hash.algorithm,
                &hash.hash,
                non_empty_or(&hash.computed_at, migration_time),
                None,
                migration_time,
            )?;
        }
    }

    for (file_path, hashes) in &project.hash_history.files {
        for hash in hashes {
            migrate_hash_record(
                conn,
                evidence_file_ids_by_path,
                &mut seen,
                file_path,
                &hash.algorithm,
                &hash.hash_value,
                non_empty_or(&hash.computed_at, migration_time),
                hash.verification.as_ref(),
                migration_time,
            )?;
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn migrate_hash_record(
    conn: &rusqlite::Connection,
    evidence_file_ids_by_path: &mut HashMap<String, String>,
    seen: &mut HashSet<String>,
    file_path: &str,
    algorithm: &str,
    hash_value: &str,
    computed_at: &str,
    verification: Option<&crate::project::ProjectVerification>,
    migration_time: &str,
) -> SqlResult<()> {
    let algorithm = match HashAlgorithm::from_str(algorithm) {
        Ok(algorithm) => algorithm,
        Err(err) => {
            warn!(
                path = file_path,
                algorithm, "Skipping cached project hash with unsupported algorithm: {}", err
            );
            return Ok(());
        }
    };
    let canonical_algorithm = algorithm.name();
    let hash_value = hash_value.trim();
    if !is_valid_hash(hash_value, algorithm) {
        warn!(
            path = file_path,
            algorithm = canonical_algorithm,
            "Skipping cached project hash with invalid digest length or characters"
        );
        return Ok(());
    }

    let dedupe_key = format!("{file_path}\0{canonical_algorithm}\0{hash_value}");
    if !seen.insert(dedupe_key.clone()) {
        return Ok(());
    }

    let file_id = ensure_evidence_file(conn, evidence_file_ids_by_path, file_path, migration_time)?;
    let source_ref_json = serde_json::json!({
        "kind": "localFile",
        "path": file_path,
    })
    .to_string();
    let hash_id = stable_id("cached_hash", &dedupe_key);

    conn.execute(
        "INSERT OR IGNORE INTO hashes (id, file_id, source_id, source_ref_json, algorithm, hash_value, computed_at, segment_index, segment_name, source)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            hash_id,
            file_id,
            file_path,
            source_ref_json,
            canonical_algorithm,
            hash_value,
            computed_at,
            None::<i32>,
            None::<String>,
            "cached",
        ],
    )?;

    if let Some(verification) = verification {
        let expected_hash = verification.verified_against.trim();
        if is_valid_hash(expected_hash, algorithm) {
            conn.execute(
                "INSERT OR IGNORE INTO verifications (id, hash_id, verified_at, result, expected_hash, actual_hash)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    stable_id("cached_verification", &format!("{hash_id}\0{}", verification.verified_at)),
                    hash_id,
                    non_empty_or(&verification.verified_at, computed_at),
                    non_empty_or(&verification.result, "match"),
                    expected_hash,
                    hash_value,
                ],
            )?;
        } else {
            warn!(
                path = file_path,
                algorithm = canonical_algorithm,
                "Skipping cached project hash verification with invalid expected digest"
            );
        }
    }

    Ok(())
}

fn ensure_evidence_file(
    conn: &rusqlite::Connection,
    evidence_file_ids_by_path: &mut HashMap<String, String>,
    file_path: &str,
    migration_time: &str,
) -> SqlResult<String> {
    if let Some(id) = evidence_file_ids_by_path.get(file_path) {
        return Ok(id.clone());
    }

    let id = file_path.to_string();
    let filename = Path::new(file_path)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or(file_path);
    conn.execute(
        "INSERT OR IGNORE INTO evidence_files (id, path, filename, container_type, total_size, segment_count, discovered_at, created, modified)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            id,
            file_path,
            filename,
            "File",
            0i64,
            1i32,
            migration_time,
            None::<String>,
            None::<String>,
        ],
    )?;
    evidence_file_ids_by_path.insert(file_path.to_string(), id.clone());
    Ok(id)
}

fn case_document_id(document_type: &str, path: &str) -> String {
    let mut id: String = format!("{document_type}-{path}")
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .take(128)
        .collect();
    if id.trim_matches('_').is_empty() {
        id = stable_id("case_doc", path);
    }
    id
}

fn stable_id(prefix: &str, source: &str) -> String {
    let digest = Sha256::digest(source.as_bytes());
    format!("{prefix}_{digest:x}")
}

fn non_empty_or<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    if value.trim().is_empty() {
        fallback
    } else {
        value
    }
}
