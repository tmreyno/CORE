// =============================================================================
// CORE-FFX - Forensic File Explorer
// Discovery Types - File discovery and scanning
// =============================================================================

/**
 * Discovery Types
 * 
 * Types for file discovery during directory scanning.
 */

// ============================================================================
// Discovered File
// ============================================================================

export type DiscoveredFile = {
  path: string;
  filename: string;
  container_type: string;
  size: number;
  segment_count?: number;
  created?: string;
  modified?: string;
};

// ============================================================================
// Verify Entry
// ============================================================================

export type VerifyEntry = {
  path: string;
  status: string;
  message?: string;
};
