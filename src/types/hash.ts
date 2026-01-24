// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Hash system type definitions and constants
 * 
 * Provides type-safe hash algorithm names, error types, and shared interfaces
 * for the hash verification system.
 */

// =============================================================================
// Hash Algorithm Constants
// =============================================================================

/**
 * Standard hash algorithm names (canonical form)
 * 
 * These match the backend Rust implementation and should be used consistently
 * throughout the frontend to avoid string inconsistencies.
 */
export const HASH_ALGORITHMS = {
  MD5: 'MD5',
  SHA1: 'SHA-1',
  SHA256: 'SHA-256',
  SHA512: 'SHA-512',
  BLAKE3: 'BLAKE3',
  BLAKE2: 'BLAKE2b',
  XXH3: 'XXH3',
  XXH64: 'XXH64',
  CRC32: 'CRC32',
} as const;

export type HashAlgorithmName = typeof HASH_ALGORITHMS[keyof typeof HASH_ALGORITHMS];

/**
 * Map preference algorithm names to canonical form
 */
export const HASH_ALGORITHM_MAP: Record<string, HashAlgorithmName> = {
  'MD5': HASH_ALGORITHMS.MD5,
  'SHA1': HASH_ALGORITHMS.SHA1,
  'SHA-1': HASH_ALGORITHMS.SHA1,
  'SHA256': HASH_ALGORITHMS.SHA256,
  'SHA-256': HASH_ALGORITHMS.SHA256,
  'SHA512': HASH_ALGORITHMS.SHA512,
  'SHA-512': HASH_ALGORITHMS.SHA512,
  'Blake3': HASH_ALGORITHMS.BLAKE3,
  'BLAKE3': HASH_ALGORITHMS.BLAKE3,
  'Blake2': HASH_ALGORITHMS.BLAKE2,
  'BLAKE2': HASH_ALGORITHMS.BLAKE2,
  'BLAKE2b': HASH_ALGORITHMS.BLAKE2,
  'XXH3': HASH_ALGORITHMS.XXH3,
  'XXH64': HASH_ALGORITHMS.XXH64,
  'CRC32': HASH_ALGORITHMS.CRC32,
};

/**
 * Normalize algorithm name for comparison (removes hyphens, uppercases)
 * 
 * @example
 * normalizeAlgorithm('sha-256') === 'SHA256'
 * normalizeAlgorithm('SHA-1') === 'SHA1'
 */
export function normalizeAlgorithm(algorithm: string): string {
  return algorithm.toUpperCase().replace(/[^A-Z0-9]/g, '');
}

/**
 * Check if two algorithm names are equivalent
 */
export function algorithmsMatch(algo1: string, algo2: string): boolean {
  return normalizeAlgorithm(algo1) === normalizeAlgorithm(algo2);
}

// =============================================================================
// Hash Error Types
// =============================================================================

export type HashErrorCode =
  | 'SEGMENT_MISSING'      // AD1 segments are incomplete
  | 'VERIFICATION_FAILED'  // Hash mismatch with stored value
  | 'UNSUPPORTED_FORMAT'   // Container format not supported for hashing
  | 'FILE_READ_ERROR'      // Failed to read file
  | 'PROGRESS_TIMEOUT'     // Progress event timeout
  | 'COMPUTATION_ERROR'    // Hash computation failed
  | 'INVALID_ALGORITHM';   // Unknown algorithm requested

/**
 * Type-safe hash error with context
 */
export class HashError extends Error {
  constructor(
    public code: HashErrorCode,
    message: string,
    public context?: Record<string, unknown>
  ) {
    super(message);
    this.name = 'HashError';
    
    // Restore prototype chain for proper instanceof checks
    Object.setPrototypeOf(this, HashError.prototype);
  }
  
  /**
   * Check if this is a specific error type
   */
  is(code: HashErrorCode): boolean {
    return this.code === code;
  }
  
