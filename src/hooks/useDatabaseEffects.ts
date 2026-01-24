// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================
// Database synchronization effects
// =============================================================================

import { createEffect } from "solid-js";
import { debounce } from "@solid-primitives/scheduled";
import type { useDatabase } from "./useDatabase";
import type { useFileManager } from "./useFileManager";

export interface UseDatabaseEffectsOptions {
  db: ReturnType<typeof useDatabase>;
  fileManager: ReturnType<typeof useFileManager>;
  sessionDebounceMs?: number;
  fileSaveDebounceMs?: number;
}

/**
 * Sets up database synchronization effects:
 * - Initializes database session when scan directory changes (debounced)
 * - Saves discovered files to database when they change (batched, debounced)
 */
export function useDatabaseEffects(options: UseDatabaseEffectsOptions): void {
  const {
    db,
    fileManager,
    sessionDebounceMs = 500,
    fileSaveDebounceMs = 1000,
  } = options;

  // Debounced session initialization
  const initSession = debounce((scanDir: string) => {
    db.initSession(scanDir)
      .then(() => console.log(`Database session initialized for: ${scanDir}`))
      .catch((e) => console.warn("Failed to initialize database session:", e));
  }, sessionDebounceMs);

  // Debounced file saving
  const saveFiles = debounce((files: ReturnType<typeof fileManager.discoveredFiles>) => {
    Promise.all(
      files.map((file) =>
        db.saveFile(file).catch((e) =>
          console.warn(`Failed to save file: ${file.path}`, e)
        )
      )
    ).then(() => {
      console.log(`Saved ${files.length} files to database`);
    });
  }, fileSaveDebounceMs);

  // Initialize/update database session when scan directory changes
  createEffect(() => {
    const scanDir = fileManager.scanDir();
    if (scanDir && scanDir.length > 0) {
      initSession(scanDir);
    }
  });

  // Save discovered files to database when they change
  createEffect(() => {
    const files = fileManager.discoveredFiles();
    const session = db.session();
    if (!session || files.length === 0) return;
    saveFiles(files);
  });
}
