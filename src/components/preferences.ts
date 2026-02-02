// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Preferences Types and Hook
 * 
 * Extracted to enable proper code-splitting of SettingsPanel component.
 * Uses @solid-primitives/storage for reactive localStorage handling.
 */

import { createSignal } from "solid-js";
import { makePersisted } from "@solid-primitives/storage";
import type { HashAlgorithmName } from "../types/hash";

// ============================================================================
// Types
// ============================================================================

export type Theme = "dark" | "light" | "light-macos" | "light-windows" | "midnight" | "system";
export type TreeDensity = "compact" | "comfortable" | "spacious";
export type HashAlgorithm = HashAlgorithmName; // Re-export centralized hash type
export type AccentColor = "cyan" | "blue" | "green" | "purple" | "orange" | "red";
export type IconSet = "outlined" | "solid" | "mini";
export type SidebarPosition = "left" | "right";
export type ExportFormat = "csv" | "json" | "pdf" | "html" | "xml";
export type ViewMode = "hex" | "text" | "preview" | "auto";
export type SortOrder = "name" | "date" | "size" | "type";
export type DateFormat = "iso" | "us" | "eu" | "relative";
export type LogLevel = "error" | "warn" | "info" | "debug";
export type HashVerificationMode = "any" | "same-algo" | "multiple";
export type ReportTemplate = "standard" | "detailed" | "summary" | "custom";
export type ActivityGrouping = "none" | "status" | "type";
export type ActivitySortOrder = "newest" | "oldest" | "name" | "progress";

export interface AppPreferences {
  // =========================================================================
  // Appearance
  // =========================================================================
  theme: Theme;
  accentColor: AccentColor;
  treeDensity: TreeDensity;
  showLineNumbers: boolean;
  fontSize: number; // 12-18
  animationsEnabled: boolean;
  iconSet: IconSet;
  sidebarPosition: SidebarPosition;
  showStatusBar: boolean;
  
  // =========================================================================
  // Activity Display (Export/Archive Progress)
  // =========================================================================
  activityShowSpeed: boolean;
  activityShowETA: boolean;
  activityShowCurrentFile: boolean;
  activityShowFileCount: boolean;
  activityShowCompressionRatio: boolean;
  activityShowThreadCount: boolean;
  activityColorCodedSpeed: boolean;
  activityPulseAnimation: boolean;
  activityGrouping: ActivityGrouping;
  activitySortOrder: ActivitySortOrder;
  activityAutoCollapse: boolean;
  activityMaxVisible: number;
  
  // =========================================================================
  // Defaults
  // =========================================================================
  defaultHashAlgorithm: HashAlgorithm;
  defaultExportFormat: ExportFormat;
  defaultViewMode: ViewMode;
  defaultSortOrder: SortOrder;
  dateFormat: DateFormat;
  autoExpandTree: boolean;
  showHiddenFiles: boolean;
  showFileSizes: boolean;
  showFileExtensions: boolean;
  rememberLastPath: boolean;
  caseSensitiveSearch: boolean;
  
  // =========================================================================
  // Behavior
  // =========================================================================
  confirmBeforeDelete: boolean;
  confirmBeforeExport: boolean;
  confirmBeforeHash: boolean;
  autoSaveProject: boolean;
  autoSaveIntervalMs: number;
  autoVerifyHashes: boolean;
  copyHashToClipboard: boolean;
  warnOnLargeContainers: boolean;
  largeContainerThresholdGb: number;
  enableSounds: boolean;
  enableNotifications: boolean;
  logLevel: LogLevel;
  
  // =========================================================================
  // Performance
  // =========================================================================
  lazyLoadThreshold: number;
  maxConcurrentOperations: number;
  useHardwareAcceleration: boolean;
  cacheSizeMb: number;
  maxPreviewSizeMb: number;
  chunkSizeKb: number;
  enableMmap: boolean;
  workerThreads: number;
  
  // =========================================================================
  // Security
  // =========================================================================
  clearClipboardOnClose: boolean;
  auditLogging: boolean;
  hashVerificationMode: HashVerificationMode;
  
  // =========================================================================
  // Paths
  // =========================================================================
  defaultEvidencePath: string;
  defaultExportPath: string;
  tempFolderPath: string;
  recentFilesCount: number;
  
  // =========================================================================
  // Reports
  // =========================================================================
  defaultReportTemplate: ReportTemplate;
  includeHashesInReports: boolean;
  includeTimestampsInReports: boolean;
  includeMetadataInReports: boolean;
  reportLogoPath: string;
  examinerName: string;
  organizationName: string;
  caseNumberPrefix: string;
  
  // =========================================================================
  // Keyboard Shortcuts (customizable)
  // =========================================================================
  shortcuts: Record<string, string>;
}

