// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Types Module Index
 *
 * Re-exports all type definitions and constants from submodules.
 *
 * @module types
 */

// Format definitions and detection utilities
export * from "./formats";

// Evidence lifecycle types (stages, verification, errors)
export * from "./lifecycle";

// Processed database types (AXIOM, PA, etc.)
export * from "./processed";

// Project file types (.cffx)
export * from "./project";

// Lazy loading types for unified container access
export * from "./lazy-loading";

// Container structure types (AD1, EWF, Archive, etc.)
export * from "./containers";

// Hash computation and verification types
export * from "./hash";

// UFED (Cellebrite) container types
export * from "./ufed";

// Companion log types
export * from "./companion";

// Database persistence types
export * from "./database";

// Hex viewer and metadata types
export * from "./viewer";

// VFS (Virtual Filesystem) types
export * from "./vfs";

// Case document types
export * from "./case-documents";

// File discovery types
export * from "./discovery";
