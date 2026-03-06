// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

export { HashBadge } from "./HashBadgeComponent";
export { HashVerificationIndicator } from "./HashVerificationIndicator";
export { CountBadge } from "./CountBadge";
export {
  getHashState,
  hasVerifiedMatch,
  getStoredHashCount,
  getTotalHashCount,
  isHashing,
  isCompleting,
  formatChunks,
} from "./hashHelpers";
export type { HashState, HashBadgeProps } from "./types";
