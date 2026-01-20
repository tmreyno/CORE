// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Discovery API - Tauri command wrappers for evidence and document discovery
 * 
 * This module provides TypeScript interfaces for the Rust discovery commands,
 * including case document and Chain of Custody (COC) form discovery.
 */

import { invoke } from "@tauri-apps/api/core";
import type { CaseDocument, DiscoveredFile } from "./types";

// =============================================================================
// Case Document Discovery
// =============================================================================

/**
 * Find case documents (COC forms, intake forms, notes, etc.) in a directory
 * 
 * @param dirPath - Directory path to search
 * @param recursive - Whether to search subdirectories
 * @returns Array of discovered case documents
 */
export async function findCaseDocuments(
  dirPath: string,
  recursive: boolean = true
): Promise<CaseDocument[]> {
  return invoke<CaseDocument[]>("find_case_documents", { dirPath, recursive });
}

/**
 * Find Chain of Custody (COC) forms specifically
 * 
 * @param dirPath - Directory path to search
 * @param recursive - Whether to search subdirectories
 * @returns Array of discovered COC forms
 */
export async function findCocForms(
  dirPath: string,
  recursive: boolean = true
): Promise<CaseDocument[]> {
  return invoke<CaseDocument[]>("find_coc_forms", { dirPath, recursive });
}

/**
 * Find case document folders relative to an evidence path
 * 
 * Searches parent directories for folders like "4.Case.Documents",
 * "Case Documents", "Paperwork", etc.
 * 
 * @param evidencePath - Path to an evidence file or directory
 * @returns Array of folder paths that may contain case documents
 */
export async function findCaseDocumentFolders(
  evidencePath: string
): Promise<string[]> {
  return invoke<string[]>("find_case_document_folders", { evidencePath });
}

/**
 * Search for case documents across the entire case folder structure
 * 
 * Given an evidence path, this finds the case root and searches all
 * typical case document locations.
 * 
 * @param evidencePath - Path to an evidence file
 * @param previewOnly - If true, skip content-based detection for faster results (default: true)
 * @returns Array of discovered case documents
 */
export async function discoverCaseDocuments(
  evidencePath: string,
  previewOnly: boolean = true
): Promise<CaseDocument[]> {
  return invoke<CaseDocument[]>("discover_case_documents", { evidencePath, previewOnly });
}

// =============================================================================
// Evidence Discovery
// =============================================================================

/**
 * Scan a directory for forensic evidence containers
 * 
 * @param dirPath - Directory to scan
 * @returns Array of discovered evidence files
 */
export async function scanDirectory(
  dirPath: string
): Promise<DiscoveredFile[]> {
  return invoke<DiscoveredFile[]>("scan_directory", { dirPath });
}

/**
 * Scan a directory recursively for forensic evidence containers
 * 
 * @param dirPath - Directory to scan
 * @returns Array of discovered evidence files
 */
export async function scanDirectoryRecursive(
  dirPath: string
): Promise<DiscoveredFile[]> {
  return invoke<DiscoveredFile[]>("scan_directory_recursive", { dirPath });
}

/**
 * Scan a directory with streaming results (emits events)
 * 
 * @param dirPath - Directory to scan
 * @param recursive - Whether to scan subdirectories
 * @returns Total count of files found
 */
export async function scanDirectoryStreaming(
  dirPath: string,
  recursive: boolean = true
): Promise<number> {
  return invoke<number>("scan_directory_streaming", { dirPath, recursive });
}

/**
 * Check if a path exists
 */
export async function pathExists(path: string): Promise<boolean> {
  return invoke<boolean>("path_exists", { path });
}

/**
 * Check if a path is a directory
 */
export async function pathIsDirectory(path: string): Promise<boolean> {
  return invoke<boolean>("path_is_directory", { path });
}

// =============================================================================
// Helper Functions
// =============================================================================

/**
 * Get human-readable document type label
 */
export function getDocumentTypeLabel(type: CaseDocument["document_type"]): string {
  switch (type) {
    case "ChainOfCustody":
      return "Chain of Custody";
    case "EvidenceIntake":
      return "Evidence Intake";
    case "CaseNotes":
      return "Case Notes";
    case "EvidenceReceipt":
      return "Evidence Receipt";
    case "LabRequest":
      return "Lab Request";
    case "ExternalReport":
      return "External Report";
    case "Other":
    default:
      return "Other Document";
  }
}

/**
 * Get document type icon (for UI display)
 */
export function getDocumentTypeIcon(type: CaseDocument["document_type"]): string {
  switch (type) {
    case "ChainOfCustody":
      return "📋"; // Clipboard
    case "EvidenceIntake":
      return "📝"; // Memo
    case "CaseNotes":
      return "📓"; // Notebook
    case "EvidenceReceipt":
      return "🧾"; // Receipt
    case "LabRequest":
      return "🔬"; // Microscope
    case "ExternalReport":
      return "📄"; // Document
    case "Other":
    default:
      return "📁"; // Folder
  }
}
