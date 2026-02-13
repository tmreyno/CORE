// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Error telemetry, logging, and forensic audit trail.
 */

import { logger } from "./logger";
const log = logger.scope('Telemetry');

// ============================================================================
// Types
// ============================================================================

export type ErrorSeverity = "debug" | "info" | "warning" | "error" | "fatal";

export type ErrorCategory =
  | "ui"
  | "network"
  | "parser"
  | "database"
  | "filesystem"
  | "tauri"
  | "unknown";

export interface ErrorEntry {
  id: string;
  timestamp: string;
  severity: ErrorSeverity;
  category: ErrorCategory;
  message: string;
  stack?: string;
  source?: string;
  context?: Record<string, unknown>;
  userAgent?: string;
  userNotified?: boolean;
}

// ============================================================================
// State
// ============================================================================

interface TelemetryConfig {
  enabled: boolean;
  consoleLogging: boolean;
  minSeverity: ErrorSeverity;
  maxErrors: number;
}

const telemetryState = {
  config: {
    enabled: true,
    consoleLogging: true,
    minSeverity: "warning" as ErrorSeverity,
    maxErrors: 100,
  } as TelemetryConfig,
  errors: [] as ErrorEntry[],
  unhandledErrorHandler: null as ((event: ErrorEvent) => void) | null,
  unhandledRejectionHandler: null as ((event: PromiseRejectionEvent) => void) | null,
};

// ============================================================================
// Utilities
// ============================================================================

function generateId(): string {
  return `${Date.now().toString(36)}-${Math.random().toString(36).substr(2, 9)}`;
}

function getSeverityRank(severity: ErrorSeverity): number {
  const ranks: Record<ErrorSeverity, number> = {
    debug: 0, info: 1, warning: 2, error: 3, fatal: 4,
  };
  return ranks[severity];
}

function detectCategory(error: Error): ErrorCategory {
  const message = error.message.toLowerCase();
  const name = error.name.toLowerCase();

  if (message.includes("network") || message.includes("fetch") || name.includes("network")) return "network";
  if (message.includes("parse") || message.includes("syntax") || name.includes("parse")) return "parser";
  if (message.includes("database") || message.includes("sql") || name.includes("database")) return "database";
  if (message.includes("file") || message.includes("path") || message.includes("enoent")) return "filesystem";
  if (message.includes("tauri") || message.includes("invoke")) return "tauri";
  if (message.includes("render") || message.includes("component") || message.includes("jsx")) return "ui";
  return "unknown";
}

function sanitizeContext(context: Record<string, unknown>): Record<string, unknown> {
  const sensitiveKeys = ["password", "token", "secret", "key", "auth", "credential"];
  const sanitized: Record<string, unknown> = {};

  for (const [key, value] of Object.entries(context)) {
    const lowerKey = key.toLowerCase();
    if (sensitiveKeys.some(sk => lowerKey.includes(sk))) {
      sanitized[key] = "[REDACTED]";
    } else if (typeof value === "object" && value !== null) {
      sanitized[key] = sanitizeContext(value as Record<string, unknown>);
    } else {
      sanitized[key] = value;
    }
  }

  return sanitized;
}

// ============================================================================
// Core Logging Functions
// ============================================================================

/**
 * Log an error with structured metadata.
 */