export const DEFAULT_PREFERENCES: AppPreferences = {
  // Appearance
  theme: "dark",
  accentColor: "cyan",
  treeDensity: "comfortable",
  showLineNumbers: true,
  fontSize: 14,
  animationsEnabled: true,
  iconSet: "outlined",
  sidebarPosition: "left",
  showStatusBar: true,
  
  // Activity Display
  activityShowSpeed: true,
  activityShowETA: true,
  activityShowCurrentFile: true,
  activityShowFileCount: true,
  activityShowCompressionRatio: true,
  activityShowThreadCount: false,
  activityColorCodedSpeed: true,
  activityPulseAnimation: true,
  activityGrouping: "none",
  activitySortOrder: "newest",
  activityAutoCollapse: false,
  activityMaxVisible: 20,
  
  // Defaults
  defaultHashAlgorithm: "SHA-256", // Use canonical hash algorithm name
  defaultExportFormat: "csv",
  defaultViewMode: "auto",
  defaultSortOrder: "name",
  dateFormat: "iso",
  autoExpandTree: false,
  showHiddenFiles: false,
  showFileSizes: true,
  showFileExtensions: true,
  rememberLastPath: true,
  caseSensitiveSearch: false,
  
  // Behavior
  confirmBeforeDelete: true,
  confirmBeforeExport: false,
  confirmBeforeHash: false,
  autoSaveProject: true,
  autoSaveIntervalMs: 60000,
  autoVerifyHashes: false,
  copyHashToClipboard: true,
  warnOnLargeContainers: true,
  largeContainerThresholdGb: 50,
  enableSounds: false,
  enableNotifications: true,
  logLevel: "info",
  
  // Performance
  lazyLoadThreshold: 100,
  maxConcurrentOperations: 4,
  useHardwareAcceleration: true,
  cacheSizeMb: 256,
  maxPreviewSizeMb: 10,
  chunkSizeKb: 1024,
  enableMmap: true,
  workerThreads: 4,
  
  // Security
  clearClipboardOnClose: false,
  auditLogging: true,
  hashVerificationMode: "same-algo",
  
  // Paths
  defaultEvidencePath: "",
  defaultExportPath: "",
  tempFolderPath: "",
  recentFilesCount: 10,
  
  // Reports
  defaultReportTemplate: "standard",
  includeHashesInReports: true,
  includeTimestampsInReports: true,
  includeMetadataInReports: true,
  reportLogoPath: "",
  examinerName: "",
  organizationName: "",
  caseNumberPrefix: "",
  
  // Shortcuts
  shortcuts: {
    "openCommandPalette": "Meta+k",
    "showShortcuts": "?",
    "openFile": "Meta+o",
    "closeModal": "Escape",
    "save": "Meta+s",
    "undo": "Meta+z",
    "redo": "Meta+Shift+z",
    "search": "Meta+f",
    "settings": "Meta+,",
  },
};

const STORAGE_KEY = "ffx-preferences";

// ============================================================================
// Preferences Hook - Using @solid-primitives/storage
// ============================================================================

export function createPreferences() {
  // Use makePersisted for automatic localStorage sync with reactivity
  const [preferences, setPreferences] = makePersisted(
    createSignal<AppPreferences>(DEFAULT_PREFERENCES),
    { name: STORAGE_KEY }
  );
  const [isDirty, setIsDirty] = createSignal(false);

  const updatePreference = <K extends keyof AppPreferences>(
    key: K,
    value: AppPreferences[K]
  ) => {
    setPreferences(prev => ({ ...prev, [key]: value }));
    setIsDirty(true);
    // Auto-save is handled by makePersisted
  };

  const updateShortcut = (action: string, shortcut: string) => {
    setPreferences(prev => ({
      ...prev,
      shortcuts: { ...prev.shortcuts, [action]: shortcut },
    }));
    setIsDirty(true);
    // Auto-save is handled by makePersisted
  };

  const resetToDefaults = () => {
    setPreferences(DEFAULT_PREFERENCES);
    setIsDirty(true);
    // makePersisted will update localStorage automatically
  };

  return {
    preferences,
    isDirty,
    updatePreference,
    updateShortcut,
    resetToDefaults,
  };
}

// ============================================================================
// Singleton Hook for Global Access
// ============================================================================
let preferencesInstance: ReturnType<typeof createPreferences> | null = null;

export function usePreferences() {
  if (!preferencesInstance) {
    preferencesInstance = createPreferences();
  }
  return preferencesInstance;
}

// ============================================================================
// Utility: Get single preference value (for hooks that need initial values)
// ============================================================================
export function getPreference<K extends keyof AppPreferences>(key: K): AppPreferences[K] {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      const parsed = JSON.parse(stored);
      if (key in parsed) {
        return parsed[key];
      }
    }
  } catch {
    // Ignore errors, return default
  }
  return DEFAULT_PREFERENCES[key];
}

