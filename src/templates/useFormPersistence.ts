// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useFormPersistence — Auto-saves form data to .ffxdb via dbSync.
 *
 * Watches a form data accessor and debounces writes to avoid hammering
 * the database on every keystroke. Each template+caseNumber combination
 * gets its own submission row.
 */

import { createEffect, on, onCleanup, type Accessor } from "solid-js";
import { dbSync } from "../hooks/project/useProjectDbSync";
import type { DbFormSubmission } from "../types/projectDb";
import type { FormData } from "./types";

export interface UseFormPersistenceOptions {
  /** Template ID (e.g., "evidence_collection") */
  templateId: string;
  /** Template version (e.g., "1.0.0") */
  templateVersion: string;
  /** Case number — ties the submission to a specific case */
  caseNumber: Accessor<string | undefined>;
  /** The form data to persist */
  data: Accessor<FormData>;
  /** Debounce interval in ms (default: 2000) */
  debounceMs?: number;
  /** Whether persistence is enabled (default: true) */
  enabled?: Accessor<boolean>;
}

/**
 * Generates a deterministic submission ID from template + case number.
 * This ensures upsert semantics — one row per template per case.
 */
function submissionId(templateId: string, caseNumber?: string): string {
  const base = caseNumber ? `${templateId}::${caseNumber}` : templateId;
  // Simple hash-like ID from the base string
  let hash = 0;
  for (let i = 0; i < base.length; i++) {
    const ch = base.charCodeAt(i);
    hash = ((hash << 5) - hash + ch) | 0;
  }
  return `form-${templateId}-${Math.abs(hash).toString(36)}`;
}

export function useFormPersistence(options: UseFormPersistenceOptions): void {
  const debounceMs = options.debounceMs ?? 2000;
  let timer: ReturnType<typeof setTimeout> | undefined;

  const persist = (fd: FormData) => {
    const enabled = options.enabled?.() ?? true;
    if (!enabled) return;

    const cn = options.caseNumber();
    const submission: DbFormSubmission = {
      id: submissionId(options.templateId, cn),
      templateId: options.templateId,
      templateVersion: options.templateVersion,
      caseNumber: cn,
      dataJson: JSON.stringify(fd),
      status: "draft",
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };

    dbSync.upsertFormSubmission(submission);
  };

  createEffect(
    on(
      () => options.data(),
      (fd) => {
        if (!fd) return;
        // Clear any pending debounce
        if (timer) clearTimeout(timer);
        timer = setTimeout(() => persist(fd), debounceMs);
      },
      { defer: true }
    )
  );

  onCleanup(() => {
    if (timer) clearTimeout(timer);
  });
}
