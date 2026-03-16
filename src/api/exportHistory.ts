// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Export history API — typed wrappers for export record CRUD.
 *
 * Wraps the `project_db_get_exports` and `project_db_delete_export` Tauri
 * commands. Insert/update are handled by `dbSync` (fire-and-forget) in the
 * individual export hooks.
 */

import { invoke } from "@tauri-apps/api/core";
import type { DbExportRecord } from "../types/projectDb";

/**
 * Get export records from the project database, most recent first.
 * @param limit Maximum number of records to return. Omit for all.
 */
export async function getExportHistory(limit?: number): Promise<DbExportRecord[]> {
  return invoke<DbExportRecord[]>("project_db_get_exports", { limit: limit ?? null });
}

/**
 * Delete a single export record by ID.
 */
export async function deleteExportRecord(id: string): Promise<void> {
  return invoke("project_db_delete_export", { id });
}
