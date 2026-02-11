// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { invoke } from "@tauri-apps/api/core";
import { createSignal } from "solid-js";
import { logger } from "../utils/logger";
const log = logger.scope("WorkspaceProfiles");

// =============================================================================
// Type Definitions - Aligned with backend workspace_profiles.rs
// =============================================================================

/**
 * Profile type enum - matches backend ProfileType
 */
export type ProfileType =
  | "Investigation"
  | "Analysis"
  | "Review"
  | "Mobile"
  | "Computer"
  | "Network"
  | "IncidentResponse"
  | "Custom";

/**
 * Center pane layout mode - matches backend CenterLayout
 */
export type CenterLayout = "Single" | "SplitVertical" | "SplitHorizontal" | "Grid";

/**
 * Layout configuration - matches backend LayoutConfig
 */
export interface LayoutConfig {
  left_panel_width: number;
  right_panel_width: number;
  bottom_panel_height: number;
  left_panel_collapsed: boolean;
  right_panel_collapsed: boolean;
  bottom_panel_collapsed: boolean;
  left_panel_tab: string;
  right_panel_tab: string;
  bottom_panel_tab: string;
  center_layout: CenterLayout;
}

/**
 * Tool configuration - matches backend ToolConfig
 */
export interface ToolConfig {
  enabled_tools: string[];
  tool_settings: Record<string, unknown>;
  default_hash_algorithms: string[];
  auto_hash: boolean;
  auto_verify: boolean;
  default_export_format: string;
  show_hex_viewer: boolean;
  show_metadata: boolean;
}

/**
 * Filter preset - matches backend FilterPreset
 */
export interface FilterPreset {
  id: string;
  name: string;
  description: string;
  file_types: string[];
  extensions: string[];
  size_range: [number, number] | null;
  date_range: [string, string] | null;
  search_terms: string[];
  include_hidden: boolean;
  include_system: boolean;
}

/**
 * View settings - matches backend ViewSettings
 */
export interface ViewSettings {
  theme: string;
  font_size: number;
  show_hidden_files: boolean;
  show_file_extensions: boolean;
  tree_indent: number;
  icon_size: number;
  detail_view_mode: string;
  thumbnail_size: number;
}

/**
 * Quick action - matches backend QuickAction
 */
export interface QuickAction {
  id: string;
  name: string;
  icon: string;
  command: string;
  shortcut: string | null;
}

/**
 * Complete workspace profile - matches backend WorkspaceProfile
 */
export interface WorkspaceProfile {
  id: string;
  name: string;
  profile_type: ProfileType;
  description: string;
  created_at: string;
  last_used: string;
  usage_count: number;
  layout: LayoutConfig;
  tools: ToolConfig;
  filters: FilterPreset[];
  view_settings: ViewSettings;
  quick_actions: QuickAction[];
  shortcuts: Record<string, string>;
  metadata: Record<string, unknown>;
}

/**
 * Profile summary for listing - matches backend ProfileSummary
 */
export interface ProfileSummary {
  id: string;
  name: string;
  profile_type: ProfileType;
  description: string;
  last_used: string;
  usage_count: number;
  is_active: boolean;
  is_default: boolean;
}

// =============================================================================
// Hook Implementation
// =============================================================================

/**
 * Hook for workspace profile management
 * 
 * Provides CRUD operations for workspace profiles including:
 * - List, get, create, update, delete profiles
 * - Clone profiles for customization
 * - Import/export profiles as JSON
 * - Switch active profile
 * 
 * @example
 * ```tsx
 * const profiles = useWorkspaceProfiles();
 * 
 * onMount(() => profiles.listProfiles());
 * 
 * const handleSwitch = async (id: string) => {
 *   await profiles.setActiveProfile(id);
 * };
 * ```
 */
