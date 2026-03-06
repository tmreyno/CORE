// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { Accessor, Setter } from "solid-js";

export type NativeExportTab = "files" | "archive";

export type ForensicHashAlgorithm = "SHA-256" | "SHA-1" | "MD5" | "SHA-256+MD5";

export interface ForensicPreset {
  id: string;
  name: string;
  description: string;
  compressionLevel: number;
  solid: boolean;
  splitSizeMb: number;
  generateManifest: boolean;
  verifyAfterCreate: boolean;
  hashAlgorithm: ForensicHashAlgorithm;
  includeExaminerInfo: boolean;
}

export interface NativeExportModeProps {
  // Sub-tab
  activeTab: Accessor<NativeExportTab>;
  setActiveTab: Setter<NativeExportTab>;

  // === File Export props ===
  exportName: Accessor<string>;
  setExportName: Setter<string>;
  computeHashes: Accessor<boolean>;
  setComputeHashes: Setter<boolean>;
  verifyAfterCopy: Accessor<boolean>;
  setVerifyAfterCopy: Setter<boolean>;
  generateJsonManifest: Accessor<boolean>;
  setGenerateJsonManifest: Setter<boolean>;
  generateTxtReport: Accessor<boolean>;
  setGenerateTxtReport: Setter<boolean>;

  // === Archive (7z) props ===
  archiveName: Accessor<string>;
  setArchiveName: Setter<string>;
  compressionLevel: Accessor<number>;
  setCompressionLevel: Setter<number>;
  estimatedUncompressed: Accessor<number>;
  estimatedCompressed: Accessor<number>;
  password: Accessor<string>;
  setPassword: Setter<string>;
  showPassword: Accessor<boolean>;
  setShowPassword: Setter<boolean>;
  showAdvanced: Accessor<boolean>;
  setShowAdvanced: Setter<boolean>;
  solid: Accessor<boolean>;
  setSolid: Setter<boolean>;
  numThreads: Accessor<number>;
  setNumThreads: Setter<number>;
  splitSizeMb: Accessor<number>;
  setSplitSizeMb: Setter<number>;
  generateManifest: Accessor<boolean>;
  setGenerateManifest: Setter<boolean>;
  verifyAfterCreate: Accessor<boolean>;
  setVerifyAfterCreate: Setter<boolean>;
  hashAlgorithm: Accessor<ForensicHashAlgorithm>;
  setHashAlgorithm: Setter<ForensicHashAlgorithm>;
  includeExaminerInfo: Accessor<boolean>;
  setIncludeExaminerInfo: Setter<boolean>;
  examinerName: Accessor<string>;
  setExaminerName: Setter<string>;
  caseNumber: Accessor<string>;
  setCaseNumber: Setter<string>;
  evidenceDescription: Accessor<string>;
  setEvidenceDescription: Setter<string>;
}
