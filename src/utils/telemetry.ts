// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import {
  createTelemetry,
  type AuditLogEntry as SharedAuditLogEntry,
  type ErrorCategory,
  type ErrorEntry,
  type ErrorSeverity,
} from "@core-suite/logging";
import { logger } from "./logger";

export type { ErrorCategory, ErrorEntry, ErrorSeverity };

export type AuditAction =
  | "file_opened"
  | "hash_computed"
  | "hash_verified"
  | "container_expanded"
  | "file_exported"
  | "report_generated"
  | "project_created"
  | "project_loaded"
  | "project_saved"
  | "search_performed"
  | "entry_selected";

export type AuditLogEntry = SharedAuditLogEntry<AuditAction>;

const telemetry = createTelemetry<AuditAction>({
  logger,
  audit: {
    storageKey: "ffx-audit-log",
    preferencesKey: "ffx-preferences",
    preferenceFlag: "auditLogging",
    maxEntries: 1000,
  },
});

export const {
  generateId,
  getSeverityRank,
  detectCategory,
  sanitizeContext,
  logError,
  logInfo,
  initGlobalErrorHandlers,
  removeGlobalErrorHandlers,
  logAuditAction,
} = telemetry;