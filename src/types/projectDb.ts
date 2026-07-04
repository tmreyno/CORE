// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Shared per-project SQLite database (.ffxdb) types.
 *
 * This local file remains as a compatibility wrapper so existing CORE-FFX
 * imports do not change while the canonical interfaces live in
 * @core-suite/types.
 */

import type { DbCollectedItem as CoreDbCollectedItem } from "@core-suite/types";

/** File extension for project databases */
export const PROJECT_DB_EXTENSION = ".ffxdb";

export type {
  ActivityQuery,
  DbActivityEntry,
  DbAnnotation,
  DbArtifactCategory,
  DbAxiomCaseInfo,
  DbAxiomEvidenceSource,
  DbAxiomSearchResult,
  DbBookmark,
  DbCaseDocument,
  DbCocAmendment,
  DbCocAuditEntry,
  DbCocItem,
  DbCocTransfer,
  DbEvidenceCollection,
  DbEvidenceDataAlternative,
  DbEvidenceFile,
  DbExportRecord,
  DbFormSubmission,
  DbNote,
  DbProcessedDatabase,
  DbProcessedDbIntegrity,
  DbProcessedDbMetrics,
  DbProjectHash,
  DbProjectSession,
  DbProjectTab,
  DbProjectUser,
  DbProjectVerification,
  DbRecentSearch,
  DbReportRecord,
  DbSavedSearch,
  DbTag,
  DbTagAssignment,
  FtsSearchResult,
  ProjectDbStats,
} from "@core-suite/types";

export interface DbCollectedItem extends CoreDbCollectedItem {
  sourceId?: string;
  sourceRefJson?: string;
  hashAlgorithm?: string;
  hashValue?: string;
  hashComputedAt?: string;
}

export interface EvidenceCollectionPackageImportSummary {
  sourceApp: string;
  sourceCaseNumber: string;
  importedCollections: number;
  importedItems: number;
  importedCocItems: number;
  droppedEvidenceFileLinks: number;
  droppedCocLinks: number;
}
