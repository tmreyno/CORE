// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Types Module Index
 *
 * Re-exports type definitions from submodules.
 * Note: Most types are defined in src/types.ts. This folder contains
 * only modular type definitions that are directly imported by specific modules.
 *
 * @module types
 */

// Processed database types (AXIOM, PA, etc.)
export * from "./processed";

// Project file types (.ffxproj)
export * from "./project";

// Lazy loading types for unified container access
export * from "./lazy-loading";
