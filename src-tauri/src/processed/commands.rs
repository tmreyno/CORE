// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tauri commands for processed database operations

use std::path::PathBuf;
use tauri::command;

use super::types::*;
use super::detection::*;
use super::axiom::{
    parse_axiom_case, get_artifact_categories, query_axiom_artifacts,
    list_axiom_tables, AxiomCaseInfo, AxiomArtifact, ArtifactCategorySummary
};

/// Scan a directory for processed databases
#[command]
pub fn scan_processed_databases(
    path: String,
    recursive: bool,
) -> Result<Vec<ProcessedDbInfo>, String> {
    let path = PathBuf::from(&path);
    
    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()));
    }
    
    Ok(scan_for_processed_dbs(&path, recursive))
}

/// Get info about a specific processed database
#[command]
pub fn get_processed_db_details(path: String) -> Result<ProcessedDbInfo, String> {
    let path = PathBuf::from(&path);
    
    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()));
    }
    
    let db_type = detect_processed_db(&path)
        .ok_or_else(|| "Not a recognized processed database format".to_string())?;
    
    get_processed_db_info(&path, db_type)
        .ok_or_else(|| "Failed to get database info".to_string())
}

/// Check if a path is a processed database
#[command]
pub fn is_processed_database(path: String) -> Result<Option<String>, String> {
    let path = PathBuf::from(&path);
    
    if !path.exists() {
        return Ok(None);
    }
    
    Ok(detect_processed_db(&path).map(|t| t.as_str().to_string()))
}

/// Get summary statistics for processed databases in a directory
#[command]
pub fn get_processed_db_summary(
    path: String,
    recursive: bool,
) -> Result<ProcessedDbSummary, String> {
    let path = PathBuf::from(&path);
    
    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()));
    }
    
    let dbs = scan_for_processed_dbs(&path, recursive);
    
    let mut summary = ProcessedDbSummary {
        total_count: dbs.len(),
        ..Default::default()
    };
    
    for db in &dbs {
        summary.total_size += db.total_size;
        *summary.by_type
            .entry(db.db_type.as_str().to_string())
            .or_insert(0) += 1;
    }
    
    Ok(summary)
}

// ============================================================================
// AXIOM-specific commands
// ============================================================================

/// Get AXIOM case information
#[command]
pub fn get_axiom_case_info(path: String) -> Result<AxiomCaseInfo, String> {
    let path = PathBuf::from(&path);
    
    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()));
    }
    
    parse_axiom_case(&path).map_err(|e| e.to_string())
}

/// Get artifact categories from an AXIOM database
#[command]
pub fn get_axiom_artifact_categories(path: String) -> Result<Vec<ArtifactCategorySummary>, String> {
    let path = PathBuf::from(&path);
    
    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()));
    }
    
    get_artifact_categories(&path).map_err(|e| e.to_string())
}

/// Query artifacts from an AXIOM database with pagination
#[command]
pub fn query_axiom_artifacts_cmd(
    path: String,
    artifact_type: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<Vec<AxiomArtifact>, String> {
    let path = PathBuf::from(&path);
    
    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()));
    }
    
    let limit = limit.unwrap_or(100);
    let offset = offset.unwrap_or(0);
    
    query_axiom_artifacts(&path, artifact_type.as_deref(), limit, offset).map_err(|e| e.to_string())
}

/// List all tables in an AXIOM database (for exploration/debugging)
#[command]
pub fn list_axiom_db_tables(path: String) -> Result<Vec<(String, u64)>, String> {
    let path = PathBuf::from(&path);
    
    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()));
    }
    
    list_axiom_tables(&path).map_err(|e| e.to_string())
}
