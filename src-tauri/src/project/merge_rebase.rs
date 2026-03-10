// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Path rebasing for project merge — relocates all paths from old_base to new_base.

use super::types::*;
use super::FFXProject;
use std::collections::HashMap;
use std::path::Path;
use tracing::info;

/// Rebase all paths in a project from old_base to new_base.
/// Used when relocating a project to a new directory.
pub fn rebase_paths(project: &mut FFXProject, old_base: &Path, new_base: &Path) {
    info!("Rebasing paths: {:?} → {:?}", old_base, new_base);

    let rebase = |path: &str| -> String {
        let p = Path::new(path);
        if let Ok(relative) = p.strip_prefix(old_base) {
            new_base.join(relative).to_string_lossy().to_string()
        } else {
            // Path doesn't start with old_base, leave as-is
            path.to_string()
        }
    };

    // Root path
    project.root_path = rebase(&project.root_path);

    // Locations
    if let Some(ref mut loc) = project.locations {
        loc.project_root = rebase(&loc.project_root);
        loc.evidence_path = rebase(&loc.evidence_path);
        loc.processed_db_path = rebase(&loc.processed_db_path);
        if let Some(ref path) = loc.case_documents_path {
            loc.case_documents_path = Some(rebase(path));
        }
    }

    // Tabs
    for tab in &mut project.tabs {
        tab.file_path = rebase(&tab.file_path);
        if let Some(ref path) = tab.document_path {
            tab.document_path = Some(rebase(path));
        }
        if let Some(ref path) = tab.entry_container_path {
            tab.entry_container_path = Some(rebase(path));
        }
        if let Some(ref path) = tab.processed_db_path {
            tab.processed_db_path = Some(rebase(path));
        }
    }

    // Hash history
    let mut new_hh: HashMap<String, Vec<ProjectFileHash>> = HashMap::new();
    for (path, hashes) in project.hash_history.files.drain() {
        new_hh.insert(rebase(&path), hashes);
    }
    project.hash_history.files = new_hh;

    // Evidence cache
    if let Some(ref mut cache) = project.evidence_cache {
        for f in &mut cache.discovered_files {
            f.path = rebase(&f.path);
        }
        let mut new_fi: HashMap<String, serde_json::Value> = HashMap::new();
        for (path, info) in cache.file_info.drain() {
            new_fi.insert(rebase(&path), info);
        }
        cache.file_info = new_fi;

        let mut new_ch: HashMap<String, CachedFileHash> = HashMap::new();
        for (path, hash) in cache.computed_hashes.drain() {
            new_ch.insert(rebase(&path), hash);
        }
        cache.computed_hashes = new_ch;
    }

    // Case documents cache
    if let Some(ref mut cache) = project.case_documents_cache {
        cache.search_path = rebase(&cache.search_path);
        for doc in &mut cache.documents {
            doc.path = rebase(&doc.path);
        }
    }

    // Processed databases
    project.processed_databases.loaded_paths = project
        .processed_databases
        .loaded_paths
        .iter()
        .map(|p| rebase(p))
        .collect();
    if let Some(ref path) = project.processed_databases.selected_path {
        project.processed_databases.selected_path = Some(rebase(path));
    }
    if let Some(ref mut meta) = project.processed_databases.cached_metadata {
        let mut new: HashMap<String, serde_json::Value> = HashMap::new();
        for (k, v) in meta.drain() {
            new.insert(rebase(&k), v);
        }
        *meta = new;
    }
    // Rebase integrity keys
    let mut new_integrity: HashMap<String, ProcessedDbIntegrity> = HashMap::new();
    for (k, mut v) in project.processed_databases.integrity.drain() {
        v.path = rebase(&v.path);
        new_integrity.insert(rebase(&k), v);
    }
    project.processed_databases.integrity = new_integrity;

    // Activity log file_paths
    for entry in &mut project.activity_log {
        if let Some(ref path) = entry.file_path {
            entry.file_path = Some(rebase(path));
        }
    }

    // Open directories
    for dir in &mut project.open_directories {
        dir.path = rebase(&dir.path);
    }

    // Recent directories
    for dir in &mut project.recent_directories {
        dir.path = rebase(&dir.path);
    }

    // File selection
    project.file_selection.selected_paths = project
        .file_selection
        .selected_paths
        .iter()
        .map(|p| rebase(p))
        .collect();
    if let Some(ref path) = project.file_selection.active_path {
        project.file_selection.active_path = Some(rebase(path));
    }

    // Bookmarks
    for b in &mut project.bookmarks {
        b.target_path = rebase(&b.target_path);
    }

    // Notes
    for n in &mut project.notes {
        if let Some(ref path) = n.target_path {
            n.target_path = Some(rebase(path));
        }
    }

    // Reports
    for r in &mut project.reports {
        if let Some(ref path) = r.output_path {
            r.output_path = Some(rebase(path));
        }
    }

    // UI state - case documents path
    if let Some(ref path) = project.ui_state.case_documents_path {
        project.ui_state.case_documents_path = Some(rebase(path));
    }

    // Active tab path
    if let Some(ref path) = project.active_tab_path {
        project.active_tab_path = Some(rebase(path));
    }
}
