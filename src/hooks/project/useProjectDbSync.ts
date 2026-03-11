// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Write-through sync layer: mirrors in-memory project state to the .ffxdb
 * SQLite database via Tauri IPC commands.
 *
 * All calls are fire-and-forget (non-blocking) — the .cffx JSON remains the
 * source of truth for the current session, while the .ffxdb provides durable
 * queryable storage, FTS5 search, and forensic audit trails.
 *
 * Usage: import { dbSync } from "./useProjectDbSync" in any hook, then call
 * the appropriate method after mutating local state.
 */

import { invoke } from "@tauri-apps/api/core";
import { logger } from "../../utils/logger";
import { getBasename } from "../../utils/pathUtils";
import type {
  DbBookmark,
  DbNote,
  DbActivityEntry,
  DbTag,
  DbTagAssignment,
  DbProjectSession,
  DbProjectUser,
  DbEvidenceFile,
  DbProjectHash,
  DbProjectVerification,
  DbReportRecord,
  DbSavedSearch,
  DbCaseDocument,
  DbRecentSearch,
  DbProcessedDatabase,
  DbAxiomCaseInfo,
  DbCocItem,
  DbCocTransfer,
  DbCocAuditEntry,
  DbEvidenceCollection,
  DbCollectedItem,
  DbEvidenceDataAlternative,
  DbFormSubmission,
  DbExportRecord,
} from "../../types/projectDb";
import type {
  ProjectBookmark,
  ProjectNote,
  ActivityLogEntry,
  ProjectTag,
  ProjectReportRecord,
  SavedSearch,
} from "../../types/project";
import type { CaseDocument } from "../../types/caseDocument";
import type { ProcessedDatabase } from "../../types/processed";

const log = logger.scope("DbSync");

// =============================================================================
// Internal: fire-and-forget invoke wrapper
// =============================================================================

/**
 * Call a Tauri IPC command without blocking the caller.
 * Logs errors but never throws — the .cffx is still the primary store.
 */
function syncInvoke<T>(cmd: string, args: Record<string, unknown>): void {
  invoke<T>(cmd, args).catch((err) => {
    log.warn(`DbSync: ${cmd} failed (non-fatal):`, err);
  });
}

// =============================================================================
// Bookmark sync
// =============================================================================

/** Convert a ProjectBookmark to a DbBookmark for the .ffxdb */
function toDbBookmark(b: ProjectBookmark): DbBookmark {
  return {
    id: b.id,
    targetType: b.target_type,
    targetPath: b.target_path,
    name: b.name,
    createdBy: b.created_by,
    createdAt: b.created_at,
    color: b.color,
    notes: b.notes,
    context: b.context ? JSON.stringify(b.context) : undefined,
  };
}

function syncUpsertBookmark(bookmark: ProjectBookmark): void {
  syncInvoke("project_db_upsert_bookmark", {
    bookmark: toDbBookmark(bookmark),
  });
}

function syncDeleteBookmark(bookmarkId: string): void {
  syncInvoke("project_db_delete_bookmark", { id: bookmarkId });
}

// =============================================================================
// Note sync
// =============================================================================

/** Convert a ProjectNote to a DbNote for the .ffxdb */
function toDbNote(n: ProjectNote): DbNote {
  return {
    id: n.id,
    targetType: n.target_type,
    targetPath: n.target_path,
    title: n.title,
    content: n.content,
    createdBy: n.created_by,
    createdAt: n.created_at,
    modifiedAt: n.modified_at,
    priority: n.priority,
  };
}

function syncUpsertNote(note: ProjectNote): void {
  syncInvoke("project_db_upsert_note", { note: toDbNote(note) });
}

function syncDeleteNote(noteId: string): void {
  syncInvoke("project_db_delete_note", { id: noteId });
}

// =============================================================================
// Activity log sync
// =============================================================================

/** Convert an ActivityLogEntry to a DbActivityEntry for the .ffxdb */
function toDbActivity(a: ActivityLogEntry): DbActivityEntry {
  return {
    id: a.id,
    timestamp: a.timestamp,
    user: a.user,
    category: a.category,
    action: a.action,
    description: a.description,
    filePath: a.file_path,
    details: a.details ? JSON.stringify(a.details) : undefined,
  };
}

