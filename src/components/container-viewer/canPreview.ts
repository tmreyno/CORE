// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Check if file type can be previewed with a native viewer.
 * Uses centralized type guards — single source of truth.
 */

import {
  isPdf,
  isImage,
  isSpreadsheet,
  isOffice,
  isTextDocument,
  isCode,
  isConfig,
  isEmail,
  isPst,
  isPlist,
  isBinaryExecutable,
  isDatabase,
  isRegistryHive,
} from "../../utils/fileTypeUtils";

export function canPreview(name: string): boolean {
  return (
    isPdf(name) ||
    isImage(name) ||
    isSpreadsheet(name) ||
    isOffice(name) ||
    isTextDocument(name) ||
    isCode(name) ||
    isConfig(name) ||
    isEmail(name) ||
    isPst(name) ||
    isPlist(name) ||
    isBinaryExecutable(name) ||
    isDatabase(name) ||
    isRegistryHive(name)
  );
}
