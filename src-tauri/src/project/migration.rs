// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Path portability (relative/absolute conversion) and project version migration.

use super::types::*;
use super::FFXProject;
use std::collections::HashMap;
use std::path::Path;
use tracing::info;

// =============================================================================
// Path Portability - Relative/Absolute Path Conversion
// =============================================================================

/// Convert an absolute path to a relative path based on the project file location.
/// Returns the relative path if possible, otherwise returns the original path.
pub(crate) fn to_relative_path(absolute_path: &str, project_dir: &Path) -> String {
    let path = Path::new(absolute_path);
    
    // Try to make path relative to project directory
    if let Ok(relative) = path.strip_prefix(project_dir) {
        // Use forward slashes for cross-platform compatibility
        let relative_str = relative.to_string_lossy().replace('\\', "/");
        if relative_str.is_empty() {
            ".".to_string()
        } else {
            format!("./{}", relative_str)
        }
    } else if let Some(parent) = project_dir.parent() {
        // Try relative to parent directory (for sibling folders)
        if let Ok(relative) = path.strip_prefix(parent) {
            let relative_str = relative.to_string_lossy().replace('\\', "/");
            format!("./{}", relative_str)
        } else {
            // Can't make relative, keep absolute
            absolute_path.to_string()
        }
    } else {
        // Can't make relative, keep absolute
        absolute_path.to_string()
    }
}

/// Convert a relative path to an absolute path based on the project file location.
/// If the path is already absolute, returns it unchanged.
pub(crate) fn to_absolute_path(relative_path: &str, project_dir: &Path) -> String {
    let path = Path::new(relative_path);
    
    // If already absolute, return as-is
    if path.is_absolute() {
        return relative_path.to_string();
    }
    
    // Handle relative paths starting with "./" or without prefix
    let normalized = if let Some(stripped) = relative_path.strip_prefix("./") {
        stripped
    } else if relative_path.starts_with("../") {
        relative_path
    } else if relative_path == "." {
        ""
    } else {
        relative_path
    };
    
    // Resolve relative to project directory
    let resolved = if normalized.is_empty() {
        project_dir.to_path_buf()
    } else {
        project_dir.join(normalized)
    };
    
    // Canonicalize if the path exists, otherwise just normalize
    match resolved.canonicalize() {
        Ok(canonical) => canonical.to_string_lossy().to_string(),
        Err(_) => {
            // Path doesn't exist yet or can't be resolved - just join and return
            resolved.to_string_lossy().to_string()
        }
    }
}

// =============================================================================
// Make Paths Relative (for Save)
// =============================================================================