function syncInsertActivity(entry: ActivityLogEntry): void {
  syncInvoke("project_db_insert_activity", {
    entry: toDbActivity(entry),
  });
}

// =============================================================================
// Tag sync
// =============================================================================

/** Convert a ProjectTag to a DbTag for the .ffxdb */
function toDbTag(t: ProjectTag): DbTag {
  return {
    id: t.id,
    name: t.name,
    color: t.color,
    description: t.description,
    createdAt: t.created_at,
  };
}

function syncUpsertTag(tag: ProjectTag): void {
  syncInvoke("project_db_upsert_tag", { tag: toDbTag(tag) });
}

function syncDeleteTag(tagId: string): void {
  syncInvoke("project_db_delete_tag", { id: tagId });
}

function syncAssignTag(
  tagId: string,
  targetType: string,
  targetId: string,
  assignedBy: string,
): void {
  const assignment: DbTagAssignment = {
    tagId,
    targetType,
    targetId,
    assignedAt: new Date().toISOString(),
    assignedBy,
  };
  syncInvoke("project_db_assign_tag", { assignment });
}

function syncRemoveTag(
  tagId: string,
  targetType: string,
  targetId: string,
): void {
  syncInvoke("project_db_remove_tag", { tagId, targetType, targetId });
}

// =============================================================================
// Session & User sync
// =============================================================================

function syncUpsertSession(session: DbProjectSession): void {
  syncInvoke("project_db_upsert_session", { session });
}

function syncEndSession(
  sessionId: string,
  summary?: string,
): void {
  syncInvoke("project_db_end_session", { sessionId, summary });
}

function syncUpsertUser(user: DbProjectUser): void {
  syncInvoke("project_db_upsert_user", { user });
}

// =============================================================================
// Evidence & Hash sync
// =============================================================================

function syncUpsertEvidenceFile(file: DbEvidenceFile): void {
  syncInvoke("project_db_upsert_evidence_file", { file });
}

/** Batch insert/update evidence files in a single transaction (awaitable). */
async function batchUpsertEvidenceFiles(files: DbEvidenceFile[]): Promise<number> {
  if (files.length === 0) return 0;
  try {
    return await invoke<number>("project_db_batch_upsert_evidence_files", { files });
  } catch (err) {
    log.warn("batch_upsert_evidence_files failed:", err);
    return 0;
  }
}

function syncInsertHash(hash: DbProjectHash): void {
  syncInvoke("project_db_insert_hash", { hash });
}

function syncInsertVerification(v: DbProjectVerification): void {
  syncInvoke("project_db_insert_verification", { v });
}

// =============================================================================
// Report sync
// =============================================================================

function syncInsertReport(report: ProjectReportRecord): void {
  const dbReport: DbReportRecord = {
    id: report.id,
    title: report.title,
    reportType: report.report_type,
    format: report.format,
    outputPath: report.output_path,
    generatedAt: report.generated_at,
    generatedBy: report.generated_by,
    status: report.status,
    error: report.error,
    config: report.config ? JSON.stringify(report.config) : undefined,
  };
  syncInvoke("project_db_insert_report", { report: dbReport });
}

// =============================================================================
// Case Document sync
// =============================================================================

/** Convert a CaseDocument to a DbCaseDocument for the .ffxdb */
function toDbCaseDocument(doc: CaseDocument): DbCaseDocument {
  return {
    id: `${doc.document_type}-${doc.path}`.replace(/[^a-zA-Z0-9_-]/g, "_").slice(0, 128),
    path: doc.path,
    filename: doc.filename,
    documentType: doc.document_type,
    size: doc.size,
    format: doc.format,
    caseNumber: doc.case_number ?? undefined,
    evidenceId: doc.evidence_id ?? undefined,
    modified: doc.modified ?? undefined,
    discoveredAt: new Date().toISOString(),
  };
}

function syncUpsertCaseDocument(doc: CaseDocument): void {
  syncInvoke("project_db_upsert_case_document", {
    doc: toDbCaseDocument(doc),
  });
}

