// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useDatabase - Hook for SQLite database operations
 * 
 * Provides persistence for:
 * - Sessions (open directories)
 * - Files (discovered containers)
 * - Hashes (computed hash records)
 * - Verifications (audit trail)
 * - Open tabs (UI state)
 * - Settings (app preferences)
 */

import { invoke } from "@tauri-apps/api/core";
import { createSignal } from "solid-js";
import type { 
  DbSession, 
  DbFileRecord, 
  DbHashRecord, 
  DbVerificationRecord, 
  DbOpenTabRecord,
  DiscoveredFile 
} from "../types";

// ============================================================================
// Session Operations
// ============================================================================

/** Get or create a session for a directory path */
export async function getOrCreateSession(rootPath: string): Promise<DbSession> {
  return invoke<DbSession>("db_get_or_create_session", { rootPath });
}

/** Get recent sessions */
export async function getRecentSessions(limit: number = 10): Promise<DbSession[]> {
  return invoke<DbSession[]>("db_get_recent_sessions", { limit });
}

/** Get the last opened session */
export async function getLastSession(): Promise<DbSession | null> {
  return invoke<DbSession | null>("db_get_last_session");
}

// ============================================================================
// File Operations
// ============================================================================

/** Generate a unique ID */
function generateId(): string {
  return crypto.randomUUID();
}

/** Create a file record from a discovered file */
export function createFileRecord(
  sessionId: string,
  file: DiscoveredFile
): DbFileRecord {
  return {
    id: generateId(),
    session_id: sessionId,
    path: file.path,
    filename: file.filename,
    container_type: file.container_type,
    total_size: file.size,
    segment_count: file.segment_count ?? 1,
    discovered_at: new Date().toISOString(),
  };
}

/** Save or update a file record */
export async function upsertFile(file: DbFileRecord): Promise<void> {
  return invoke("db_upsert_file", { file });
}

/** Get all files for a session */
export async function getFilesForSession(sessionId: string): Promise<DbFileRecord[]> {
  return invoke<DbFileRecord[]>("db_get_files_for_session", { sessionId });
}

/** Get a file by path */
export async function getFileByPath(sessionId: string, path: string): Promise<DbFileRecord | null> {
  return invoke<DbFileRecord | null>("db_get_file_by_path", { sessionId, path });
}

// ============================================================================
// Hash Operations
// ============================================================================

/** Create a hash record */
export function createHashRecord(
  fileId: string,
  algorithm: string,
  hashValue: string,
  source: "computed" | "stored" | "imported" = "computed",
  segmentIndex?: number,
  segmentName?: string
): DbHashRecord {
  return {
    id: generateId(),
    file_id: fileId,
    algorithm,
    hash_value: hashValue,
    computed_at: new Date().toISOString(),
    segment_index: segmentIndex ?? null,
    segment_name: segmentName ?? null,
    source,
  };
}

/** Insert a hash record */
export async function insertHash(hash: DbHashRecord): Promise<void> {
  return invoke("db_insert_hash", { hash });
}

/** Get all hashes for a file */
export async function getHashesForFile(fileId: string): Promise<DbHashRecord[]> {
  return invoke<DbHashRecord[]>("db_get_hashes_for_file", { fileId });
}

/** Get the latest hash for a file/algorithm/segment combo */
export async function getLatestHash(
  fileId: string,
  algorithm: string,
  segmentIndex?: number
): Promise<DbHashRecord | null> {
  return invoke<DbHashRecord | null>("db_get_latest_hash", { 
    fileId, 
    algorithm, 
    segmentIndex: segmentIndex ?? null 
  });
}

// ============================================================================
// Verification Operations
// ============================================================================

/** Create a verification record */
export function createVerificationRecord(
  hashId: string,
  result: "match" | "mismatch",
  expectedHash: string,
  actualHash: string
): DbVerificationRecord {
  return {
    id: generateId(),
    hash_id: hashId,
    verified_at: new Date().toISOString(),
    result,
    expected_hash: expectedHash,
    actual_hash: actualHash,
  };
}

/** Insert a verification record */
export async function insertVerification(verification: DbVerificationRecord): Promise<void> {
  return invoke("db_insert_verification", { verification });
}

/** Get verifications for a file */
export async function getVerificationsForFile(fileId: string): Promise<DbVerificationRecord[]> {
  return invoke<DbVerificationRecord[]>("db_get_verifications_for_file", { fileId });
}

// ============================================================================
// Open Tabs Operations
// ============================================================================

/** Save open tabs for a session */
export async function saveOpenTabs(sessionId: string, tabs: DbOpenTabRecord[]): Promise<void> {
  return invoke("db_save_open_tabs", { sessionId, tabs });
}

/** Get open tabs for a session */
export async function getOpenTabs(sessionId: string): Promise<DbOpenTabRecord[]> {
  return invoke<DbOpenTabRecord[]>("db_get_open_tabs", { sessionId });
}

/** Create tab records from paths */
export function createTabRecords(
  sessionId: string,
  filePaths: string[],
  activeFilePath?: string
): DbOpenTabRecord[] {
  return filePaths.map((path, index) => ({
    id: generateId(),
    session_id: sessionId,
    file_path: path,
    tab_order: index,
    is_active: path === activeFilePath,
  }));
}

// ============================================================================
// Settings Operations
// ============================================================================

/** Set a setting value */
export async function setSetting(key: string, value: string): Promise<void> {
  return invoke("db_set_setting", { key, value });
}

