// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * EvidenceTree Hooks Index
 * 
 * Re-exports all tree management hooks.
 */

// Core state hooks
export { useTreeState, useCacheState } from "./useTreeState";
export type { TreeStateReturn, CacheStateReturn } from "./useTreeState";

// Master composing hook
export { useEvidenceTree } from "./useEvidenceTree";
export type { UseEvidenceTreeReturn, UseEvidenceTreeProps, SelectedEntry } from "./useEvidenceTree";

// Container-specific hooks
export { useAd1Tree } from "./useAd1Tree";
export type { UseAd1TreeReturn } from "./useAd1Tree";

export { useVfsTree } from "./useVfsTree";
export type { UseVfsTreeReturn } from "./useVfsTree";

export { useArchiveTree, type ArchiveQuickMetadata } from "./useArchiveTree";
export type { UseArchiveTreeReturn } from "./useArchiveTree";

export { useLazyTree } from "./useLazyTree";
export type { UseLazyTreeReturn } from "./useLazyTree";

// Legacy hooks (for backward compatibility)
export { useTreeExpansion } from "./useTreeExpansion";
export type { UseTreeExpansionReturn } from "./useTreeExpansion";

export { useTreeSelection } from "./useTreeSelection";
export type { UseTreeSelectionReturn } from "./useTreeSelection";

export { useTreeCache } from "./useTreeCache";
export type { UseTreeCacheReturn } from "./useTreeCache";

export { useTreeLoading } from "./useTreeLoading";
export type { UseTreeLoadingReturn } from "./useTreeLoading";
