// Re-export shim — see ./hash-badge/ for implementation
export {
  HashBadge,
  HashVerificationIndicator,
  getHashState,
  hasVerifiedMatch,
  getStoredHashCount,
  getTotalHashCount,
  isHashing,
  isCompleting,
  formatChunks,
} from "./hash-badge";
export type { HashState, HashBadgeProps } from "./hash-badge";
