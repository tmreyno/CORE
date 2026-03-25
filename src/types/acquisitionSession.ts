// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Acquisition Session — lightweight JSON-backed project data for the Acquire
 * edition. Replaces the full .cffx + .ffxdb dual-file system with a single
 * `.acquisition.json` session file.
 *
 * All data in one flat file: session metadata, acquisitions, evidence
 * collections, activity log, and system snapshot.
 */

// -----------------------------------------------------------------------------
// Top-level session
// -----------------------------------------------------------------------------

export const ACQUISITION_SESSION_VERSION = "1.0";
export const ACQUISITION_SESSION_EXTENSION = ".acquisition.json";

export interface AcquisitionSession {
  version: string;
  createdAt: string;
  modifiedAt: string;

  // Session identity
  caseNumber: string;
  caseName: string;
  examiner: string;
  organization: string;

  // Paths
  sessionFilePath: string;     // Absolute path to the .acquisition.json file itself
  outputFolder: string;        // Root output folder for exports
  evidenceFolder: string;      // Evidence subfolder path

  // Records
  acquisitions: SessionAcquisitionRecord[];
  collections: SessionCollectionRecord[];
  activity: SessionActivityEntry[];

  // System snapshot (captured once at session creation)
  system: SessionSystemInfo;
}

// -----------------------------------------------------------------------------
// Acquisition record
// -----------------------------------------------------------------------------

export interface SessionAcquisitionRecord {
  id: string;
  type: "e01" | "l01" | "raw" | "aff4" | "archive" | "file_copy" | "memory" | "triage";
  status: "in_progress" | "completed" | "failed";
  outputPath: string;
  sources: string[];

  // Forensic metadata
  format: string;
  totalBytes: number;
  totalFiles?: number;
  segments?: number;
  compressed: boolean;

  // Hashes
  md5?: string;
  sha1?: string;
  sha256?: string;

  // Timing
  startedAt: string;
  completedAt?: string;
  durationMs?: number;

  // Case context
  caseNumber?: string;
  evidenceNumber?: string;
  examiner?: string;
  description?: string;
  notes?: string;

  // Error (if failed)
  error?: string;
}

// -----------------------------------------------------------------------------
// Evidence collection record
// -----------------------------------------------------------------------------

export interface SessionCollectionRecord {
  id: string;
  caseNumber: string;
  collectionDate: string;
  collectionLocation: string;
  collectingOfficer: string;
  documentationNotes: string;
  status: "draft";
  items: SessionCollectedItem[];
  createdAt: string;
  modifiedAt: string;
}

export interface SessionCollectedItem {
  id: string;
  collectionId: string;
  itemNumber: string;
  description: string;
  foundLocation: string;
  itemType: string;
  condition: string;
  imageFormat: string;
  acquisitionMethod: string;
  storageNotes: string;
  notes: string;
  itemCollectionDatetime: string;
  itemCollectingOfficer: string;
  deviceType: string;
  brand: string;
  make: string;
  model: string;
  serialNumber: string;
  building: string;
}

// -----------------------------------------------------------------------------
// Activity entry
// -----------------------------------------------------------------------------

export interface SessionActivityEntry {
  id: string;
  timestamp: string;
  action: string;
  description: string;
  details?: string;
}

// -----------------------------------------------------------------------------
// System snapshot
// -----------------------------------------------------------------------------

export interface SessionSystemInfo {
  hostname: string;
  username: string;
  osName: string;
  osVersion: string;
  systemModel: string;
  systemSerialNumber: string;
  systemManufacturer: string;
  drives: SessionDriveSnapshot[];
}

export interface SessionDriveSnapshot {
  name: string;
  mountPoint: string;
  fileSystem: string;
  totalBytes: number;
  availableBytes: number;
  kind: string;
  isRemovable: boolean;
}

// -----------------------------------------------------------------------------
// Factory
// -----------------------------------------------------------------------------

export function createEmptySession(opts: {
  caseNumber: string;
  examiner: string;
  outputFolder: string;
  sessionFilePath: string;
  organization?: string;
  caseName?: string;
}): AcquisitionSession {
  const now = new Date().toISOString();
  return {
    version: ACQUISITION_SESSION_VERSION,
    createdAt: now,
    modifiedAt: now,
    caseNumber: opts.caseNumber,
    caseName: opts.caseName || "",
    examiner: opts.examiner,
    organization: opts.organization || "",
    sessionFilePath: opts.sessionFilePath,
    outputFolder: opts.outputFolder,
    evidenceFolder: `${opts.outputFolder}/Evidence`,
    acquisitions: [],
    collections: [],
    activity: [],
    system: {
      hostname: "",
      username: "",
      osName: "",
      osVersion: "",
      systemModel: "",
      systemSerialNumber: "",
      systemManufacturer: "",
      drives: [],
    },
  };
}
