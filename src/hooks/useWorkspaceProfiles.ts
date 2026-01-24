// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { invoke } from "@tauri-apps/api/core";
import { createSignal } from "solid-js";

/**
 * Workspace profile from backend
 */
export interface WorkspaceProfile {
  id: string;
  name: string;
  profile_type: ProfileType;
  description: string;
  created_at: string;
  last_used: string;
  use_count: number;
  layout_config: LayoutConfig;
  panel_config: PanelConfig;
  shortcuts: Record<string, string>;
  theme_overrides: Record<string, string>;
}

export type ProfileType =
  | "investigation"
  | "analysis"
  | "reporting"
  | "review"
  | "triage"
  | "acquisition"
  | "preservation"
  | "custom";

export interface LayoutConfig {
  sidebar_width: number;
  panel_heights: Record<string, number>;
  split_ratios: number[];
  collapsed_panels: string[];
}

export interface PanelConfig {
  visible_panels: string[];
  panel_order: string[];
  default_panel: string;
  auto_hide_panels: string[];
}

/**
 * Profile summary for listing
 */
export interface ProfileSummary {
  id: string;
  name: string;
  profile_type: ProfileType;
  last_used: string;
  use_count: number;
}

/**
 * Hook for workspace profile management
 */
export function useWorkspaceProfiles() {
  const [profiles, setProfiles] = createSignal<ProfileSummary[]>([]);
  const [currentProfile, setCurrentProfile] = createSignal<WorkspaceProfile | null>(null);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  /**
   * List all available profiles
   */
  const listProfiles = async () => {
    try {
      setLoading(true);
      setError(null);
      const result = await invoke<ProfileSummary[]>("profile_list");
      setProfiles(result);
      return result;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      console.error("Failed to list profiles:", err);
      return [];
    } finally {
      setLoading(false);
    }
  };

  /**
   * Load a profile by ID
   */
  const loadProfile = async (profileId: string) => {
    try {
      setLoading(true);
      setError(null);
      const result = await invoke<WorkspaceProfile>("profile_load", {
        profileId,
      });
      setCurrentProfile(result);
      return result;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      console.error("Failed to load profile:", err);
      return null;
    } finally {
      setLoading(false);
    }
  };

  /**
   * Save current workspace as a profile
   */
  const saveProfile = async (
    name: string,
    profileType: ProfileType,
    description: string = ""
  ): Promise<string | null> => {
    try {
      setLoading(true);
      setError(null);
      const profileId = await invoke<string>("profile_save", {
        name,
        profileType,
        description,
      });
      // Refresh profile list
      await listProfiles();
      return profileId;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      console.error("Failed to save profile:", err);
      return null;
    } finally {
      setLoading(false);
    }
  };

  /**
   * Update an existing profile
   */
  const updateProfile = async (
    profileId: string,
    updates: Partial<WorkspaceProfile>
  ): Promise<boolean> => {
    try {
      setLoading(true);
      setError(null);
      await invoke<void>("profile_update", {
        profileId,
        updates,
      });
      // Refresh profile list
      await listProfiles();
      return true;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      console.error("Failed to update profile:", err);
      return false;
    } finally {
      setLoading(false);
    }
  };

  /**
   * Delete a profile
   */
  const deleteProfile = async (profileId: string): Promise<boolean> => {
    try {
      setLoading(true);
      setError(null);
      await invoke<void>("profile_delete", { profileId });
      // Refresh profile list
      await listProfiles();
      return true;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      console.error("Failed to delete profile:", err);
      return false;
    } finally {
      setLoading(false);
    }
  };

  /**
   * Apply a profile to current workspace
   */
  const applyProfile = async (profileId: string): Promise<boolean> => {
    try {
      setLoading(true);
      setError(null);
      await invoke<void>("profile_apply", { profileId });
      const profile = await loadProfile(profileId);
      return profile !== null;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      console.error("Failed to apply profile:", err);
      return false;
    } finally {
      setLoading(false);
    }
  };

  /**
   * Get profile by type (returns most recently used)
   */
  const getProfileByType = async (profileType: ProfileType) => {
    const allProfiles = await listProfiles();
    const matching = allProfiles.filter((p) => p.profile_type === profileType);
    if (matching.length === 0) return null;
    // Sort by last_used descending
    matching.sort((a, b) => b.last_used.localeCompare(a.last_used));
    return matching[0];
  };

  return {
    // State
    profiles,
    currentProfile,
    loading,
    error,
    // Actions
    listProfiles,
    loadProfile,
    saveProfile,
    updateProfile,
    deleteProfile,
    applyProfile,
    getProfileByType,
  };
}
