// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { invoke } from "@tauri-apps/api/core";

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
  path: string;
  format: string;
  segments: number;
  totalBytes: number;
  compressed: boolean;
  segmentSize: number;
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

export interface CompanionFileInput {
  acquisitionType: string;
  case: CompanionCaseInfo;
  source: CompanionSourceInfo;
  output: CompanionOutputInfo;
  hashes: CompanionHashes;
  timing: CompanionTiming;
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
}

// --- Invoke wrappers ---

export async function writeCompanionFile(
  outputPath: string,
  data: CompanionFileInput,
): Promise<string> {
  return invoke<string>("write_companion_file", {
    outputPath,
    data,
  });
}

export async function readCompanionFile(
  companionPath: string,
): Promise<CompanionFile> {
  return invoke<CompanionFile>("read_companion_file", { companionPath });
}

export async function findCompanionFile(
  outputPath: string,
): Promise<CompanionFile | null> {
  return invoke<CompanionFile | null>("find_companion_file", { outputPath });
}
