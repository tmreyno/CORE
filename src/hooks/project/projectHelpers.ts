// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Re-export barrel for backward compatibility.
 *
 * The actual implementations live in:
 *   - projectSaveOptions.ts  — buildSaveOptions + BuildSaveOptionsParams
 *   - projectLoader.ts       — handleLoadProject + HandleLoadProjectParams
 *   - projectSetup.ts        — createDocumentEntry, handleOpenDirectory,
 *                              handleProjectSetupComplete + param types
 */

export { buildSaveOptions, type BuildSaveOptionsParams } from "./projectSaveOptions";
export { handleLoadProject, type HandleLoadProjectParams } from "./projectLoader";
export {
  createDocumentEntry,
  handleOpenDirectory,
  handleProjectSetupComplete,
  type HandleOpenDirectoryParams,
  type HandleProjectSetupCompleteParams,
} from "./projectSetup";