/// Convert all paths in a project to relative paths for portability.
/// This is called before saving.
pub(crate) fn make_paths_relative(project: &mut FFXProject, project_dir: &Path) {
    // Main paths
    project.root_path = to_relative_path(&project.root_path, project_dir);
    
    // Locations
    if let Some(ref mut locations) = project.locations {
        locations.project_root = to_relative_path(&locations.project_root, project_dir);
        locations.evidence_path = to_relative_path(&locations.evidence_path, project_dir);
        locations.processed_db_path = to_relative_path(&locations.processed_db_path, project_dir);
        if let Some(ref path) = locations.case_documents_path {
            locations.case_documents_path = Some(to_relative_path(path, project_dir));
        }
    }
    
    // UI State - case documents path
    if let Some(ref path) = project.ui_state.case_documents_path {
        project.ui_state.case_documents_path = Some(to_relative_path(path, project_dir));
    }
    
    // Tabs - convert file paths
    for tab in &mut project.tabs {
        tab.file_path = to_relative_path(&tab.file_path, project_dir);
        if let Some(ref path) = tab.document_path {
            tab.document_path = Some(to_relative_path(path, project_dir));
        }
        if let Some(ref path) = tab.entry_container_path {
            tab.entry_container_path = Some(to_relative_path(path, project_dir));
        }
        if let Some(ref path) = tab.processed_db_path {
            tab.processed_db_path = Some(to_relative_path(path, project_dir));
        }
    }
    
    // Hash history - convert keys (file paths)
    let mut new_hash_history = HashMap::new();
    for (path, hashes) in project.hash_history.files.drain() {
        let relative_path = to_relative_path(&path, project_dir);
        new_hash_history.insert(relative_path, hashes);
    }
    project.hash_history.files = new_hash_history;
    
    // Evidence cache
    if let Some(ref mut cache) = project.evidence_cache {
        for file in &mut cache.discovered_files {
            file.path = to_relative_path(&file.path, project_dir);
        }
        // file_info keys
        let mut new_file_info = HashMap::new();
        for (path, info) in cache.file_info.drain() {
            let relative_path = to_relative_path(&path, project_dir);
            new_file_info.insert(relative_path, info);
        }
        cache.file_info = new_file_info;
        // computed_hashes keys
        let mut new_computed_hashes = HashMap::new();
        for (path, hash) in cache.computed_hashes.drain() {
            let relative_path = to_relative_path(&path, project_dir);
            new_computed_hashes.insert(relative_path, hash);
        }
        cache.computed_hashes = new_computed_hashes;
    }
    
    // Case documents cache
    if let Some(ref mut cache) = project.case_documents_cache {
        cache.search_path = to_relative_path(&cache.search_path, project_dir);
        for doc in &mut cache.documents {
            doc.path = to_relative_path(&doc.path, project_dir);
        }
    }
    
    // Processed databases
    project.processed_databases.loaded_paths = project.processed_databases.loaded_paths
        .iter()
        .map(|p| to_relative_path(p, project_dir))
        .collect();
    if let Some(ref path) = project.processed_databases.selected_path {
        project.processed_databases.selected_path = Some(to_relative_path(path, project_dir));
    }
    // cached_metadata keys
    if let Some(ref mut metadata) = project.processed_databases.cached_metadata {
        let mut new_metadata = HashMap::new();
        for (path, info) in metadata.drain() {
            let relative_path = to_relative_path(&path, project_dir);
            new_metadata.insert(relative_path, info);
        }
        *metadata = new_metadata;
    }
    // cached_axiom_case_info keys
    if let Some(ref mut axiom_info) = project.processed_databases.cached_axiom_case_info {
        let mut new_axiom_info = HashMap::new();
        for (path, info) in axiom_info.drain() {
            let relative_path = to_relative_path(&path, project_dir);
            new_axiom_info.insert(relative_path, info);
        }
        *axiom_info = new_axiom_info;
    }
    // cached_artifact_categories keys
    if let Some(ref mut categories) = project.processed_databases.cached_artifact_categories {
        let mut new_categories = HashMap::new();
        for (path, cats) in categories.drain() {
            let relative_path = to_relative_path(&path, project_dir);
            new_categories.insert(relative_path, cats);
        }
        *categories = new_categories;
    }
    
    // cached_databases - convert path fields in JSON values
    if let Some(ref mut databases) = project.processed_databases.cached_databases {
        for db in databases.iter_mut() {
            if let Some(obj) = db.as_object_mut() {
                // Convert main path field
                if let Some(serde_json::Value::String(path)) = obj.get("path") {
                    let relative = to_relative_path(path, project_dir);
                    obj.insert("path".to_string(), serde_json::Value::String(relative));
                }
                // Convert database_files paths
                if let Some(serde_json::Value::Array(files)) = obj.get_mut("database_files") {
                    for file in files.iter_mut() {
                        if let Some(file_obj) = file.as_object_mut() {
                            if let Some(serde_json::Value::String(path)) = file_obj.get("path") {
                                let relative = to_relative_path(path, project_dir);
                                file_obj.insert("path".to_string(), serde_json::Value::String(relative));
                            }
                        }
                    }
                }
            }
        }
    }
    
    // cached_axiom_case_info - convert case_path field in JSON values
    if let Some(ref mut axiom_info) = project.processed_databases.cached_axiom_case_info {
        for info in axiom_info.values_mut() {
            if let Some(obj) = info.as_object_mut() {
                if let Some(serde_json::Value::String(path)) = obj.get("case_path") {
                    let relative = to_relative_path(path, project_dir);
                    obj.insert("case_path".to_string(), serde_json::Value::String(relative));
                }
            }
        }
    }
    
    // integrity keys
    let mut new_integrity = HashMap::new();
    for (path, info) in project.processed_databases.integrity.drain() {
        let relative_path = to_relative_path(&path, project_dir);
        let mut new_info = info;
        new_info.path = to_relative_path(&new_info.path, project_dir);
        new_integrity.insert(relative_path, new_info);
    }
    project.processed_databases.integrity = new_integrity;
    
    // Activity log - convert file_path fields
    for entry in &mut project.activity_log {
        if let Some(ref path) = entry.file_path {
            entry.file_path = Some(to_relative_path(path, project_dir));
        }
    }
    
    // Open directories
    for dir in &mut project.open_directories {
        dir.path = to_relative_path(&dir.path, project_dir);
    }
    
    // Recent directories
    for dir in &mut project.recent_directories {
        dir.path = to_relative_path(&dir.path, project_dir);
    }
    
    // File selection
    project.file_selection.selected_paths = project.file_selection.selected_paths
        .iter()
        .map(|p| to_relative_path(p, project_dir))
        .collect();
    if let Some(ref path) = project.file_selection.active_path {
        project.file_selection.active_path = Some(to_relative_path(path, project_dir));
    }
    
    // Bookmarks
    for bookmark in &mut project.bookmarks {
        bookmark.target_path = to_relative_path(&bookmark.target_path, project_dir);
    }
    
    // Notes
    for note in &mut project.notes {
        if let Some(ref path) = note.target_path {
            note.target_path = Some(to_relative_path(path, project_dir));
        }
    }
    
    // Reports
    for report in &mut project.reports {
        if let Some(ref path) = report.output_path {
            report.output_path = Some(to_relative_path(path, project_dir));
        }
    }
    
    // Preview cache
    if let Some(ref mut cache) = project.preview_cache {
        if let Some(ref path) = cache.cache_dir {
            cache.cache_dir = Some(to_relative_path(path, project_dir));
        }
        for entry in &mut cache.entries {
            entry.container_path = to_relative_path(&entry.container_path, project_dir);
            entry.temp_path = to_relative_path(&entry.temp_path, project_dir);
        }
    }
    
    // UI state - selected entry
    if let Some(ref mut entry) = project.ui_state.selected_entry {
        entry.container_path = to_relative_path(&entry.container_path, project_dir);
    }
    
    // Tree expansion state paths
    if let Some(ref mut tree_state) = project.ui_state.tree_expansion_state {
        tree_state.containers = tree_state.containers.iter().map(|p| to_relative_path(p, project_dir)).collect();
        tree_state.vfs = tree_state.vfs.iter().map(|p| to_relative_path(p, project_dir)).collect();
        tree_state.archive = tree_state.archive.iter().map(|p| to_relative_path(p, project_dir)).collect();
        tree_state.lazy = tree_state.lazy.iter().map(|p| to_relative_path(p, project_dir)).collect();
        tree_state.ad1 = tree_state.ad1.iter().map(|p| to_relative_path(p, project_dir)).collect();
    }
    
    // Tree state
    for node in &mut project.ui_state.tree_state {
        convert_tree_node_paths_relative(node, project_dir);
    }
    
    // Scroll positions keys
    let mut new_scroll_positions = HashMap::new();
    for (path, pos) in project.ui_state.scroll_positions.drain() {
        let relative_path = to_relative_path(&path, project_dir);
        new_scroll_positions.insert(relative_path, pos);
    }
    project.ui_state.scroll_positions = new_scroll_positions;
}