export function useWorkspaceProfiles() {
  const [profiles, setProfiles] = createSignal<ProfileSummary[]>([]);
  const [currentProfile, setCurrentProfile] = createSignal<WorkspaceProfile | null>(null);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  // =========================================================================
  // Profile Listing & Retrieval
  // =========================================================================

  /**
   * List all available profiles
   */
  const listProfiles = async (): Promise<ProfileSummary[]> => {
    try {
      setLoading(true);
      setError(null);
      const result = await invoke<ProfileSummary[]>("profile_list");
      setProfiles(result);
      return result;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      log.error("Failed to list profiles:", err);
      return [];
    } finally {
      setLoading(false);
    }
  };

  /**
   * Get a profile by ID (uses profile_get command)
   */
  const getProfile = async (id: string): Promise<WorkspaceProfile | null> => {
    try {
      setLoading(true);
      setError(null);
      const result = await invoke<WorkspaceProfile>("profile_get", { id });
      setCurrentProfile(result);
      return result;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      log.error("Failed to get profile:", err);
      return null;
    } finally {
      setLoading(false);
    }
  };

  /**
   * Get the currently active profile
   */
  const getActiveProfile = async (): Promise<WorkspaceProfile | null> => {
    try {
      setLoading(true);
      setError(null);
      const result = await invoke<WorkspaceProfile>("profile_get_active");
      setCurrentProfile(result);
      return result;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      log.error("Failed to get active profile:", err);
      return null;
    } finally {
      setLoading(false);
    }
  };

  // =========================================================================
  // Profile Management
  // =========================================================================

  /**
   * Set the active profile (uses profile_set_active command)
   */
  const setActiveProfile = async (id: string): Promise<boolean> => {
    try {
      setLoading(true);
      setError(null);
      await invoke<void>("profile_set_active", { id });
      // Refresh current profile and list
      await getActiveProfile();
      await listProfiles();
      return true;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      log.error("Failed to set active profile:", err);
      return false;
    } finally {
      setLoading(false);
    }
  };

  /**
   * Add a new profile (uses profile_add command)
   */
  const addProfile = async (profile: WorkspaceProfile): Promise<boolean> => {
    try {
      setLoading(true);
      setError(null);
      await invoke<void>("profile_add", { profile });
      await listProfiles();
      return true;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      log.error("Failed to add profile:", err);
      return false;
    } finally {
      setLoading(false);
    }
  };

  /**
   * Update an existing profile (uses profile_update command)
   */
  const updateProfile = async (profile: WorkspaceProfile): Promise<boolean> => {
    try {
      setLoading(true);
      setError(null);
      await invoke<void>("profile_update", { profile });
      await listProfiles();
      if (currentProfile()?.id === profile.id) {
        setCurrentProfile(profile);
      }
      return true;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      log.error("Failed to update profile:", err);
      return false;
    } finally {
      setLoading(false);
    }
  };

  /**
   * Delete a profile (uses profile_delete command)
   */
  const deleteProfile = async (id: string): Promise<boolean> => {
    try {
      setLoading(true);
      setError(null);
      await invoke<void>("profile_delete", { id });
      await listProfiles();
      if (currentProfile()?.id === id) {
        setCurrentProfile(null);
      }
      return true;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      log.error("Failed to delete profile:", err);
      return false;
    } finally {
      setLoading(false);
    }
  };

  /**
   * Clone a profile with a new name (uses profile_clone command)
   */
  const cloneProfile = async (sourceId: string, newName: string): Promise<string | null> => {
    try {
      setLoading(true);
      setError(null);
      const newId = await invoke<string>("profile_clone", { 
        source_id: sourceId, 
        new_name: newName 
      });
      await listProfiles();
      return newId;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      log.error("Failed to clone profile:", err);
      return null;
    } finally {
      setLoading(false);
    }
  };

  // =========================================================================
  // Import/Export
  // =========================================================================

  /**
   * Export a profile to JSON string
   */
  const exportProfile = async (id: string): Promise<string | null> => {
    try {
      setLoading(true);
      setError(null);
      const json = await invoke<string>("profile_export", { id });
      return json;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      log.error("Failed to export profile:", err);
      return null;
    } finally {
      setLoading(false);
    }
  };

  /**
   * Import a profile from JSON string
   */
  const importProfile = async (json: string): Promise<string | null> => {
    try {
      setLoading(true);
      setError(null);
      const id = await invoke<string>("profile_import", { json });
      await listProfiles();
      return id;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      log.error("Failed to import profile:", err);
      return null;
    } finally {
      setLoading(false);
    }
  };

  // =========================================================================
  // Convenience Methods
  // =========================================================================

  /**
   * Get profiles filtered by type
   */
  const getProfilesByType = (profileType: ProfileType): ProfileSummary[] => {
    return profiles().filter((p) => p.profile_type === profileType);
  };

  /**
   * Get the most recently used profile of a specific type
   */
  const getMostRecentByType = (profileType: ProfileType): ProfileSummary | null => {
    const matching = getProfilesByType(profileType);
    if (matching.length === 0) return null;
    // Sort by last_used descending
    const sorted = [...matching].sort((a, b) => b.last_used.localeCompare(a.last_used));
    return sorted[0];
  };

  /**
   * Create a default profile structure for a new custom profile
   */
  const createDefaultProfile = (name: string, description: string = ""): WorkspaceProfile => {
    const now = new Date().toISOString();
    return {
      id: `custom_${Date.now()}`,
      name,
      profile_type: "Custom",
      description,
      created_at: now,
      last_used: now,
      usage_count: 0,
      layout: {
        left_panel_width: 300,
        right_panel_width: 350,
        bottom_panel_height: 200,
        left_panel_collapsed: false,
        right_panel_collapsed: false,
        bottom_panel_collapsed: true,
        left_panel_tab: "evidence",
        right_panel_tab: "details",
        bottom_panel_tab: "activity",
        center_layout: "Single",
      },
      tools: {
        enabled_tools: ["hash", "viewer", "bookmarks", "notes", "search"],
        tool_settings: {},
        default_hash_algorithms: ["SHA-256", "MD5"],
        auto_hash: false,
        auto_verify: true,
        default_export_format: "json",
        show_hex_viewer: true,
        show_metadata: true,
      },
      filters: [],
      view_settings: {
        theme: "auto",
        font_size: 14,
        show_hidden_files: true,
        show_file_extensions: true,
        tree_indent: 16,
        icon_size: 20,
        detail_view_mode: "list",
        thumbnail_size: 128,
      },
      quick_actions: [],
      shortcuts: {
        save: "Ctrl+S",
        search: "Ctrl+F",
        hash: "Ctrl+H",
      },
      metadata: {},
    };
  };

  // =========================================================================
  // Legacy Compatibility Methods (deprecated, use new names)
  // =========================================================================

  /** @deprecated Use getProfile instead */
  const loadProfile = getProfile;
  
  /** @deprecated Use setActiveProfile instead */
  const applyProfile = setActiveProfile;
  
  /** @deprecated Use addProfile with a full profile object */
  const saveProfile = async (
    name: string,
    profileType: ProfileType,
    description: string = ""
  ): Promise<string | null> => {
    const profile = createDefaultProfile(name, description);
    profile.profile_type = profileType;
    const success = await addProfile(profile);
    return success ? profile.id : null;
  };

  // =========================================================================
  // Return API
  // =========================================================================

  return {
    // State
    profiles,
    currentProfile,
    loading,
    error,

    // Profile retrieval
    listProfiles,
    getProfile,
    getActiveProfile,

    // Profile management
    setActiveProfile,
    addProfile,
    updateProfile,
    deleteProfile,
    cloneProfile,

    // Import/export
    exportProfile,
    importProfile,

    // Convenience
    getProfilesByType,
    getMostRecentByType,
    createDefaultProfile,

    // Legacy compatibility (deprecated)
    loadProfile,
    applyProfile,
    saveProfile,
  };
}
