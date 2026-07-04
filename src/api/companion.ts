// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { invoke } from "@tauri-apps/api/core";
import { isTauri } from "../utils/platform";

// --- Types matching src-tauri/src/commands/companion.rs ---

export interface CompanionCaseInfo {
  caseNumber: string;
  evidenceNumber: string;
  examiner: string;
  description: string;
  notes: string;
}

export interface CompanionSourceInfo {
  paths: string[];
  totalFiles: number;
  totalBytes: number;
}

export interface CompanionOutputInfo {
  primaryPath: string;
  format: string;
  segments?: string[];
  totalBytes: number;
  totalFiles?: number;
  compressed?: boolean;
  segmentSize?: number;
}

export interface CompanionHashes {
  md5: string;
  sha1: string;
  sha256: string;
}

export interface CompanionTiming {
  startedAt: string;
  completedAt: string;
  durationMs: number;
}

export interface CompanionSystemInfo {
  hostname: string;
  username: string;
  sourceDrive: string;
  sourceFileSystem: string;
  sourceCapacity: number;
  sourceDriveType: string;
  sourceRemovable: boolean;
  // System identification (from Identify phase)
  systemModel?: string;
  systemSerialNumber?: string;
  systemManufacturer?: string;
  osName?: string;
  osVersion?: string;
}

export interface CompanionFileInput {
  acquisitionType: string;
  case?: CompanionCaseInfo;
  source: CompanionSourceInfo;
  output: CompanionOutputInfo;
  hashes?: CompanionHashes;
  timing: CompanionTiming;
  system?: CompanionSystemInfo;
}

export interface CompanionFile {
  version: string;
  tool: string;
  toolVersion: string;
  createdAt: string;
  acquisitionType: string;
  case: CompanionCaseInfo;
  source: CompanionSourceInfo;
  output: CompanionOutputInfo;
  hashes: CompanionHashes;
  timing: CompanionTiming;
  system?: CompanionSystemInfo;
}

// --- Invoke wrappers ---

export async function writeCompanionFile(
  outputPath: string,
  data: CompanionFileInput,
): Promise<string> {
  if (!isTauri) {
    void outputPath;
    void data;
    throw new Error("Companion sidecar writing is available in the desktop app.");
  }

  return invoke<string>("write_companion_file", {
    outputPath,
    data,
  });
}

export async function readCompanionFile(
  companionPath: string,
): Promise<CompanionFile> {
  if (!isTauri) {
    void companionPath;
    throw new Error("Companion sidecar reading is available in the desktop app.");
  }

  return invoke<CompanionFile>("read_companion_file", { companionPath });
}

export async function findCompanionFile(
  outputPath: string,
): Promise<string | null> {
  if (!isTauri) {
    void outputPath;
    return null;
  }

  return invoke<string | null>("find_companion_file", { evidencePath: outputPath });
}
