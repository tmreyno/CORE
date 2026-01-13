// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Report API - Tauri command wrappers for report generation
 * 
 * This module provides TypeScript interfaces for the Rust report commands
 */

import { invoke } from "@tauri-apps/api/core";
import type { ContainerInfo, DiscoveredFile } from "../types";

// =============================================================================
// Input Types (sent to backend)
// =============================================================================

export interface StoredHashInput {
  algorithm: string;
  hash: string;
  verified?: boolean;
}

export interface ContainerInfoInput {
  container_type: string;
  path: string;
  filename: string;
  size: number;
  // EWF fields
  case_number?: string;
  evidence_number?: string;
  examiner_name?: string;
  description?: string;
  notes?: string;
  acquiry_date?: string;
  model?: string;
  serial_number?: string;
  total_size?: number;
  // Hash info
  stored_hashes?: StoredHashInput[];
  computed_hash?: StoredHashInput;
}

// =============================================================================
// Output Types (received from backend)
// =============================================================================

export interface HashRecord {
  item: string;
  algorithm: string;
  value: string;
  computed_at?: string;
  verified?: boolean;
}

export interface ImageInfo {
  format: string;
  file_names: string[];
  total_size: number;
  segments?: number;
  compression?: string;
  acquisition_tool?: string;
  acquisition_date?: string;
}

export interface EvidenceItem {
  evidence_id: string;
  description: string;
  evidence_type: string;
  make?: string;
  model?: string;
  serial_number?: string;
  capacity?: string;
  condition?: string;
  received_date?: string;
  submitted_by?: string;
  acquisition_hashes: HashRecord[];
  image_info?: ImageInfo;
  notes?: string;
}

// =============================================================================
// Conversion Helpers
// =============================================================================

/**
 * Convert frontend container info to backend input format
 */
export function containerToInput(
  file: DiscoveredFile,
  info: ContainerInfo | undefined,
  hashInfo: { algorithm: string; hash: string; verified?: boolean | null } | undefined
): ContainerInfoInput {
  // Extract data from various container types
  const ewfInfo = info?.e01 || info?.l01;
  const ad1Info = info?.ad1;
  const ufedInfo = info?.ufed;
  
  // Get stored hashes from container
  const storedHashes: StoredHashInput[] = [];
  
  if (ewfInfo?.stored_hashes) {
    for (const h of ewfInfo.stored_hashes) {
      storedHashes.push({
        algorithm: h.algorithm,
        hash: h.hash,
        verified: h.verified ?? undefined,
      });
    }
  }
  
  if (ad1Info?.companion_log) {
    const log = ad1Info.companion_log;
    if (log.md5_hash) {
      storedHashes.push({ algorithm: "MD5", hash: log.md5_hash });
    }
    if (log.sha1_hash) {
      storedHashes.push({ algorithm: "SHA1", hash: log.sha1_hash });
    }
  }
  
  return {
    container_type: file.container_type,
    path: file.path,
    filename: file.filename,
    size: file.size,
    case_number: ewfInfo?.case_number ?? ad1Info?.companion_log?.case_number ?? ufedInfo?.case_info?.case_identifier ?? undefined,
    evidence_number: ewfInfo?.evidence_number ?? ad1Info?.companion_log?.evidence_number ?? ufedInfo?.evidence_number ?? undefined,
    examiner_name: ewfInfo?.examiner_name ?? ad1Info?.companion_log?.examiner ?? ufedInfo?.case_info?.examiner_name ?? undefined,
    description: ewfInfo?.description ?? undefined,
    notes: ewfInfo?.notes ?? ad1Info?.companion_log?.notes ?? undefined,
    acquiry_date: ewfInfo?.acquiry_date ?? ad1Info?.companion_log?.acquisition_date ?? ufedInfo?.extraction_info?.start_time ?? undefined,
    model: ewfInfo?.model ?? ufedInfo?.device_info?.model ?? undefined,
    serial_number: ewfInfo?.serial_number ?? ufedInfo?.device_info?.serial_number ?? undefined,
    total_size: ewfInfo?.total_size ?? ad1Info?.total_size ?? ufedInfo?.size ?? undefined,
    stored_hashes: storedHashes.length > 0 ? storedHashes : undefined,
    computed_hash: hashInfo ? {
      algorithm: hashInfo.algorithm,
      hash: hashInfo.hash,
      verified: hashInfo.verified ?? undefined,
    } : undefined,
  };
}

/**
 * Convert multiple containers to input format
 */
export function containersToInputs(
  files: DiscoveredFile[],
  fileInfoMap: Map<string, ContainerInfo>,
  fileHashMap: Map<string, { algorithm: string; hash: string; verified?: boolean | null }>
): ContainerInfoInput[] {
  return files.map(file => containerToInput(
    file,
    fileInfoMap.get(file.path),
    fileHashMap.get(file.path)
  ));
}

