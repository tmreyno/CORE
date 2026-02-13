// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Types Module Index
 *
 * Re-exports type definitions from all submodules.
 *
 * @module types
 */

// Container structure types (TreeEntry, ArchiveTreeEntry, SegmentHeader, etc.)
export * from "./container";

// Container info types (Ad1Info, EwfInfo, ContainerInfo, StoredHash, HASH_ALGORITHMS, etc.)
export * from "./containerInfo";

// Database persistence types (DbSession, DbFileRecord, etc.)
export * from "./database";

// Viewer types (FileChunk, HeaderRegion, ParsedMetadata, FileTypeInfo)
export * from "./viewer";

// VFS types (VfsEntry, VfsPartitionInfo, VfsMountInfo)
export * from "./vfs";

// Case document types (CaseDocument, CaseDocumentType)
export * from "./caseDocument";

// Processed database types (AXIOM, PA, etc.)
export * from "./processed";

// Project file types (.cffx)
export * from "./project";

// Lazy loading types for unified container access
export * from "./lazy-loading";

// Viewer metadata types for right panel
export * from "./viewerMetadata";
