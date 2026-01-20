// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Tree Components - Unified tree view primitives for evidence containers
 * 
 * These components provide a consistent, accessible tree view for all
 * forensic container types (AD1, E01, UFED, Archive, etc.)
 */

// Tree-specific constants and styling tokens
export * from './constants';

// Re-export container utility functions from ui/constants
export { getContainerIconColor, getContainerIconType } from '../ui/constants';

// Re-export icon utility functions
export { getContainerTypeIcon, getDatabaseTypeIcon, getFileIcon } from '../icons';

// Shared primitives
export { TreeRow, type TreeRowProps } from './TreeRow';
export { TreeIcon, type TreeIconProps } from './TreeIcon';
export { ExpandIcon } from './ExpandIcon';
export { TreeEmptyState, type TreeEmptyStateProps } from './TreeEmptyState';
export { TreeLoadingState } from './TreeLoadingState';
export { TreeErrorState, type TreeErrorStateProps } from './TreeErrorState';
export { LoadMoreButton, type LoadMoreButtonProps } from './LoadMoreButton';

// Container-specific tree nodes
export { ContainerHeader, type ContainerHeaderProps } from './ContainerHeader';

// Types
export type { TreeNodeState, TreeItemData } from './types';
