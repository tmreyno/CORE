// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Path utilities and evidence discovery commands for Project Setup Wizard.

use std::path::PathBuf;

use tauri::Emitter;
use tracing::{debug, info, instrument};

use crate::containers;
#[cfg(feature = "flavor-review")]
use crate::processed;

// =============================================================================
// Filesystem directory listing (host filesystem)
// =============================================================================

/// A single entry returned by `list_directory`.
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<i64>,
}

/// List the contents of a host-filesystem directory.
/// Returns files and subdirectories (non-recursive, single level).
#[tauri::command]
pub fn list_directory(path: String) -> Result<Vec<DirEntry>, String> {
    let dir = PathBuf::from(&path);
    if !dir.exists() {
        return Err(format!("Path does not exist: {}", dir.display()));
    }
    if !dir.is_dir() {
        return Err(format!("Path is not a directory: {}", dir.display()));
    }

    let read_dir = std::fs::read_dir(&dir)
        .map_err(|e| format!("Cannot read directory {}: {e}", dir.display()))?;

    let mut entries = Vec::new();
    for entry in read_dir {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let name = entry.file_name().to_string_lossy().to_string();
        // Skip hidden files (Unix dotfiles)
        if name.starts_with('.') {
            continue;
        }
        let meta = entry.metadata();
        let (is_dir, size, modified) = match &meta {
            Ok(m) => {
                let mod_time = m.modified().ok().and_then(|t| {
                    t.duration_since(std::time::UNIX_EPOCH)
                        .ok()
                        .map(|d| d.as_secs() as i64)
                });
                (m.is_dir(), m.len(), mod_time)
            }
            Err(_) => (false, 0, None),
        };
        entries.push(DirEntry {
            name,
            path: entry.path().to_string_lossy().to_string(),
            is_dir,
            size,
            modified,
        });
    }

    // Sort: directories first, then alphabetically
    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Ok(entries)
}

/// Check if a path exists (file or directory)
#[tauri::command]
pub fn path_exists(path: String) -> Result<bool, String> {
    let path = std::path::PathBuf::from(&path);
    Ok(path.exists())
}

/// Check if a path is a directory
#[tauri::command]
pub fn path_is_directory(path: String) -> Result<bool, String> {
    let path = std::path::PathBuf::from(&path);
    Ok(path.is_dir())
}

/// Discover evidence files (E01, AD1, L01, etc.) in a directory
/// Returns just the file paths for quick discovery
#[tauri::command]
pub fn discover_evidence_files(
    #[allow(non_snake_case)] dirPath: String,
    recursive: bool,
) -> Result<Vec<String>, String> {
    let path = std::path::PathBuf::from(&dirPath);

    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()));
    }

    if !path.is_dir() {
        return Err(format!("Path is not a directory: {}", path.display()));
    }

    let files = if recursive {
        containers::scan_directory_recursive(&dirPath)?
    } else {
        containers::scan_directory(&dirPath)?
    };

    Ok(files.into_iter().map(|f| f.path).collect())
}

/// Scan for processed databases (AXIOM, Cellebrite, etc.) and return them
/// Returns ProcessedDbInfo directly (can be converted to ProcessedDatabase in frontend)
#[cfg(feature = "flavor-review")]
#[tauri::command]
pub fn scan_for_processed_databases(
    #[allow(non_snake_case)] dirPath: String,
) -> Result<Vec<processed::types::ProcessedDbInfo>, String> {
    use std::path::PathBuf;

    let path = PathBuf::from(&dirPath);

    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()));
    }

    // Use the processed database scanner
    let dbs = processed::detection::scan_for_processed_dbs(&path, true);

    Ok(dbs)
}