// =============================================================================
// Recent Search sync
// =============================================================================

function syncInsertRecentSearch(query: string, resultCount: number): void {
  const dbSearch: DbRecentSearch = {
    query,
    timestamp: new Date().toISOString(),
    resultCount,
  };
  syncInvoke("project_db_insert_recent_search", { search: dbSearch });
}

// =============================================================================
// Search sync
// =============================================================================

function syncUpsertSavedSearch(search: SavedSearch): void {
  const dbSearch: DbSavedSearch = {
    id: search.id,
    name: search.name,
    query: search.query,
    searchType: search.search_type,
    isRegex: search.is_regex,
    caseSensitive: search.case_sensitive,
    scope: search.scope,
    createdAt: search.created_at,
    useCount: search.use_count,
    lastUsed: search.last_used,
  };
  syncInvoke("project_db_upsert_saved_search", { search: dbSearch });
}

// =============================================================================
// UI State sync
// =============================================================================

function syncSetUiState(key: string, value: string): void {
  syncInvoke("project_db_set_ui_state", { key, value });
}

// =============================================================================
// Processed Database sync
// =============================================================================

/** Convert a ProcessedDatabase to a DbProcessedDatabase for the .ffxdb */
function toDbProcessedDatabase(db: ProcessedDatabase): DbProcessedDatabase {
  const id = `pdb-${db.path}`.replace(/[^a-zA-Z0-9_-]/g, "_").slice(0, 128);
  return {
    id,
    path: db.path,
    name: db.name || getBasename(db.path) || "Unknown",
    dbType: db.db_type || "Unknown",
    caseNumber: db.case_number,
    examiner: db.examiner,
    createdDate: db.created_date,
    totalSize: db.total_size ?? 0,
    artifactCount: db.artifact_count,
    notes: db.notes,
    registeredAt: new Date().toISOString(),
    metadataJson: db.artifacts ? JSON.stringify({
      artifacts: db.artifacts,
      database_files: db.database_files,
      version: db.version,
    }) : undefined,
  };
}

function syncUpsertProcessedDatabase(db: ProcessedDatabase): void {
  syncInvoke("project_db_upsert_processed_database", {
    db: toDbProcessedDatabase(db),
  });
}

function syncUpsertAxiomCaseInfo(info: DbAxiomCaseInfo): void {
  syncInvoke("project_db_upsert_axiom_case_info", { info });
}

// =============================================================================
// COC Items (v5 — immutability model)
// =============================================================================

function syncInsertCocItem(record: DbCocItem): void {
  syncInvoke("project_db_insert_coc_item", { record });
}

function syncUpsertCocItem(record: DbCocItem): void {
  syncInvoke("project_db_upsert_coc_item", { record });
}

function syncLockCocItem(id: string, lockedBy: string): void {
  syncInvoke("project_db_lock_coc_item", { id, lockedBy });
}

function syncDeleteCocItem(id: string, voidedBy: string, reason: string): void {
  syncInvoke("project_db_delete_coc_item", { id, voidedBy, reason });
}

function syncInsertCocAuditEntry(entry: DbCocAuditEntry): void {
  syncInvoke("project_db_insert_coc_audit_entry", { entry });
}

// =============================================================================
// COC Transfers
// =============================================================================

function syncUpsertCocTransfer(record: DbCocTransfer): void {
  syncInvoke("project_db_upsert_coc_transfer", { record });
}

function syncDeleteCocTransfer(id: string): void {
  syncInvoke("project_db_delete_coc_transfer", { id });
}

// =============================================================================
// Evidence Collections
// =============================================================================

function syncUpsertEvidenceCollection(record: DbEvidenceCollection): void {
  syncInvoke("project_db_upsert_evidence_collection", { record });
}

function syncDeleteEvidenceCollection(id: string): void {
  syncInvoke("project_db_delete_evidence_collection", { id });
}

// =============================================================================
// Collected Items
// =============================================================================

function syncUpsertCollectedItem(record: DbCollectedItem): void {
  syncInvoke("project_db_upsert_collected_item", { record });
}

function syncDeleteCollectedItem(id: string): void {
  syncInvoke("project_db_delete_collected_item", { id });
}

