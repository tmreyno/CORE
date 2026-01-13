// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * @fileoverview
 * Error telemetry and crash reporting infrastructure.
 * 
 * Provides structured error logging, crash reporting, and user feedback
 * mechanisms for tracking and diagnosing issues in production.
 */

import { createSignal, onMount, onCleanup } from "solid-js";

// ============================================================================
// Types
// ============================================================================

/**
 * Severity levels for errors
 */
export type ErrorSeverity = "debug" | "info" | "warning" | "error" | "fatal";

/**
 * Categories for errors to aid in filtering and analysis
 */
export type ErrorCategory = 
  | "ui"
  | "network"
  | "parser"
  | "database"
  | "filesystem"
  | "tauri"
  | "unknown";

/**
 * Structured error entry
 */
export interface ErrorEntry {
  /** Unique identifier for this error instance */
  id: string;
  /** ISO timestamp when error occurred */
  timestamp: string;
  /** Error severity level */
  severity: ErrorSeverity;
  /** Error category */
  category: ErrorCategory;
  /** Error message */
  message: string;
  /** Error stack trace if available */
  stack?: string;
  /** Component or function where error occurred */
  source?: string;
  /** Additional context data */
  context?: Record<string, unknown>;
  /** User agent string */
  userAgent?: string;
  /** App version */
  appVersion?: string;
  /** Whether user has been notified */
  userNotified?: boolean;
}

/**
 * Error report for crash reporting services
 */
export interface ErrorReport {
  /** All errors since last report */
  errors: ErrorEntry[];
  /** Session information */
  session: {
    id: string;
    startTime: string;
    duration: number;
    platform: string;
    appVersion: string;
  };
  /** System information */
  system: {
    userAgent: string;
    platform: string;
    language: string;
    timezone: string;
    screenSize: string;
    memoryUsage?: number;
  };
}

/**
 * Configuration for error telemetry
 */
export interface TelemetryConfig {
  /** Enable telemetry collection */
  enabled: boolean;
  /** Enable console logging */
  consoleLogging: boolean;
  /** Minimum severity to log */
  minSeverity: ErrorSeverity;
  /** Maximum errors to store */
  maxErrors: number;
  /** Auto-report errors to backend */
  autoReport: boolean;
  /** Backend endpoint for error reports */
  reportEndpoint?: string;
  /** Sample rate (0-1) for error reporting */
  sampleRate: number;
  /** Include user context in reports */
  includeUserContext: boolean;
}

// ============================================================================
// Error Store
// ============================================================================

const DEFAULT_CONFIG: TelemetryConfig = {
  enabled: true,
  consoleLogging: true,
  minSeverity: "warning",
  maxErrors: 100,
  autoReport: false,
  sampleRate: 1.0,
  includeUserContext: false,
};

const telemetryState = {
  config: { ...DEFAULT_CONFIG },
  errors: [] as ErrorEntry[],
  sessionId: generateId(),
  sessionStart: new Date().toISOString(),
  unhandledErrorHandler: null as ((event: ErrorEvent) => void) | null,
  unhandledRejectionHandler: null as ((event: PromiseRejectionEvent) => void) | null,
};

// ============================================================================
// Utilities
// ============================================================================

/**
 * Generate a unique ID
 */
function generateId(): string {
  return `${Date.now().toString(36)}-${Math.random().toString(36).substr(2, 9)}`;
}

/**
 * Get severity rank for comparison
 */
function getSeverityRank(severity: ErrorSeverity): number {
  const ranks: Record<ErrorSeverity, number> = {
    debug: 0,
    info: 1,
    warning: 2,
    error: 3,
    fatal: 4,
  };
  return ranks[severity];
}

/**
 * Detect error category from error object
 */
function detectCategory(error: Error): ErrorCategory {
  const message = error.message.toLowerCase();
  const name = error.name.toLowerCase();

  if (message.includes("network") || message.includes("fetch") || name.includes("network")) {
    return "network";
  }
  if (message.includes("parse") || message.includes("syntax") || name.includes("parse")) {
    return "parser";
  }
  if (message.includes("database") || message.includes("sql") || name.includes("database")) {
    return "database";
  }
  if (message.includes("file") || message.includes("path") || message.includes("enoent")) {
    return "filesystem";
  }
  if (message.includes("tauri") || message.includes("invoke")) {
    return "tauri";
  }
  if (message.includes("render") || message.includes("component") || message.includes("jsx")) {
    return "ui";
  }

  return "unknown";
}

/**
 * Sanitize sensitive data from context
 */
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
// Core Functions
// ============================================================================

/**
 * Configure error telemetry
 */
export function configureTelemetry(config: Partial<TelemetryConfig>): void {
  telemetryState.config = { ...telemetryState.config, ...config };
}

/**
 * Get current telemetry configuration
 */
export function getTelemetryConfig(): TelemetryConfig {
  return { ...telemetryState.config };
}

/**
 * Log an error
 * 
 * @param error - Error object or message
 * @param options - Additional options
 * @returns The created error entry
 * 
 * @example
 * ```tsx
 * // Log a simple error
 * logError(new Error("Something went wrong"));
 * 
 * // Log with context
 * logError(error, {
 *   severity: "error",
 *   category: "parser",
 *   source: "AD1Parser.parse",
 *   context: { filePath, offset },
 * });
 * ```
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

  // Check if we should log this error
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

  // Store error
  telemetryState.errors.push(entry);
  
  // Trim if over max
  if (telemetryState.errors.length > config.maxErrors) {
    telemetryState.errors = telemetryState.errors.slice(-config.maxErrors);
  }

  // Console logging
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

  // Auto-report fatal errors
  if (config.autoReport && severity === "fatal") {
    reportErrors().catch(console.error);
  }

  return entry;
}

/**
 * Log a warning
 */
