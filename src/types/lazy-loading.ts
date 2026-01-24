// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Lazy Loading Types (TypeScript)
 *
 * This module mirrors the Rust lazy_loading.rs definitions for use in the frontend.
 * It provides unified types for lazy loading across all container formats.
 *
 * @module lazy-loading
 */

// =============================================================================
// CONFIGURATION
// =============================================================================

/**
 * Configuration for lazy loading behavior
 * 
 * These settings can be adjusted through the app config to tune
 * performance based on system capabilities and user preferences.
 */
export interface LazyLoadConfig {
  /** Whether lazy loading is enabled (default: true) */
  enabled: boolean;
  
  /** Maximum number of entries to load per batch/level (default: 100) */
  batch_size: number;
  
  /** Threshold for auto-expanding directories (default: 50) */
  auto_expand_threshold: number;
  
  /** Entry count threshold for switching to lazy loading (default: 10,000) */
  large_container_threshold: number;
  
  /** Maximum entries to show before pagination (default: 500) */
  pagination_threshold: number;
  
  /** Whether to show entry count before loading (default: true) */
  show_entry_count: boolean;
  
  /** Timeout for entry count operations in milliseconds (default: 5000) */
  count_timeout_ms: number;
  
  /** Timeout for children loading operations in milliseconds (default: 30000) */
  load_timeout_ms: number;
}

/**
 * Default lazy loading configuration
 */
export const DEFAULT_LAZY_LOAD_CONFIG: LazyLoadConfig = {
  enabled: true,
  batch_size: 100,
  auto_expand_threshold: 50,
  large_container_threshold: 10_000,
  pagination_threshold: 500,
  show_entry_count: true,
  count_timeout_ms: 5_000,
  load_timeout_ms: 30_000,
};

// =============================================================================
// TREE ENTRY
// =============================================================================

/**
 * Unified tree entry for all container types
 * 
 * This provides a consistent interface for the UI regardless of whether
 * the source is AD1, E01, UFED, ZIP, or any other container format.
 */
export interface LazyTreeEntry {
  /** Unique identifier for this entry (format-specific) */
  id: string;
  
  /** Display name */
  name: string;
  
  /** Full path within the container */
  path: string;
  
  /** Whether this is a directory/folder */
  is_dir: boolean;
  
  /** File size in bytes (0 for directories) */
  size: number;
  
  /** Entry type (file, folder, ad1, e01, zip, etc.) */
  entry_type: string;
  
  /** Number of children (if known, -1 if unknown) */
  child_count: number;
  
  /** Whether children have been loaded */
  children_loaded: boolean;
  
  /** Hash value if available */
  hash: string | null;
  
  /** Last modified timestamp if available */
  modified: string | null;
  
  /** Container-specific metadata (JSON string) */
  metadata: string | null;
}

// =============================================================================
// LAZY LOAD RESULT
// =============================================================================

/**
 * Result of a lazy loading operation
 */
export interface LazyLoadResult {
  /** Loaded entries */
  entries: LazyTreeEntry[];
  
  /** Total number of entries at this level */
  total_count: number;
  
  /** Whether there are more entries to load */
  has_more: boolean;
  
  /** Offset for pagination (next page starts here) */
  next_offset: number;
  
  /** Whether lazy loading was used */
  lazy_loaded: boolean;
  
  /** Configuration that was applied */
  config: LazyLoadConfig;
}

// =============================================================================
// CONTAINER SUMMARY
// =============================================================================

/**
 * Summary information for lazy loading decisions
 * 
 * Call lazy_get_container_summary FIRST to determine if lazy loading
 * should be used for a container.
 */
export interface ContainerSummary {
  /** Path to the container file */
  path: string;
  
  /** Container type (ad1, e01, zip, etc.) */
  container_type: string;
  
  /** Total file size */
  total_size: number;
  
  /** Total entry count (files + directories) */
  entry_count: number;
  
  /** Root-level entry count */
  root_entry_count: number;
  
  /** Whether lazy loading is recommended */
  lazy_loading_recommended: boolean;
  
  /** Estimated time to load all entries (ms) */
  estimated_load_time_ms: number | null;
}

// =============================================================================
// SUPPORTED CONTAINER TYPES
// =============================================================================

/**
 * Container types supported by the unified lazy loading API
 */
