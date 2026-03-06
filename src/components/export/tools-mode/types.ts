// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { Accessor, Setter } from "solid-js";

export type ToolsTabId = "test" | "repair" | "validate" | "extract" | "compress" | "decompress";

export interface ToolsModeProps {
  toolsTab: Accessor<ToolsTabId>;
  setToolsTab: (tab: ToolsTabId) => void;
  testArchivePath: Accessor<string>;
  setTestArchivePath: Setter<string> | ((path: string) => void);
  repairCorruptedPath: Accessor<string>;
  setRepairCorruptedPath: Setter<string> | ((path: string) => void);
  repairOutputPath: Accessor<string>;
  setRepairOutputPath: Setter<string> | ((path: string) => void);
  validateArchivePath: Accessor<string>;
  setValidateArchivePath: Setter<string> | ((path: string) => void);
  extractFirstVolume: Accessor<string>;
  setExtractFirstVolume: Setter<string> | ((path: string) => void);
  extractOutputDir: Accessor<string>;
  setExtractOutputDir: Setter<string> | ((path: string) => void);
  lzmaInputPath: Accessor<string>;
  setLzmaInputPath: Setter<string> | ((path: string) => void);
  lzmaOutputPath: Accessor<string>;
  setLzmaOutputPath: Setter<string> | ((path: string) => void);
  lzmaAlgorithm: Accessor<"lzma" | "lzma2">;
  setLzmaAlgorithm: (algo: "lzma" | "lzma2") => void;
  lzmaLevel: Accessor<number>;
  setLzmaLevel: (level: number) => void;
  lzmaDecompressInput: Accessor<string>;
  setLzmaDecompressInput: Setter<string> | ((path: string) => void);
  lzmaDecompressOutput: Accessor<string>;
  setLzmaDecompressOutput: Setter<string> | ((path: string) => void);
}