export function logError(
  error: Error | string,
  options: {
    severity?: ErrorSeverity;
    category?: ErrorCategory;
    source?: string;
    context?: Record<string, unknown>;
    notify?: boolean;
  } = {}
): ErrorEntry {
  const config = telemetryState.config;
  const errorObj = typeof error === "string" ? new Error(error) : error;
  const severity = options.severity ?? "error";

  if (!config.enabled || getSeverityRank(severity) < getSeverityRank(config.minSeverity)) {
    return {} as ErrorEntry;
  }

  const entry: ErrorEntry = {
    id: generateId(),
    timestamp: new Date().toISOString(),
    severity,
    category: options.category ?? detectCategory(errorObj),
    message: errorObj.message,
    stack: errorObj.stack,
    source: options.source,
    context: options.context ? sanitizeContext(options.context) : undefined,
    userAgent: navigator.userAgent,
    userNotified: options.notify ?? false,
  };

  telemetryState.errors.push(entry);

  if (telemetryState.errors.length > config.maxErrors) {
    telemetryState.errors = telemetryState.errors.slice(-config.maxErrors);
  }

  if (config.consoleLogging) {
    const consoleMethod = severity === "fatal" || severity === "error"
      ? console.error
      : severity === "warning"
        ? console.warn
        : console.log;

    consoleMethod(`[${severity.toUpperCase()}] [${entry.category}] ${entry.message}`, {
      source: entry.source,
      context: entry.context,
      stack: entry.stack,
    });
  }

  return entry;
}

/**
 * Log info
 */
export function logInfo(
  message: string,
  options?: { source?: string; context?: Record<string, unknown> }
): ErrorEntry {
  return logError(message, { ...options, severity: "info" });
}

// ============================================================================
// Global Error Handlers
// ============================================================================

/**
 * Initialize global error handlers. Call once when app starts.
 */
export function initGlobalErrorHandlers(): void {
  telemetryState.unhandledErrorHandler = (event: ErrorEvent) => {
    logError(event.error ?? event.message, {
      severity: "fatal",
      source: `${event.filename}:${event.lineno}:${event.colno}`,
      context: { type: "unhandled_error" },
    });
  };

  telemetryState.unhandledRejectionHandler = (event: PromiseRejectionEvent) => {
    const error = event.reason instanceof Error
      ? event.reason
      : new Error(String(event.reason));

    logError(error, {
      severity: "error",
      source: "Promise",
      context: { type: "unhandled_rejection" },
    });
  };

  window.addEventListener("error", telemetryState.unhandledErrorHandler);
  window.addEventListener("unhandledrejection", telemetryState.unhandledRejectionHandler);
}

/**
 * Remove global error handlers.
 */
export function removeGlobalErrorHandlers(): void {
  if (telemetryState.unhandledErrorHandler) {
    window.removeEventListener("error", telemetryState.unhandledErrorHandler);
    telemetryState.unhandledErrorHandler = null;
  }

  if (telemetryState.unhandledRejectionHandler) {
    window.removeEventListener("unhandledrejection", telemetryState.unhandledRejectionHandler);
    telemetryState.unhandledRejectionHandler = null;
  }
}

// ============================================================================
// Audit Logging (Forensic Action Tracking)
// ============================================================================

export interface AuditLogEntry {
  timestamp: string;
  action: AuditAction;
  details: Record<string, unknown>;
  examiner?: string;
}

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

const AUDIT_LOG_KEY = "ffx-audit-log";
const MAX_AUDIT_ENTRIES = 1000;

function isAuditLoggingEnabled(): boolean {
  try {
    const prefs = localStorage.getItem("ffx-preferences");
    if (prefs) {
      const parsed = JSON.parse(prefs);
      return parsed.auditLogging ?? true;
    }
  } catch {
    // Ignore
  }
  return true;
}

/**
 * Log an auditable forensic action.
 */
export function logAuditAction(action: AuditAction, details: Record<string, unknown>, examiner?: string): void {
  if (!isAuditLoggingEnabled()) return;

  const entry: AuditLogEntry = {
    timestamp: new Date().toISOString(),
    action,
    details,
    examiner,
  };

  try {
    const stored = localStorage.getItem(AUDIT_LOG_KEY);
    const entries: AuditLogEntry[] = stored ? JSON.parse(stored) : [];

    entries.unshift(entry);

    if (entries.length > MAX_AUDIT_ENTRIES) {
      entries.length = MAX_AUDIT_ENTRIES;
    }

    localStorage.setItem(AUDIT_LOG_KEY, JSON.stringify(entries));
    log.debug(`AUDIT: ${action}:`, details);
  } catch (e) {
    log.warn("Failed to write audit log:", e);
  }
}