fn convert_tree_node_paths_relative(node: &mut TreeNodeState, project_dir: &Path) {
    node.path = to_relative_path(&node.path, project_dir);
    for child in &mut node.children {
        convert_tree_node_paths_relative(child, project_dir);
    }
}

// =============================================================================
// Make Paths Absolute (for Load)
// =============================================================================

/// Convert all relative paths in a project to absolute paths.
/// This is called after loading.
pub(crate) fn make_paths_absolute(project: &mut FFXProject, project_dir: &Path) {
    // Main paths
    project.root_path = to_absolute_path(&project.root_path, project_dir);
    
    // Locations
    if let Some(ref mut locations) = project.locations {
        locations.project_root = to_absolute_path(&locations.project_root, project_dir);
        locations.evidence_path = to_absolute_path(&locations.evidence_path, project_dir);
        locations.processed_db_path = to_absolute_path(&locations.processed_db_path, project_dir);
        if let Some(ref path) = locations.case_documents_path {
            locations.case_documents_path = Some(to_absolute_path(path, project_dir));
        }
    }
    
    // UI State - case documents path
    if let Some(ref path) = project.ui_state.case_documents_path {
        project.ui_state.case_documents_path = Some(to_absolute_path(path, project_dir));
    }
    
    // Tabs - convert file paths
    for tab in &mut project.tabs {
        tab.file_path = to_absolute_path(&tab.file_path, project_dir);
        if let Some(ref path) = tab.document_path {
            tab.document_path = Some(to_absolute_path(path, project_dir));
        }
        if let Some(ref path) = tab.entry_container_path {
            tab.entry_container_path = Some(to_absolute_path(path, project_dir));
        }
        if let Some(ref path) = tab.processed_db_path {
            tab.processed_db_path = Some(to_absolute_path(path, project_dir));
        }
    }
    
    // Hash history - convert keys (file paths)
    let mut new_hash_history = HashMap::new();
    for (path, hashes) in project.hash_history.files.drain() {
        let absolute_path = to_absolute_path(&path, project_dir);
        new_hash_history.insert(absolute_path, hashes);
    }
    project.hash_history.files = new_hash_history;
    
    // Evidence cache
    if let Some(ref mut cache) = project.evidence_cache {
        for file in &mut cache.discovered_files {
            file.path = to_absolute_path(&file.path, project_dir);
        }
        // file_info keys
        let mut new_file_info = HashMap::new();
        for (path, info) in cache.file_info.drain() {
            let absolute_path = to_absolute_path(&path, project_dir);
            new_file_info.insert(absolute_path, info);
        }
        cache.file_info = new_file_info;
        // computed_hashes keys
        let mut new_computed_hashes = HashMap::new();
        for (path, hash) in cache.computed_hashes.drain() {
            let absolute_path = to_absolute_path(&path, project_dir);
            new_computed_hashes.insert(absolute_path, hash);
        }
        cache.computed_hashes = new_computed_hashes;
    }
    
    // Case documents cache
    if let Some(ref mut cache) = project.case_documents_cache {
        cache.search_path = to_absolute_path(&cache.search_path, project_dir);
        for doc in &mut cache.documents {
            doc.path = to_absolute_path(&doc.path, project_dir);
        }
    }
    
    // Processed databases
    project.processed_databases.loaded_paths = project.processed_databases.loaded_paths
        .iter()
        .map(|p| to_absolute_path(p, project_dir))
        .collect();
    if let Some(ref path) = project.processed_databases.selected_path {
        project.processed_databases.selected_path = Some(to_absolute_path(path, project_dir));
    }
    // cached_metadata keys
    if let Some(ref mut metadata) = project.processed_databases.cached_metadata {
        let mut new_metadata = HashMap::new();
        for (path, info) in metadata.drain() {
            let absolute_path = to_absolute_path(&path, project_dir);
            new_metadata.insert(absolute_path, info);
        }
        *metadata = new_metadata;
    }
    // cached_axiom_case_info keys
    if let Some(ref mut axiom_info) = project.processed_databases.cached_axiom_case_info {
        let mut new_axiom_info = HashMap::new();
        for (path, info) in axiom_info.drain() {
            let absolute_path = to_absolute_path(&path, project_dir);
            new_axiom_info.insert(absolute_path, info);
        }
        *axiom_info = new_axiom_info;
    }
    // cached_artifact_categories keys
    if let Some(ref mut categories) = project.processed_databases.cached_artifact_categories {
        let mut new_categories = HashMap::new();
        for (path, cats) in categories.drain() {
            let absolute_path = to_absolute_path(&path, project_dir);
            new_categories.insert(absolute_path, cats);
        }
        *categories = new_categories;
    }
    
    // cached_databases - convert path fields in JSON values
    if let Some(ref mut databases) = project.processed_databases.cached_databases {
        for db in databases.iter_mut() {
            if let Some(obj) = db.as_object_mut() {
                // Convert main path field
                if let Some(serde_json::Value::String(path)) = obj.get("path") {
                    let absolute = to_absolute_path(path, project_dir);
                    obj.insert("path".to_string(), serde_json::Value::String(absolute));
                }
                // Convert database_files paths
                if let Some(serde_json::Value::Array(files)) = obj.get_mut("database_files") {
                    for file in files.iter_mut() {
                        if let Some(file_obj) = file.as_object_mut() {
                            if let Some(serde_json::Value::String(path)) = file_obj.get("path") {
                                let absolute = to_absolute_path(path, project_dir);
                                file_obj.insert("path".to_string(), serde_json::Value::String(absolute));
                            }
                        }
                    }
                }
            }
        }
    }
    
    // cached_axiom_case_info - convert case_path field in JSON values
    if let Some(ref mut axiom_info) = project.processed_databases.cached_axiom_case_info {
        for info in axiom_info.values_mut() {
            if let Some(obj) = info.as_object_mut() {
                if let Some(serde_json::Value::String(path)) = obj.get("case_path") {
                    let absolute = to_absolute_path(path, project_dir);
                    obj.insert("case_path".to_string(), serde_json::Value::String(absolute));
                }
            }
        }
    }
    
    // integrity keys
    let mut new_integrity = HashMap::new();
    for (path, info) in project.processed_databases.integrity.drain() {
        let absolute_path = to_absolute_path(&path, project_dir);
        let mut new_info = info;
        new_info.path = to_absolute_path(&new_info.path, project_dir);
        new_integrity.insert(absolute_path, new_info);
    }
    project.processed_databases.integrity = new_integrity;
    
    // Activity log - convert file_path fields
    for entry in &mut project.activity_log {
        if let Some(ref path) = entry.file_path {
            entry.file_path = Some(to_absolute_path(path, project_dir));
        }
    }
    
    // Open directories
    for dir in &mut project.open_directories {
        dir.path = to_absolute_path(&dir.path, project_dir);
    }
    
    // Recent directories
    for dir in &mut project.recent_directories {
        dir.path = to_absolute_path(&dir.path, project_dir);
    }
    
    // File selection
    project.file_selection.selected_paths = project.file_selection.selected_paths
        .iter()
        .map(|p| to_absolute_path(p, project_dir))
        .collect();
    if let Some(ref path) = project.file_selection.active_path {
        project.file_selection.active_path = Some(to_absolute_path(path, project_dir));
    }
    
    // Bookmarks
    for bookmark in &mut project.bookmarks {
        bookmark.target_path = to_absolute_path(&bookmark.target_path, project_dir);
    }
    
    // Notes
    for note in &mut project.notes {
        if let Some(ref path) = note.target_path {
            note.target_path = Some(to_absolute_path(path, project_dir));
        }
    }
    
    // Reports
    for report in &mut project.reports {
        if let Some(ref path) = report.output_path {
            report.output_path = Some(to_absolute_path(path, project_dir));
        }
    }
    
    // Preview cache
    if let Some(ref mut cache) = project.preview_cache {
        if let Some(ref path) = cache.cache_dir {
            cache.cache_dir = Some(to_absolute_path(path, project_dir));
        }
        for entry in &mut cache.entries {
            entry.container_path = to_absolute_path(&entry.container_path, project_dir);
            entry.temp_path = to_absolute_path(&entry.temp_path, project_dir);
        }
    }
    
    // UI state - selected entry
    if let Some(ref mut entry) = project.ui_state.selected_entry {
        entry.container_path = to_absolute_path(&entry.container_path, project_dir);
    }
    
    // Tree expansion state paths
    if let Some(ref mut tree_state) = project.ui_state.tree_expansion_state {
        tree_state.containers = tree_state.containers.iter().map(|p| to_absolute_path(p, project_dir)).collect();
        tree_state.vfs = tree_state.vfs.iter().map(|p| to_absolute_path(p, project_dir)).collect();
        tree_state.archive = tree_state.archive.iter().map(|p| to_absolute_path(p, project_dir)).collect();
        tree_state.lazy = tree_state.lazy.iter().map(|p| to_absolute_path(p, project_dir)).collect();
        tree_state.ad1 = tree_state.ad1.iter().map(|p| to_absolute_path(p, project_dir)).collect();
    }
    
    // Tree state
    for node in &mut project.ui_state.tree_state {
        convert_tree_node_paths_absolute(node, project_dir);
    }
    
    // Scroll positions keys
    let mut new_scroll_positions = HashMap::new();
    for (path, pos) in project.ui_state.scroll_positions.drain() {
        let absolute_path = to_absolute_path(&path, project_dir);
        new_scroll_positions.insert(absolute_path, pos);
    }
    project.ui_state.scroll_positions = new_scroll_positions;
}

