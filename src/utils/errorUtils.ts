// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Error Utility Functions
 *
 * Provides type-safe error message extraction for use in catch blocks
 * with `unknown` error types (TypeScript best practice).
 *
 * @example
 * ```ts
 * try {
 *   await riskyOperation();
 * } catch (error: unknown) {
 *   toast.error("Failed", getErrorMessage(error));
 * }
 * ```
 */

/**
 * Extract a human-readable error message from an unknown error value.
 *
 * Handles Error instances, string throws, and arbitrary values.
 * Preferred over `catch (error: any)` for type safety.
 */
export function getErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  if (typeof error === "string") {
    return error;
  }
  return String(error);
}