#[tauri::command]
pub fn scan_directory(
    #[allow(non_snake_case)] dirPath: String,
) -> Result<Vec<containers::DiscoveredFile>, String> {
    containers::scan_directory(&dirPath).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn scan_directory_recursive(
    #[allow(non_snake_case)] dirPath: String,
) -> Result<Vec<containers::DiscoveredFile>, String> {
    containers::scan_directory_recursive(&dirPath).map_err(|e| e.to_string())
}

#[tauri::command]
#[instrument(skip(window), fields(path = %dirPath, recursive))]
pub async fn scan_directory_streaming(
    window: tauri::Window,
    #[allow(non_snake_case)] dirPath: String,
    recursive: bool,
) -> Result<usize, String> {
    use tokio::sync::mpsc;

    info!("Starting directory scan");
    let (tx, mut rx) = mpsc::unbounded_channel::<containers::DiscoveredFile>();

    // Spawn blocking directory scan in background thread
    let dir_path_clone = dirPath.clone();
    let scan_handle = tauri::async_runtime::spawn_blocking(move || {
        containers::scan_directory_streaming(&dir_path_clone, recursive, |file| {
            let _ = tx.send(file.clone());
        })
    });

    // Stream results to frontend as they arrive
    let mut emitted = 0usize;
    while let Some(file) = rx.recv().await {
        debug!(file = %file.filename, "Found file");
        let _ = window.emit("scan-file-found", &file);
        emitted += 1;
    }

    // Wait for scan to complete and return count
    let result = scan_handle.await.map_err(|e| format!("Task failed: {e}"))?;
    info!(count = emitted, "Scan complete");
    result.map_err(|e| e.to_string())
}

// =============================================================================
// Case Document Discovery Commands
// =============================================================================

/// Find case documents (COC forms, intake forms, notes, etc.) in a directory
#[tauri::command]
pub fn find_case_documents(
    #[allow(non_snake_case)] dirPath: String,
    recursive: bool,
) -> Result<Vec<containers::CaseDocument>, String> {
    let config = containers::CaseDocumentSearchConfig {
        recursive,
        document_types: vec![],
        max_depth: if recursive { 5 } else { 0 },
        preview_only: true, // Default to preview mode for speed
    };

    Ok(containers::find_case_documents(&dirPath, &config))
}

/// Find Chain of Custody (COC) forms specifically
#[tauri::command]
pub fn find_coc_forms(
    #[allow(non_snake_case)] dirPath: String,
    recursive: bool,
) -> Result<Vec<containers::CaseDocument>, String> {
    Ok(containers::find_coc_forms(&dirPath, recursive))
}

/// Find case document folders relative to an evidence path
///
/// Searches parent directories for folders like "4.Case.Documents",
/// "Case Documents", "Paperwork", etc.
#[tauri::command]
pub fn find_case_document_folders(
    #[allow(non_snake_case)] evidencePath: String,
) -> Result<Vec<String>, String> {
    let folders = containers::find_case_document_folders(&evidencePath);
    Ok(folders
        .into_iter()
        .filter_map(|p| p.to_str().map(|s| s.to_string()))
        .collect())
}

/// Search for case documents across the entire case folder structure
///
/// Given an evidence path, this finds the case root and searches all
/// typical case document locations.
#[tauri::command]
pub fn discover_case_documents(
    #[allow(non_snake_case)] evidencePath: String,
    #[allow(non_snake_case)] previewOnly: Option<bool>,
) -> Result<Vec<containers::CaseDocument>, String> {
    let preview_only = previewOnly.unwrap_or(true); // Default to preview mode for speed
    info!(
        "discover_case_documents called with path: {}, preview_only: {}",
        evidencePath, preview_only
    );

    let mut all_documents = Vec::new();

    // First, find all case document folders by pattern matching
    let doc_folders = containers::find_case_document_folders(&evidencePath);
    info!(
        "Found {} case document folders by pattern",
        doc_folders.len()
    );
    for folder in &doc_folders {
        info!("  - {:?}", folder);
    }

    // Search each folder for documents
    let config = containers::CaseDocumentSearchConfig {
        recursive: true,
        document_types: vec![],
        max_depth: 3,
        preview_only,
    };

    for folder in &doc_folders {
        if let Some(path_str) = folder.to_str() {
            let docs = containers::find_case_documents(path_str, &config);
            info!("Found {} documents in {:?}", docs.len(), folder);
            all_documents.extend(docs);
        }
    }

    // FALLBACK: If no specific case document folders found, search parent directories directly
    if doc_folders.is_empty() {
        info!("No specific case doc folders found, searching parent directories...");
        let path = std::path::Path::new(&evidencePath);

        // Get the starting directory
        let start_dir = if path.is_file() {
            path.parent()
        } else {
            Some(path)
        };

        if let Some(start) = start_dir {
            // Search current directory and up to 3 parents
            let mut current = start.to_path_buf();
            for level in 0..4 {
                info!("Fallback search level {}: {:?}", level, current);
                if let Some(path_str) = current.to_str() {
                    // Use non-recursive search at each level to avoid going too deep
                    let shallow_config = containers::CaseDocumentSearchConfig {
                        recursive: false,
                        document_types: vec![],
                        max_depth: 0,
                        preview_only,
                    };
                    let docs = containers::find_case_documents(path_str, &shallow_config);
                    info!("Fallback found {} documents at level {}", docs.len(), level);
                    all_documents.extend(docs);
                }

                // Move up one directory
                if let Some(parent) = current.parent() {
                    current = parent.to_path_buf();
                } else {
                    break;
                }
            }
        }
    }

    // Remove duplicates by path
    all_documents.sort_by(|a, b| a.path.cmp(&b.path));
    all_documents.dedup_by(|a, b| a.path == b.path);

    info!("Returning {} total case documents", all_documents.len());
    Ok(all_documents)
}

// =============================================================================
// Project Folder Template Commands
// =============================================================================

/// A folder entry in the project template
#[derive(Clone, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TemplateFolderEntry {
    pub path: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub children: Vec<TemplateFolderEntry>,
}