fn convert_tree_node_paths_absolute(node: &mut TreeNodeState, project_dir: &Path) {
    node.path = to_absolute_path(&node.path, project_dir);
    for child in &mut node.children {
        convert_tree_node_paths_absolute(child, project_dir);
    }
}

// =============================================================================
// Version Migration
// =============================================================================

/// Migrate a project from an older version to the current version
pub(crate) fn migrate_project(project: &mut FFXProject) {
    let old_version = project.version;
    
    // Migration from v1 to v2
    if old_version < 2 {
        info!("Applying v1 -> v2 migration");
        
        // Ensure all tabs have IDs
        for (i, tab) in project.tabs.iter_mut().enumerate() {
            if tab.id.is_empty() || tab.id.starts_with("tab_") {
                tab.id = format!("evidence:{}", tab.file_path);
            }
            // Ensure tab type is set
            if tab.tab_type.is_empty() {
                tab.tab_type = "evidence".to_string();
            }
            // Ensure order is set
            tab.order = i as u32;
        }
        
        // Initialize new caches if not present
        if project.evidence_cache.is_none() {
            project.evidence_cache = Some(EvidenceCache::default());
        }
        if project.case_documents_cache.is_none() {
            project.case_documents_cache = Some(CaseDocumentsCache::default());
        }
        if project.preview_cache.is_none() {
            project.preview_cache = Some(PreviewCache::default());
        }
        if project.center_pane_state.is_none() {
            project.center_pane_state = Some(CenterPaneState::default());
        }
    }
    
    // Future migrations would go here:
    // if old_version < 3 { ... }
}