export type SupportedContainerType = 
  | 'ad1'      // FTK Logical Images
  | 'e01'      // EnCase Physical
  | 'l01'      // EnCase Logical
  | 'ex01'     // EnCase Physical v2
  | 'lx01'     // EnCase Logical v2
  | 'ewf'      // Expert Witness Format
  | 'ufed'     // Cellebrite UFED
  | 'ufd'      // UFED folder
  | 'ufdr'     // UFED report
  | 'ufdx'     // UFED extraction
  | 'zip'      // ZIP archives
  | '7z'       // 7-Zip archives
  | 'rar'      // RAR archives
  | 'tar'      // TAR archives
  | 'raw'      // Raw disk images
  | 'unknown'; // Unknown type

/**
 * Check if a container type supports lazy loading
 */
export function supportsLazyLoading(containerType: string): boolean {
  const supported: SupportedContainerType[] = [
    'ad1', 'ufed', 'ufd', 'ufdr', 'ufdx', 'zip'
  ];
  return supported.includes(containerType.toLowerCase() as SupportedContainerType);
}

/**
 * Check if a container type always recommends lazy loading
 */
export function alwaysUseLazyLoading(containerType: string): boolean {
  const always: SupportedContainerType[] = [
    'e01', 'l01', 'ex01', 'lx01', 'ewf', '7z', 'rar', 'tar', 'raw'
  ];
  return always.includes(containerType.toLowerCase() as SupportedContainerType);
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/**
 * Determine if lazy loading should be used based on container summary
 */
export function shouldUseLazyLoading(
  summary: ContainerSummary, 
  config: LazyLoadConfig = DEFAULT_LAZY_LOAD_CONFIG
): boolean {
  if (!config.enabled) return false;
  if (summary.lazy_loading_recommended) return true;
  return summary.entry_count > config.large_container_threshold;
}

/**
 * Calculate batch size based on config and available entries
 */
export function calculateBatchSize(
  totalEntries: number,
  config: LazyLoadConfig = DEFAULT_LAZY_LOAD_CONFIG
): number {
  return Math.min(config.batch_size, totalEntries);
}

/**
 * Check if pagination is needed
 */
export function needsPagination(
  childCount: number,
  config: LazyLoadConfig = DEFAULT_LAZY_LOAD_CONFIG
): boolean {
  return childCount > config.pagination_threshold;
}

/**
 * Check if a directory should auto-expand
 */
export function shouldAutoExpand(
  childCount: number,
  config: LazyLoadConfig = DEFAULT_LAZY_LOAD_CONFIG
): boolean {
  if (!config.enabled) return true;
  return childCount <= config.auto_expand_threshold;
}

// =============================================================================
// UNIFIED CONTAINER TYPES
// =============================================================================

/**
 * Container type enum - matches Rust ContainerType
 * 
 * Used for type-safe container type handling across the unified API.
 */
export enum ContainerType {
  Ad1 = 'Ad1',
  Ewf = 'Ewf',
  Ufed = 'Ufed',
  Zip = 'Zip',
  SevenZip = 'SevenZip',
  Tar = 'Tar',
  Rar = 'Rar',
  Raw = 'Raw',
}

/**
 * File entry in unified container results - matches Rust FileEntry
 */
export interface FileEntry {
  /** Unique identifier within the container */
  id: string;
  /** Display name */
  name: string;
  /** Full path within container */
  path: string;
  /** Whether this is a directory */
  is_directory: boolean;
  /** File size in bytes (0 for directories) */
  size: number;
  /** File type classification */
  file_type: string;
  /** Number of children (-1 if unknown) */
  child_count: number;
}

/**
 * Get container type from file extension
 * 
 * @deprecated Use detectContainerType from utils/containerUtils.ts instead
 */
export function detectContainerType(path: string): ContainerType | null {
  const ext = path.toLowerCase().split('.').pop();
  switch (ext) {
    case 'ad1':
      return ContainerType.Ad1;
    case 'e01':
    case 'l01':
    case 'ex01':
    case 'lx01':
      return ContainerType.Ewf;
    case 'ufd':
    case 'ufdr':
    case 'ufdx':
      return ContainerType.Ufed;
    case 'zip':
      return ContainerType.Zip;
    case '7z':
      return ContainerType.SevenZip;
    case 'tar':
    case 'gz':
    case 'tgz':
      return ContainerType.Tar;
    case 'rar':
      return ContainerType.Rar;
    case 'dd':
    case 'raw':
    case 'img':
    case '001':
      return ContainerType.Raw;
    default:
      return null;
  }
}
