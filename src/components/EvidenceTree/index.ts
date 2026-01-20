// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * EvidenceTree Module
 * 
 * Unified tree components for displaying forensic evidence containers.
 * 
 * This module provides:
 * - GenericTreeNode: A unified tree node component that works with all container types
 * - Adapters: Type-specific adapters for AD1, VFS, Archive, UFED, and Lazy entries
 * - Hooks: Reusable state management hooks for tree expansion, selection, and caching
 * - Container detection utilities
 * 
 * @module EvidenceTree
 */

// Types
export * from "./types";

// Container detection utilities
export * from "./containerDetection";

// Generic tree node component
export { GenericTreeNode } from "./GenericTreeNode";
export type { GenericTreeNodeProps } from "./GenericTreeNode";

// Adapters for different container types
export * from "./adapters";

// Hooks for state management
export * from "./hooks";

// Row renderers for different container types
export * from "./renderers";

// Utility functions
export * from "./utils";

// Re-export the main component (EvidenceTree remains the entry point)
// This allows gradual migration while keeping backward compatibility
export { EvidenceTree } from "../EvidenceTree";

// SelectedEntry is canonical in ./types.ts - also re-exported from ../EvidenceTree for compat
export type { SelectedEntry } from "./types";