export function logWarning(
  message: string,
  options?: { source?: string; context?: Record<string, unknown> }
): ErrorEntry {
  return logError(message, { ...options, severity: "warning" });
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

/**
 * Log debug info
 */
export function logDebug(
  message: string,
  options?: { source?: string; context?: Record<string, unknown> }
): ErrorEntry {
  return logError(message, { ...options, severity: "debug" });
}

/**
 * Get all logged errors
 */
export function getErrors(): ErrorEntry[] {
  return [...telemetryState.errors];
}

/**
 * Get errors by category
 */
export function getErrorsByCategory(category: ErrorCategory): ErrorEntry[] {
  return telemetryState.errors.filter(e => e.category === category);
}

/**
 * Get errors by severity
 */
export function getErrorsBySeverity(severity: ErrorSeverity): ErrorEntry[] {
  return telemetryState.errors.filter(e => e.severity === severity);
}

/**
 * Clear all logged errors
 */
export function clearErrors(): void {
  telemetryState.errors = [];
}

/**
 * Generate an error report
 */
export function generateErrorReport(): ErrorReport {
  const now = new Date();
  const sessionStart = new Date(telemetryState.sessionStart);

  return {
    errors: telemetryState.errors,
    session: {
      id: telemetryState.sessionId,
      startTime: telemetryState.sessionStart,
      duration: now.getTime() - sessionStart.getTime(),
      platform: navigator.platform,
      appVersion: "0.1.0", // TODO: Get from app config
    },
    system: {
      userAgent: navigator.userAgent,
      platform: navigator.platform,
      language: navigator.language,
      timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
      screenSize: `${window.screen.width}x${window.screen.height}`,
      // @ts-expect-error - memory API not in types
      memoryUsage: performance.memory?.usedJSHeapSize,
    },
  };
}

/**
 * Report errors to backend
 */
export async function reportErrors(): Promise<boolean> {
  const config = telemetryState.config;
  
  if (!config.reportEndpoint || telemetryState.errors.length === 0) {
    return false;
  }

  // Sample rate check
  if (Math.random() > config.sampleRate) {
    return false;
  }

  try {
    const report = generateErrorReport();
    
    await fetch(config.reportEndpoint, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(report),
    });

    return true;
  } catch (e) {
    console.error("Failed to report errors:", e);
    return false;
  }
}

// ============================================================================
// Global Error Handlers
// ============================================================================

/**
 * Initialize global error handlers
 * Call this once when app starts
 */
export function initGlobalErrorHandlers(): void {
  // Unhandled errors
  telemetryState.unhandledErrorHandler = (event: ErrorEvent) => {
    logError(event.error ?? event.message, {
      severity: "fatal",
      source: `${event.filename}:${event.lineno}:${event.colno}`,
      context: { type: "unhandled_error" },
    });
  };

  // Unhandled promise rejections
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
 * Remove global error handlers
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
// Hooks
// ============================================================================

/**
 * Hook for error telemetry
 */
export function useErrorTelemetry() {
  const [errors, setErrors] = createSignal<ErrorEntry[]>([]);
  const [errorCount, setErrorCount] = createSignal(0);

  onMount(() => {
    initGlobalErrorHandlers();
    
    // Update errors periodically
    const interval = setInterval(() => {
      setErrors(getErrors());
      setErrorCount(telemetryState.errors.length);
    }, 1000);

    onCleanup(() => {
      clearInterval(interval);
      removeGlobalErrorHandlers();
    });
  });

  return {
    errors,
    errorCount,
    logError,
    logWarning,
    logInfo,
    clearErrors,
    generateReport: generateErrorReport,
    reportErrors,
  };
}

/**
 * Wrapper for try/catch with automatic error logging
 * 
 * @example
 * ```tsx
 * const result = await tryCatch(
 *   async () => await parseFile(path),
 *   { source: "parseFile", context: { path } }
 * );
 * 
 * if (result.success) {
 *   console.log(result.data);
 * } else {
 *   console.log(result.error);
 * }
 * ```
 */
export async function tryCatch<T>(
  fn: () => T | Promise<T>,
  options?: {
    source?: string;
    context?: Record<string, unknown>;
    rethrow?: boolean;
  }
): Promise<{ success: true; data: T } | { success: false; error: ErrorEntry }> {
  try {
    const data = await fn();
    return { success: true, data };
  } catch (e) {
    const error = e instanceof Error ? e : new Error(String(e));
    const entry = logError(error, {
      source: options?.source,
      context: options?.context,
    });

    if (options?.rethrow) {
      throw e;
    }

    return { success: false, error: entry };
  }
}

/**
 * Decorator for wrapping functions with error logging
 */
export function withErrorLogging<T extends (...args: any[]) => any>(
  fn: T,
  source: string
): T {
  return ((...args: Parameters<T>) => {
    try {
      const result = fn(...args);
      
      // Handle promises
      if (result instanceof Promise) {
        return result.catch((e: Error) => {
          logError(e, { source, context: { args } });
          throw e;
        });
      }
      
      return result;
    } catch (e) {
      logError(e as Error, { source, context: { args } });
      throw e;
    }
  }) as T;
}
