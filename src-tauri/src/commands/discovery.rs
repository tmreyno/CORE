// =============================================================================
// CORE-FFX - Discovery Commands
// =============================================================================

//! Path utilities and evidence discovery commands for Project Setup Wizard.

use tracing::{debug, info, instrument};
use tauri::Emitter;

use crate::containers;
use crate::processed;

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
    #[allow(non_snake_case)]
    dirPath: String,
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
#[tauri::command]
pub fn scan_for_processed_databases(
    #[allow(non_snake_case)]
    dirPath: String,
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
    #[allow(non_snake_case)]
    dirPath: String,
) -> Result<Vec<containers::DiscoveredFile>, String> {
    containers::scan_directory(&dirPath).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn scan_directory_recursive(
    #[allow(non_snake_case)]
    dirPath: String,
) -> Result<Vec<containers::DiscoveredFile>, String> {
    containers::scan_directory_recursive(&dirPath).map_err(|e| e.to_string())
}

#[tauri::command]
#[instrument(skip(window), fields(path = %dirPath, recursive))]
pub async fn scan_directory_streaming(
    window: tauri::Window,
    #[allow(non_snake_case)]
    dirPath: String,
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
    #[allow(non_snake_case)]
    dirPath: String,
    recursive: bool,
) -> Result<Vec<containers::CaseDocument>, String> {
    let config = containers::CaseDocumentSearchConfig {
        recursive,
        document_types: vec![],
        max_depth: if recursive { 5 } else { 0 },
    };
    
    Ok(containers::find_case_documents(&dirPath, &config))
}

/// Find Chain of Custody (COC) forms specifically
#[tauri::command]
pub fn find_coc_forms(
    #[allow(non_snake_case)]
    dirPath: String,
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
    #[allow(non_snake_case)]
    evidencePath: String,
) -> Result<Vec<String>, String> {
    let folders = containers::find_case_document_folders(&evidencePath);
    Ok(folders.into_iter()
        .filter_map(|p| p.to_str().map(|s| s.to_string()))
        .collect())
}

/// Search for case documents across the entire case folder structure
/// 
/// Given an evidence path, this finds the case root and searches all
/// typical case document locations.
#[tauri::command]
pub fn discover_case_documents(
    #[allow(non_snake_case)]
    evidencePath: String,
) -> Result<Vec<containers::CaseDocument>, String> {
    info!("discover_case_documents called with path: {}", evidencePath);
    
    let mut all_documents = Vec::new();
    
    // First, find all case document folders
    let doc_folders = containers::find_case_document_folders(&evidencePath);
    info!("Found {} case document folders", doc_folders.len());
    for folder in &doc_folders {
        info!("  - {:?}", folder);
    }
    
    // Search each folder for documents
    let config = containers::CaseDocumentSearchConfig {
        recursive: true,
        document_types: vec![],
        max_depth: 3,
    };
    
    for folder in doc_folders {
        if let Some(path_str) = folder.to_str() {
            let docs = containers::find_case_documents(path_str, &config);
            info!("Found {} documents in {:?}", docs.len(), folder);
            all_documents.extend(docs);
        }
    }
    
    // Remove duplicates by path
    all_documents.sort_by(|a, b| a.path.cmp(&b.path));
    all_documents.dedup_by(|a, b| a.path == b.path);
    
    info!("Returning {} total case documents", all_documents.len());
    Ok(all_documents)
}
