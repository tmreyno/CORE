// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Shared TypeScript types for forensic container analysis
 *
 * This file re-exports all types from domain-specific modules.
 * Import from here (e.g., `from "../types"`) for convenience.
 *
 * NOTE ON TYPE NAMING:
 * - `TreeEntry` (types/container.ts) - AD1-specific tree entry type matching Rust's `ad1::TreeEntry`
 *   Used for direct Tauri command responses from AD1 operations.
 * - `TreeEntryInfo` (types/lifecycle.ts) - Generic tree entry type for the unified
 *   container trait API. Used in the trait-based abstraction layer.
 */

// Container structure types (TreeEntry, ArchiveTreeEntry, SegmentHeader, etc.)
export * from './types/container';

// Container info types (Ad1Info, EwfInfo, ContainerInfo, StoredHash, HASH_ALGORITHMS, etc.)
export * from './types/containerInfo';

// Database persistence types (DbSession, DbFileRecord, etc.)
export * from './types/database';

// Viewer types (FileChunk, HeaderRegion, ParsedMetadata, FileTypeInfo)
export * from './types/viewer';

// VFS types (VfsEntry, VfsPartitionInfo, VfsMountInfo)
export * from './types/vfs';

// Case document types (CaseDocument, CaseDocumentType)
export * from './types/caseDocument';

// Project file types (.cffx)
export * from './types/project';

// Processed database types (AXIOM, PA, etc.)
export * from './types/processed';
