// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useExaminerProfile — Reads/writes an examiner profile from the project's
 * ui_state table (key: "examiner_profile"). This profile auto-fills examiner
 * fields across all schema-driven forms (report wizard, evidence collection,
 * IAR, etc.) via the `autoFillContext` mechanism in useFormTemplate.
 *
 * The profile is stored as a JSON string in the ui_state table, so no schema
 * migration is required.
 */

import { createSignal } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { dbSync } from "./useProjectDbSync";
import type { FormValue } from "../../templates/types";

const UI_STATE_KEY = "examiner_profile";

export interface ExaminerProfile {
  name: string;
  title: string;
  organization: string;
  badge_number: string;
  email: string;
  phone: string;
}

const EMPTY_PROFILE: ExaminerProfile = {
  name: "",
  title: "",
  organization: "",
  badge_number: "",
  email: "",
  phone: "",
};

/**
 * Load the examiner profile from the project's ui_state table.
 * Returns EMPTY_PROFILE if no profile is stored or if no project DB is open.
 */
export async function loadExaminerProfile(): Promise<ExaminerProfile> {
  try {
    const json = await invoke<string | null>("project_db_get_ui_state", { key: UI_STATE_KEY });
    if (json) {
      return { ...EMPTY_PROFILE, ...JSON.parse(json) };
    }
  } catch {
    // No project DB open or key not found — return empty
  }
  return { ...EMPTY_PROFILE };
}

/**
 * Save the examiner profile to the project's ui_state table (fire-and-forget).
 */
export function saveExaminerProfile(profile: ExaminerProfile): void {
  dbSync.setUiState(UI_STATE_KEY, JSON.stringify(profile));
}

/**
 * Convert an ExaminerProfile to an auto-fill context record for useFormTemplate.
 * Usage: `autoFillContext: { examiner: examinerProfileToContext(profile) }`
 */
export function examinerProfileToContext(
  profile: ExaminerProfile,
): Record<string, FormValue> {
  return {
    name: profile.name,
    title: profile.title,
    organization: profile.organization,
    badge_number: profile.badge_number,
    email: profile.email,
    phone: profile.phone,
  };
}

/**
 * Reactive hook that manages the examiner profile signal.
 * Call `refresh()` after project load to re-read from DB.
 */
export function useExaminerProfile() {
  const [profile, setProfile] = createSignal<ExaminerProfile>({ ...EMPTY_PROFILE });

  const refresh = async () => {
    const p = await loadExaminerProfile();
    setProfile(p);
  };

  const save = (updates: Partial<ExaminerProfile>) => {
    const updated = { ...profile(), ...updates };
    setProfile(updated);
    saveExaminerProfile(updated);
  };

  return {
    /** Current examiner profile */
    profile,
    /** Auto-fill context record for useFormTemplate */
    autoFillContext: () => examinerProfileToContext(profile()),
    /** Save partial updates to the profile */
    save,
    /** Reload profile from DB (call after project load) */
    refresh,
  };
}
