// =============================================================================
// CORE-FFX - Forensic File Explorer
// Hash Types - Hash computation, verification, and history
// =============================================================================

/**
 * Hash Types
 * 
 * Types for hash computation, storage, verification, and audit trails.
 */

// ============================================================================
// Stored Hash Types
// ============================================================================

export type StoredHash = {
  algorithm: string;
  hash: string;
  verified?: boolean | null;
  timestamp?: string | null;
  source?: string | null;
  /** Filename this hash belongs to (for UFED which has per-file hashes) */
  filename?: string | null;
  /** Byte offset in file where raw hash bytes are located */
  offset?: number | null;
  /** Size in bytes of the hash (MD5=16, SHA1=20, SHA256=32) */
  size?: number | null;
};

export type SegmentHash = {
  segment_name: string;
  segment_number: number;
  algorithm: string;
  hash: string;
  offset_from?: number | null;
  offset_to?: number | null;
  size?: number | null;
  verified?: boolean | null;
};

export type SegmentHashResult = {
  segment_name: string;
  segment_number: number;
  segment_path: string;
  algorithm: string;
  computed_hash: string;
  expected_hash?: string | null;
  verified?: boolean | null;
  size: number;
  duration_secs: number;
};

// ============================================================================
// Hash History Types
// ============================================================================

export type HashHistoryEntry = {
  algorithm: string;
  hash: string;
  timestamp: Date;
  source: "computed" | "stored" | "verified";
  verified?: boolean | null;
  verified_against?: string | null;
};

// ============================================================================
// Hash Algorithm Types
// ============================================================================

export type HashAlgorithm = "md5" | "sha1" | "sha256" | "sha512" | "blake3" | "blake2" | "xxh3" | "xxh64" | "crc32";

export type HashAlgorithmInfo = { 
  value: HashAlgorithm; 
  label: string; 
  speed: "fast" | "medium" | "slow";
  forensic: boolean;  // Court-accepted for forensics
  cryptographic: boolean;
};

export const HASH_ALGORITHMS: HashAlgorithmInfo[] = [
  { value: "sha1", label: "SHA-1", speed: "medium", forensic: true, cryptographic: true },
  { value: "sha256", label: "SHA-256", speed: "medium", forensic: true, cryptographic: true },
  { value: "md5", label: "MD5", speed: "medium", forensic: true, cryptographic: false },
  { value: "blake3", label: "BLAKE3 ⚡", speed: "fast", forensic: false, cryptographic: true },
  { value: "sha512", label: "SHA-512", speed: "slow", forensic: true, cryptographic: true },
  { value: "blake2", label: "BLAKE2b", speed: "fast", forensic: false, cryptographic: true },
  { value: "xxh3", label: "XXH3 ⚡⚡", speed: "fast", forensic: false, cryptographic: false },
  { value: "xxh64", label: "XXH64 ⚡⚡", speed: "fast", forensic: false, cryptographic: false },
  { value: "crc32", label: "CRC32", speed: "fast", forensic: false, cryptographic: false },
];
