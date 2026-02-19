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
    parse_axiom_case, get_artifact_categories,
    AxiomCaseInfo, ArtifactCategorySummary
};
use super::cellebrite::{
    parse_cellebrite_case, get_cellebrite_categories,
    CellebriteCaseInfo, CellebriteArtifactCategory,
};
use super::autopsy::{
    parse_autopsy_case, get_autopsy_categories,
    AutopsyCaseInfo, AutopsyArtifactCategory,
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

// ============================================================================
// Cellebrite-specific commands
// ============================================================================

/// Get Cellebrite PA case information
#[command]
pub fn get_cellebrite_case_info(path: String) -> Result<CellebriteCaseInfo, String> {
    let path = PathBuf::from(&path);

    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()));
    }

    parse_cellebrite_case(&path).map_err(|e| e.to_string())
}

/// Get artifact categories from a Cellebrite PA database
#[command]
pub fn get_cellebrite_artifact_categories(
    path: String,
) -> Result<Vec<CellebriteArtifactCategory>, String> {
    let path = PathBuf::from(&path);

    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()));
    }

    get_cellebrite_categories(&path).map_err(|e| e.to_string())
}

// ============================================================================
// Autopsy-specific commands
// ============================================================================

/// Get Autopsy case information
#[command]
pub fn get_autopsy_case_info(path: String) -> Result<AutopsyCaseInfo, String> {
    let path = PathBuf::from(&path);

    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()));
    }

    parse_autopsy_case(&path).map_err(|e| e.to_string())
}

/// Get artifact categories from an Autopsy database
#[command]
pub fn get_autopsy_artifact_categories(
    path: String,
) -> Result<Vec<AutopsyArtifactCategory>, String> {
    let path = PathBuf::from(&path);

    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()));
    }

    get_autopsy_categories(&path).map_err(|e| e.to_string())
}