/// A project folder template definition
#[derive(Clone, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProjectFolderTemplate {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    pub folders: Vec<TemplateFolderEntry>,
    #[serde(default)]
    pub role_mapping: Option<std::collections::HashMap<String, String>>,
}

/// Result of folder creation from a template
#[derive(Clone, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateFoldersResult {
    /// Total folders created
    pub created_count: usize,
    /// Total folders that already existed (skipped)
    pub existing_count: usize,
    /// Mapping of role → absolute path, for auto-populating project locations
    pub role_paths: std::collections::HashMap<String, String>,
    /// All folders that were created or already existed
    pub all_paths: Vec<String>,
}

/// Create a project folder structure from a JSON template definition.
///
/// The `template_json` parameter is the full JSON content of the template.
/// The `root_path` is the target directory where folders will be created.
/// The optional `case_name` replaces `{case_name}` placeholders in folder names.
///
/// Returns a `CreateFoldersResult` with counts and role→path mapping.
#[tauri::command]
pub fn create_folders_from_template(
    template_json: String,
    root_path: String,
    case_name: Option<String>,
) -> Result<CreateFoldersResult, String> {
    let template: ProjectFolderTemplate =
        serde_json::from_str(&template_json).map_err(|e| format!("Invalid template JSON: {e}"))?;

    let root = PathBuf::from(&root_path);
    if !root.exists() {
        std::fs::create_dir_all(&root)
            .map_err(|e| format!("Failed to create root directory {}: {e}", root.display()))?;
        info!("Created project root: {}", root.display());
    }
    if !root.is_dir() {
        return Err(format!("Root path is not a directory: {}", root.display()));
    }

    let mut created_count = 0usize;
    let mut existing_count = 0usize;
    let mut role_paths = std::collections::HashMap::new();
    let mut all_paths = Vec::new();

    // Recursive helper to create folders
    fn create_entries(
        entries: &[TemplateFolderEntry],
        parent: &std::path::Path,
        case_name: &Option<String>,
        created: &mut usize,
        existing: &mut usize,
        roles: &mut std::collections::HashMap<String, String>,
        all: &mut Vec<String>,
    ) -> Result<(), String> {
        for entry in entries {
            // Replace {case_name} placeholder if present
            let folder_name = if let Some(ref cn) = case_name {
                entry.path.replace("{case_name}", cn)
            } else {
                entry.path.clone()
            };

            let full_path = parent.join(&folder_name);

            if full_path.exists() {
                *existing += 1;
                debug!("Folder already exists: {}", full_path.display());
            } else {
                std::fs::create_dir_all(&full_path)
                    .map_err(|e| format!("Failed to create {}: {e}", full_path.display()))?;
                *created += 1;
                info!("Created folder: {}", full_path.display());
            }

            let abs_path = full_path.to_string_lossy().to_string();
            all.push(abs_path.clone());

            // Track role → path mapping
            if let Some(ref role) = entry.role {
                roles.insert(role.clone(), abs_path);
            }

            // Recursively create children
            if !entry.children.is_empty() {
                create_entries(
                    &entry.children,
                    &full_path,
                    case_name,
                    created,
                    existing,
                    roles,
                    all,
                )?;
            }
        }
        Ok(())
    }

    create_entries(
        &template.folders,
        &root,
        &case_name,
        &mut created_count,
        &mut existing_count,
        &mut role_paths,
        &mut all_paths,
    )?;

    info!(
        "Template '{}' applied to {}: {} created, {} existing",
        template.name,
        root.display(),
        created_count,
        existing_count
    );

    Ok(CreateFoldersResult {
        created_count,
        existing_count,
        role_paths,
        all_paths,
    })
}
