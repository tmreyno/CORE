// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Barrel re-export for COC and Evidence Collection DB utilities.
 *
 * Split into:
 *   - cocConverters.ts  — type conversion (wizard ↔ DB)
 *   - cocPersistence.ts — DB load/save/delete operations (awaitable invoke)
 *   - cocExport.ts      — export functions and package import/export helpers
 *
 * All downstream consumers can continue importing from this file.
 */

// Type converters
export {
  cocItemToDb,
  dbToCocItem,
  cocTransferToDb,
  dbToCocTransfer,
  evidenceCollectionToDb,
  collectedItemToDb,
  dbToEvidenceCollectionData,
  dbToCollectedItem,
} from "./cocConverters";

// Persistence (awaitable invoke)
export {
  persistCocItemsToDb,
  persistEvidenceCollectionToDb,
  loadCocItemsFromDb,
  loadEvidenceCollectionFromDb,
  loadEvidenceCollectionById,
  loadAllEvidenceCollections,
  updateEvidenceCollectionStatus,
  deleteEvidenceCollection,
} from "./cocPersistence";

// Export functions
export {
  exportEvidenceCollectionPdf,
  exportEvidenceCollection,
  importEvidenceCollectionPackage,
} from "./cocExport";
export type { EvidenceExportFormat } from "./cocExport";