  /**
   * Get user-friendly error message
   */
  getUserMessage(): string {
    switch (this.code) {
      case 'SEGMENT_MISSING':
        return 'Cannot hash incomplete container - some segments are missing';
      case 'VERIFICATION_FAILED':
        return 'Hash verification failed - computed hash does not match stored value';
      case 'UNSUPPORTED_FORMAT':
        return 'This container format does not support hash verification';
      case 'FILE_READ_ERROR':
        return 'Failed to read file for hashing';
      case 'PROGRESS_TIMEOUT':
        return 'Hash operation timed out - progress stopped';
      case 'COMPUTATION_ERROR':
        return 'Hash computation failed';
      case 'INVALID_ALGORITHM':
        return 'Unknown hash algorithm requested';
      default:
        return this.message;
    }
  }
  
  /**
   * Convert to string with context
   */
  toString(): string {
    const parts = [`[${this.code}] ${this.message}`];
    if (this.context && Object.keys(this.context).length > 0) {
      parts.push(`Context: ${JSON.stringify(this.context)}`);
    }
    return parts.join(' | ');
  }
}

// =============================================================================
// Stored Hash Types
// =============================================================================

/**
 * Stored hash entry (unified across all container types)
 */
export interface StoredHashEntry {
  /** Algorithm name (canonical form) */
  algorithm: string;
  /** Hash value (lowercase hex) */
  hash: string;
  /** When hash was created (ISO 8601 string) */
  timestamp?: string | null;
  /** Source of hash: container metadata, companion log, etc. */
  source: 'container' | 'companion' | 'computed';
  /** For UFED: which file this hash belongs to */
  filename?: string | null;
  /** Verification status (if compared against computed hash) */
  verified?: boolean | null;
}

/**
 * Hash history entry (for tracking all hash operations)
 */
export interface HashHistoryEntry {
  /** Algorithm used */
  algorithm: string;
  /** Computed hash value */
  hash: string;
  /** When hash was computed */
  timestamp: Date;
  /** How this hash was obtained */
  source: 'stored' | 'computed' | 'verified';
  /** Verification result (if compared) */
  verified?: boolean;
  /** What hash this was verified against */
  verified_against?: string;
}

/**
 * File hash info (current computed hash for a file)
 */
export interface FileHashInfo {
  /** Algorithm used */
  algorithm: string;
  /** Computed hash value */
  hash: string;
  /** Verification status vs stored hash */
  verified?: boolean | null;
}

// =============================================================================
// Progress Event Types
// =============================================================================

/**
 * Hash progress event payload
 */
export interface HashProgressEvent {
  /** File path being hashed */
  path: string;
  /** Progress percentage (0-100) */
  percent: number;
  /** Bytes processed so far */
  bytes_processed?: number;
  /** Total bytes to process */
  total_bytes?: number;
}

/**
 * Batch hash progress event payload
 */
export interface BatchHashProgressEvent extends HashProgressEvent {
  /** Current status */
  status: 'started' | 'progress' | 'completed' | 'error';
  /** Files completed so far */
  files_completed: number;
  /** Total files to hash */
  files_total: number;
  /** Chunks processed (for compressed containers) */
  chunks_processed?: number;
  /** Total chunks */
  chunks_total?: number;
  /** Computed hash (when completed) */
  hash?: string;
  /** Algorithm used */
  algorithm?: string;
  /** Error message (if status is error) */
  error?: string;
}

/**
 * Segment verification progress event
 */
export interface SegmentVerifyProgressEvent {
  /** Segment name/identifier */
  segment_name: string;
  /** Segment number (1-indexed) */
  segment_number: number;
  /** Progress percentage for current segment */
  percent: number;
  /** Segments completed so far */
  segments_completed: number;
  /** Total segments to verify */
  segments_total: number;
}

// =============================================================================
// Progress Listener Types
// =============================================================================

/**
 * Progress listener cleanup function
 */
export type UnlistenFn = () => void;

/**
 * Progress listener configuration
 */
export interface ProgressListenerConfig {
  /** File path being monitored */
  filePath: string;
  /** Callback for progress updates */
  onProgress: (percent: number, chunks?: { processed: number; total: number }) => void;
  /** Callback for errors */
  onError?: (error: string) => void;
  /** Event name to listen for */
  eventName?: string;
}

/**
 * Progress listener handle
 */
export interface ProgressListener {
  /** Cleanup the listener */
  cleanup: UnlistenFn;
  /** File path being monitored */
  filePath: string;
}
