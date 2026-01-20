// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Central Constants Index
 * 
 * This file re-exports all constants for easy importing.
 * Components can import from '@/components/constants' instead of
 * navigating to specific constant files.
 * 
 * Usage:
 *   import { TREE_ROW_HEIGHT, getFileIconColor } from '@/components/constants';
 */

// Export tree constants (layout, styling)
export * from '../tree/constants';

// Export utility functions for container types
export { 
  getContainerIconColor, 
  getContainerIconType,
  type ContainerIconType,
  CONTAINER_ICON_COLORS,
} from '../ui/constants';