// =============================================================================
// API Commands
// =============================================================================

/**
 * Extract evidence items from container info
 * 
 * This sends container data to the backend for conversion to properly
 * formatted evidence items suitable for reports.
 */
export async function extractEvidenceFromContainers(
  containers: ContainerInfoInput[]
): Promise<EvidenceItem[]> {
  return invoke<EvidenceItem[]>("extract_evidence_from_containers", { containers });
}

/**
 * Create a single evidence item from container info
 */
export async function createEvidenceFromContainer(
  container: ContainerInfoInput,
  evidenceId: string
): Promise<EvidenceItem> {
  return invoke<EvidenceItem>("create_evidence_from_container", { container, evidenceId });
}

/**
 * Generate evidence items from current app state
 * 
 * Convenience function that converts file data and calls the backend.
 */
export async function generateEvidenceFromFiles(
  files: DiscoveredFile[],
  fileInfoMap: Map<string, ContainerInfo>,
  fileHashMap: Map<string, { algorithm: string; hash: string; verified?: boolean | null }>
): Promise<EvidenceItem[]> {
  const containers = containersToInputs(files, fileInfoMap, fileHashMap);
  return extractEvidenceFromContainers(containers);
}

/**
 * Get a report template for a specific investigation type
 */
export async function getReportTemplate(investigationType: string): Promise<unknown> {
  return invoke("get_report_template", { investigationType });
}

/**
 * Check if AI assistant is available
 */
export async function isAiAvailable(): Promise<boolean> {
  return invoke<boolean>("is_ai_available");
}

// =============================================================================
// AI Assistant Types
// =============================================================================

export interface AiProviderInfo {
  id: string;
  name: string;
  description: string;
  requires_api_key: boolean;
  default_model: string;
  available_models: string[];
}

export type NarrativeType = 
  | "executive_summary"
  | "finding"
  | "timeline"
  | "evidence"
  | "methodology"
  | "conclusion";

// =============================================================================
// AI Assistant Commands
// =============================================================================

/**
 * Get available AI providers
 */
export async function getAiProviders(): Promise<AiProviderInfo[]> {
  return invoke<AiProviderInfo[]>("get_ai_providers");
}

/**
 * Check if Ollama is running locally
 */
export async function checkOllamaConnection(): Promise<boolean> {
  return invoke<boolean>("check_ollama_connection");
}

/**
 * Generate AI narrative for a report section
 */
export async function generateAiNarrative(
  context: string,
  narrativeType: NarrativeType,
  provider: string,
  model: string,
  apiKey?: string
): Promise<string> {
  return invoke<string>("generate_ai_narrative", {
    context,
    narrativeType,
    provider,
    model,
    apiKey: apiKey ?? null,
  });
}

/**
 * Build context string from evidence items for AI
 */
export function buildEvidenceContext(evidenceItems: EvidenceItem[]): string {
  const lines: string[] = [];
  
  lines.push("=== EVIDENCE ITEMS ===\n");
  
  for (const item of evidenceItems) {
    lines.push(`Evidence ID: ${item.evidence_id}`);
    lines.push(`Description: ${item.description}`);
    lines.push(`Type: ${item.evidence_type}`);
    
    if (item.model) lines.push(`Model: ${item.model}`);
    if (item.serial_number) lines.push(`Serial Number: ${item.serial_number}`);
    if (item.capacity) lines.push(`Capacity: ${item.capacity}`);
    
    if (item.image_info) {
      lines.push(`Image Format: ${item.image_info.format}`);
      lines.push(`Total Size: ${formatBytes(item.image_info.total_size)}`);
      if (item.image_info.acquisition_tool) {
        lines.push(`Acquisition Tool: ${item.image_info.acquisition_tool}`);
      }
    }
    
    if (item.acquisition_hashes.length > 0) {
      lines.push("Hash Values:");
      for (const hash of item.acquisition_hashes) {
        const verified = hash.verified ? " (verified)" : "";
        lines.push(`  ${hash.algorithm}: ${hash.value}${verified}`);
      }
    }
    
    if (item.notes) lines.push(`Notes: ${item.notes}`);
    
    lines.push(""); // Blank line between items
  }
  
  return lines.join("\n");
}

/**
 * Format bytes to human-readable string
 */
function formatBytes(bytes: number): string {
  const units = ["B", "KB", "MB", "GB", "TB"];
  let value = bytes;
  let unitIndex = 0;
  
  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex++;
  }
  
  return unitIndex === 0 
    ? `${bytes} ${units[0]}`
    : `${value.toFixed(2)} ${units[unitIndex]}`;
}

/**
 * Export report to JSON
 */
export async function exportReportJson(report: unknown): Promise<string> {
  return invoke<string>("export_report_json", { report });
}

/**
 * Import report from JSON
 */
export async function importReportJson(json: string): Promise<unknown> {
  return invoke("import_report_json", { json });
}