/** Get a setting value */
export async function getSetting(key: string): Promise<string | null> {
  return invoke<string | null>("db_get_setting", { key });
}

/** Get a setting with a default value */
export async function getSettingWithDefault(key: string, defaultValue: string): Promise<string> {
  const value = await getSetting(key);
  return value ?? defaultValue;
}

// ============================================================================
// useDatabase Hook
// ============================================================================

export interface DatabaseState {
  session: DbSession | null;
  files: Map<string, DbFileRecord>; // path -> record
  isLoading: boolean;
  error: string | null;
}

/**
 * Hook to manage database state for a session
 */
export function useDatabase() {
  const [session, setSession] = createSignal<DbSession | null>(null);
  const [files, setFiles] = createSignal<Map<string, DbFileRecord>>(new Map());
  const [isLoading, setIsLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  /**
   * Initialize or load a session for a directory
   */
  async function initSession(rootPath: string): Promise<DbSession> {
    setIsLoading(true);
    setError(null);
    try {
      const sess = await getOrCreateSession(rootPath);
      setSession(sess);
      
      // Load files for this session
      const fileRecords = await getFilesForSession(sess.id);
      const fileMap = new Map<string, DbFileRecord>();
      for (const file of fileRecords) {
        fileMap.set(file.path, file);
      }
      setFiles(fileMap);
      
      return sess;
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
      throw e;
    } finally {
      setIsLoading(false);
    }
  }

  /**
   * Try to restore the last session
   */
  async function restoreLastSession(): Promise<DbSession | null> {
    setIsLoading(true);
    setError(null);
    try {
      const sess = await getLastSession();
      if (sess) {
        setSession(sess);
        
        // Load files for this session
        const fileRecords = await getFilesForSession(sess.id);
        const fileMap = new Map<string, DbFileRecord>();
        for (const file of fileRecords) {
          fileMap.set(file.path, file);
        }
        setFiles(fileMap);
      }
      return sess;
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
      return null;
    } finally {
      setIsLoading(false);
    }
  }

  /**
   * Save a discovered file to the database
   */
  async function saveFile(file: DiscoveredFile): Promise<DbFileRecord> {
    const sess = session();
    if (!sess) {
      throw new Error("No active session");
    }
    
    // Check if file already exists
    let record = files().get(file.path);
    if (!record) {
      record = createFileRecord(sess.id, file);
    } else {
      // Update existing record
      record = {
        ...record,
        filename: file.filename,
        container_type: file.container_type,
        total_size: file.size,
        segment_count: file.segment_count ?? 1,
      };
    }
    
    await upsertFile(record);
    
    // Update local state
    setFiles(prev => {
      const newMap = new Map(prev);
      newMap.set(file.path, record!);
      return newMap;
    });
    
    return record;
  }

  /**
   * Save a hash result to the database
   */
  async function saveHash(
    filePath: string,
    algorithm: string,
    hashValue: string,
    source: "computed" | "stored" | "imported" = "computed",
    segmentIndex?: number,
    segmentName?: string
  ): Promise<DbHashRecord> {
    const fileRecord = files().get(filePath);
    if (!fileRecord) {
      throw new Error(`File not found in database: ${filePath}`);
    }
    
    const hash = createHashRecord(
      fileRecord.id,
      algorithm,
      hashValue,
      source,
      segmentIndex,
      segmentName
    );
    
    await insertHash(hash);
    return hash;
  }

  /**
   * Save a verification result to the database
   */
  async function saveVerification(
    hashId: string,
    result: "match" | "mismatch",
    expectedHash: string,
    actualHash: string
  ): Promise<DbVerificationRecord> {
    const verification = createVerificationRecord(
      hashId,
      result,
      expectedHash,
      actualHash
    );
    
    await insertVerification(verification);
    return verification;
  }

  /**
   * Get hash history for a file
   */
  async function getFileHashes(filePath: string): Promise<DbHashRecord[]> {
    const fileRecord = files().get(filePath);
    if (!fileRecord) {
      return [];
    }
    return getHashesForFile(fileRecord.id);
  }

  /**
   * Get verification history for a file
   */
  async function getFileVerifications(filePath: string): Promise<DbVerificationRecord[]> {
    const fileRecord = files().get(filePath);
    if (!fileRecord) {
      return [];
    }
    return getVerificationsForFile(fileRecord.id);
  }

  /**
   * Save current open tabs
   */
  async function saveTabs(filePaths: string[], activeFilePath?: string): Promise<void> {
    const sess = session();
    if (!sess) return;
    
    const tabs = createTabRecords(sess.id, filePaths, activeFilePath);
    await saveOpenTabs(sess.id, tabs);
  }

  /**
   * Load open tabs for current session
   */
  async function loadTabs(): Promise<DbOpenTabRecord[]> {
    const sess = session();
    if (!sess) return [];
    
    return getOpenTabs(sess.id);
  }

  return {
    // State
    session,
    files,
    isLoading,
    error,
    // Session operations
    initSession,
    restoreLastSession,
    // File operations
    saveFile,
    getFileRecord: (path: string) => files().get(path),
    // Hash operations
    saveHash,
    getFileHashes,
    // Verification operations
    saveVerification,
    getFileVerifications,
    // Tab operations
    saveTabs,
    loadTabs,
    // Settings
    setSetting,
    getSetting,
    getSettingWithDefault,
  };
}