// ============================================================================
// Utility: Last Path Persistence - Using @solid-primitives/storage
// ============================================================================
const LAST_PATH_KEY = "ffx-last-paths";

type PathType = "evidence" | "export" | "project" | "general";

interface LastPaths {
  evidence?: string;
  export?: string;
  project?: string;
  general?: string;
}

/**
 * Get the last used path for a specific dialog type.
 * Returns the remembered path if rememberLastPath is enabled, otherwise the default.
 * Note: Using manual localStorage here since paths are not reactive state.
 */
export function getLastPath(type: PathType): string | undefined {
  if (!getPreference("rememberLastPath")) {
    // Return default paths from preferences if not remembering
    switch (type) {
      case "evidence": return getPreference("defaultEvidencePath") || undefined;
      case "export": return getPreference("defaultExportPath") || undefined;
      default: return undefined;
    }
  }
  
  try {
    const stored = localStorage.getItem(LAST_PATH_KEY);
    if (stored) {
      const paths: LastPaths = JSON.parse(stored);
      return paths[type];
    }
  } catch {
    // Ignore errors
  }
  
  // Fall back to default paths from preferences
  switch (type) {
    case "evidence": return getPreference("defaultEvidencePath") || undefined;
    case "export": return getPreference("defaultExportPath") || undefined;
    default: return undefined;
  }
}

/**
 * Save the last used path for a specific dialog type.
 * Only saves if rememberLastPath is enabled.
 */
export function setLastPath(type: PathType, path: string): void {
  if (!getPreference("rememberLastPath")) {
    return;
  }
  
  try {
    const stored = localStorage.getItem(LAST_PATH_KEY);
    const paths: LastPaths = stored ? JSON.parse(stored) : {};
    paths[type] = path;
    localStorage.setItem(LAST_PATH_KEY, JSON.stringify(paths));
  } catch (e) {
    console.warn("Failed to save last path:", e);
  }
}

// ============================================================================
// Utility: Recent Projects Persistence
// ============================================================================
const RECENT_PROJECTS_KEY = "ffx-recent-projects";

export interface RecentProject {
  path: string;
  name: string;
  lastOpened: string; // ISO timestamp
}

/**
 * Get list of recent projects.
 * Returns most recent first, limited by recentFilesCount preference.
 */
export function getRecentProjects(): RecentProject[] {
  try {
    const stored = localStorage.getItem(RECENT_PROJECTS_KEY);
    if (stored) {
      const projects: RecentProject[] = JSON.parse(stored);
      const maxCount = getPreference("recentFilesCount");
      console.log(`[DEBUG] getRecentProjects: Found ${projects.length} projects, returning max ${maxCount}`);
      return projects.slice(0, maxCount);
    }
    console.log("[DEBUG] getRecentProjects: No stored projects found");
  } catch (e) {
    console.log(`[DEBUG] getRecentProjects: Error - ${e}`);
  }
  return [];
}

/**
 * Add a project to the recent projects list.
 * If the project already exists, it updates the timestamp and moves it to the top.
 */
export function addRecentProject(path: string, name: string): void {
  console.log(`[DEBUG] addRecentProject: path=${path}, name=${name}`);
  try {
    const maxCount = getPreference("recentFilesCount");
    const stored = localStorage.getItem(RECENT_PROJECTS_KEY);
    let projects: RecentProject[] = stored ? JSON.parse(stored) : [];
    
    // Remove existing entry if present
    projects = projects.filter(p => p.path !== path);
    
    // Add to front with current timestamp
    projects.unshift({
      path,
      name,
      lastOpened: new Date().toISOString(),
    });
    
    // Trim to max count
    projects = projects.slice(0, maxCount);
    
    localStorage.setItem(RECENT_PROJECTS_KEY, JSON.stringify(projects));
    console.log(`[DEBUG] addRecentProject: Saved ${projects.length} projects`);
  } catch (e) {
    console.warn("Failed to save recent project:", e);
  }
}

/**
 * Remove a project from the recent projects list.
 * Useful when a project file no longer exists.
 */
export function removeRecentProject(path: string): void {
  try {
    const stored = localStorage.getItem(RECENT_PROJECTS_KEY);
    if (stored) {
      let projects: RecentProject[] = JSON.parse(stored);
      projects = projects.filter(p => p.path !== path);
      localStorage.setItem(RECENT_PROJECTS_KEY, JSON.stringify(projects));
    }
  } catch (e) {
    console.warn("Failed to remove recent project:", e);
  }
}

/**
 * Clear all recent projects.
 */
export function clearRecentProjects(): void {
  try {
    localStorage.removeItem(RECENT_PROJECTS_KEY);
  } catch (e) {
    console.warn("Failed to clear recent projects:", e);
  }
}