// =============================================================================
// Evidence Data Alternatives (Conflict Resolution — schema v10)
// =============================================================================

function syncUpsertEvidenceDataAlternative(
  record: DbEvidenceDataAlternative
): void {
  syncInvoke("project_db_upsert_evidence_data_alternative", { record });
}

function syncDeleteEvidenceDataAlternative(id: string): void {
  syncInvoke("project_db_delete_evidence_data_alternative", { id });
}

function syncDeleteEvidenceDataAlternativesForItem(
  collectedItemId: string
): void {
  syncInvoke("project_db_delete_evidence_data_alternatives_for_item", {
    collectedItemId,
  });
}

// =============================================================================
// Form Submissions (Generic JSON-driven forms — schema v6)
// =============================================================================

function syncUpsertFormSubmission(submission: DbFormSubmission): void {
  syncInvoke("project_db_upsert_form_submission", { submission });
}

function syncDeleteFormSubmission(id: string): void {
  syncInvoke("project_db_delete_form_submission", { id });
}

// =============================================================================
// Export History
// =============================================================================

function syncInsertExport(record: DbExportRecord): void {
  syncInvoke("project_db_insert_export", { record });
}

function syncUpdateExport(record: DbExportRecord): void {
  syncInvoke("project_db_update_export", { record });
}

// =============================================================================
// Public API — single export object for convenience
// =============================================================================

export const dbSync = {
  // Bookmarks
  upsertBookmark: syncUpsertBookmark,
  deleteBookmark: syncDeleteBookmark,

  // Notes
  upsertNote: syncUpsertNote,
  deleteNote: syncDeleteNote,

  // Activity log
  insertActivity: syncInsertActivity,

  // Tags
  upsertTag: syncUpsertTag,
  deleteTag: syncDeleteTag,
  assignTag: syncAssignTag,
  removeTag: syncRemoveTag,

  // Sessions & Users
  upsertSession: syncUpsertSession,
  endSession: syncEndSession,
  upsertUser: syncUpsertUser,

  // Evidence & Hashes
  upsertEvidenceFile: syncUpsertEvidenceFile,
  batchUpsertEvidenceFiles,
  insertHash: syncInsertHash,
  insertVerification: syncInsertVerification,

  // Reports
  insertReport: syncInsertReport,

  // Case Documents
  upsertCaseDocument: syncUpsertCaseDocument,

  // Searches
  upsertSavedSearch: syncUpsertSavedSearch,
  insertRecentSearch: syncInsertRecentSearch,

  // UI State
  setUiState: syncSetUiState,

  // Processed Databases
  upsertProcessedDatabase: syncUpsertProcessedDatabase,
  upsertAxiomCaseInfo: syncUpsertAxiomCaseInfo,

  // COC Items & Transfers (v5 — immutability model)
  insertCocItem: syncInsertCocItem,
  upsertCocItem: syncUpsertCocItem,
  lockCocItem: syncLockCocItem,
  deleteCocItem: syncDeleteCocItem,
  insertCocAuditEntry: syncInsertCocAuditEntry,
  upsertCocTransfer: syncUpsertCocTransfer,
  deleteCocTransfer: syncDeleteCocTransfer,

  // Evidence Collections & Collected Items
  upsertEvidenceCollection: syncUpsertEvidenceCollection,
  deleteEvidenceCollection: syncDeleteEvidenceCollection,
  upsertCollectedItem: syncUpsertCollectedItem,
  deleteCollectedItem: syncDeleteCollectedItem,

  // Evidence Data Alternatives (Conflict Resolution)
  upsertEvidenceDataAlternative: syncUpsertEvidenceDataAlternative,
  deleteEvidenceDataAlternative: syncDeleteEvidenceDataAlternative,
  deleteEvidenceDataAlternativesForItem:
    syncDeleteEvidenceDataAlternativesForItem,

  // Form Submissions (Generic JSON-driven forms)
  upsertFormSubmission: syncUpsertFormSubmission,
  deleteFormSubmission: syncDeleteFormSubmission,

  // Export History
  insertExport: syncInsertExport,
  updateExport: syncUpdateExport,
} as const;
