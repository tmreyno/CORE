// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! One-time migration from .cffx FFXProject into .ffxdb database.

use super::database::ProjectDatabase;
use rusqlite::{params, Result as SqlResult};
use tracing::info;

impl ProjectDatabase {
    // ========================================================================
    // Migration from FFXProject (one-time import from .cffx)
    // ========================================================================

    /// Migrate data from an FFXProject struct into this database.
    /// Used when opening a project that has data in the .cffx but no .ffxdb yet.
    /// This is idempotent — it uses INSERT OR IGNORE to avoid duplicates.
    pub fn migrate_from_project(&self, project: &crate::project::FFXProject) -> SqlResult<()> {
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
        if let Some(ref cache) = project.evidence_cache {
            for f in &cache.discovered_files {
                let id = format!(
                    "ev_{}",
                    f.path
                        .chars()
                        .filter(|c| c.is_alphanumeric())
                        .take(16)
                        .collect::<String>()
                );
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

        // Record migration timestamp
        conn.execute(
            "INSERT OR REPLACE INTO schema_meta (key, value) VALUES ('migrated_from_cffx', ?1)",
            params![chrono::Utc::now().to_rfc3339()],
        )?;

        // --- Processed Databases ---
        let now_str = chrono::Utc::now().to_rfc3339();
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
                        let db_type = meta_val.get("db_type").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string();
                        let case_number = meta_val.get("case_number").and_then(|v| v.as_str()).map(|s| s.to_string());
                        let examiner = meta_val.get("examiner").and_then(|v| v.as_str()).map(|s| s.to_string());
                        let total_size = meta_val.get("total_size").and_then(|v| v.as_i64()).unwrap_or(0);
                        let artifact_count = meta_val.get("artifact_count").and_then(|v| v.as_i64());
                        let metadata_json = serde_json::to_string(meta_val).ok();
                        (db_type, case_number, examiner, total_size, artifact_count, metadata_json)
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
                    let case_name = axiom_val.get("case_name").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string();
                    let case_number = axiom_val.get("case_number").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let case_type = axiom_val.get("case_type").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let description = axiom_val.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let examiner = axiom_val.get("examiner").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let agency = axiom_val.get("agency").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let axiom_version = axiom_val.get("axiom_version").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let search_start = axiom_val.get("search_start").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let search_end = axiom_val.get("search_end").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let search_duration = axiom_val.get("search_duration").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let search_outcome = axiom_val.get("search_outcome").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let output_folder = axiom_val.get("output_folder").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let total_artifacts = axiom_val.get("total_artifacts").and_then(|v| v.as_i64()).unwrap_or(0);
                    let case_path = axiom_val.get("case_path").and_then(|v| v.as_str()).map(|s| s.to_string());
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
                    if let Some(sources) = axiom_val.get("evidence_sources").and_then(|v| v.as_array()) {
                        for (si, source) in sources.iter().enumerate() {
                            let src_id = format!("axs_{}_{}", idx, si);
                            let name = source.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            let evidence_number = source.get("evidence_number").and_then(|v| v.as_str()).map(|s| s.to_string());
                            let source_type = source.get("source_type").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                            let path = source.get("path").and_then(|v| v.as_str()).map(|s| s.to_string());
                            let hash = source.get("hash").and_then(|v| v.as_str()).map(|s| s.to_string());
                            let size = source.get("size").and_then(|v| v.as_i64());
                            let acquired = source.get("acquired").and_then(|v| v.as_str()).map(|s| s.to_string());
                            let search_types_json = source.get("search_types").map(|v| v.to_string());

                            conn.execute(
                                "INSERT OR IGNORE INTO axiom_evidence_sources (id, axiom_case_id, name, evidence_number, source_type, path, hash, size, acquired, search_types_json)
                                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                                params![src_id, axiom_id, name, evidence_number, source_type, path, hash, size, acquired, search_types_json],
                            )?;
                        }
                    }

                    // Migrate AXIOM search results
                    if let Some(results) = axiom_val.get("search_results").and_then(|v| v.as_array()) {
                        for (ri, result) in results.iter().enumerate() {
                            let res_id = format!("axr_{}_{}", idx, ri);
                            let artifact_type = result.get("artifact_type").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            let hit_count = result.get("hit_count").and_then(|v| v.as_i64()).unwrap_or(0);

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
                        let category = cat.get("category").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let artifact_type = cat.get("artifact_type").and_then(|v| v.as_str()).unwrap_or("").to_string();
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

        info!("Migration complete for project '{}' (including {} processed databases)", project.name, pd_state.loaded_paths.len());
        Ok(())
    }
}
