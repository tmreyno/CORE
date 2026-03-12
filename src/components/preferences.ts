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
import type { ReportPreset } from "./report/constants";
export type { ReportPreset } from "./report/constants";
import type { HashAlgorithmName } from "../types/hash";
import { logger } from "../utils/logger";

const log = logger.scope("Preferences");

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
export type ActivityGrouping = "none" | "status" | "type";
export type ActivitySortOrder = "newest" | "oldest" | "name" | "progress";

// ============================================================================
// Workspace Modes — feature module show/hide system
// ============================================================================

/**
 * Feature modules that can be individually enabled/disabled per workspace mode.
 * Each module controls visibility of related UI sections (sidebar tabs, menu items,
 * toolbar sections, center pane tabs, quick actions).
 */
export type FeatureModule =
  | "forensicExplorer"      // Evidence tree, hash verification, hex/text viewers, VFS, container ops
  | "evidenceCollection"    // Evidence collection forms, COC management, linked data tree
  | "documentReview"        // Case documents panel, document viewers
  | "searchAnalysis"        // File deduplication, processed database parsers
  | "reportExport"          // Report wizard, export panel, L01/E01/7z creation, merge projects
  | "caseManagement";       // Dashboard, activity timeline, project management

/** Metadata for a feature module — used in settings UI */
export interface FeatureModuleInfo {
  id: FeatureModule;
  name: string;
  description: string;
  /** Icon key matching QuickActionsBar ICON_MAP */
  icon: string;
}

/** All available feature modules with their metadata */
export const FEATURE_MODULES: FeatureModuleInfo[] = [
  {
    id: "forensicExplorer",
    name: "Forensic Explorer",
    description: "Evidence containers, hash verification, hex/text viewers, virtual filesystem",
    icon: "evidence",
  },
  {
    id: "evidenceCollection",
    name: "Evidence Collection & COC",
    description: "On-site evidence collection forms, chain of custody records, linked data",
    icon: "extract",
  },
  {
    id: "documentReview",
    name: "Document Review",
    description: "Case documents panel and document viewers",
    icon: "document",
  },
  {
    id: "searchAnalysis",
    name: "Search & Analysis",
    description: "File deduplication and processed database parsers (AXIOM, Cellebrite, Autopsy)",
    icon: "search",
  },
  {
    id: "reportExport",
    name: "Report & Export",
    description: "Report wizard, forensic image export (E01/L01/7z), project merge",
    icon: "export",
  },
  {
    id: "caseManagement",
    name: "Case Management",
    description: "Dashboard, activity timeline, session tracking, project management",
    icon: "chart",
  },
];

/** A workspace mode preset — named combination of enabled modules */
export interface WorkspaceModePreset {
  id: string;
  name: string;
  description: string;
  modules: FeatureModule[];
  /** Whether this is the user's custom mode (editable per-module) */
  isCustom?: boolean;
}

/** All built-in workspace mode presets */
export const WORKSPACE_PRESETS: WorkspaceModePreset[] = [
  {
    id: "full",
    name: "Full Suite",
    description: "All features enabled — complete forensic workflow",
    modules: ["forensicExplorer", "evidenceCollection", "documentReview", "searchAnalysis", "reportExport", "caseManagement"],
  },
  {
    id: "forensic",
    name: "Forensic Explorer",
    description: "Focus on evidence container browsing, hash verification, and file viewing",
    modules: ["forensicExplorer"],
  },
  {
    id: "collection",
    name: "Evidence Collection & COC",
    description: "On-site evidence intake, collection forms, and chain of custody",
    modules: ["forensicExplorer", "evidenceCollection"],
  },
  {
    id: "review",
    name: "Document Review",
    description: "Case document review with search, bookmarks, and processed databases",
    modules: ["forensicExplorer", "documentReview", "searchAnalysis"],
  },
  {
    id: "analysis",
    name: "Search & Analysis",
    description: "Full-text search, deduplication, and processed database analysis",
    modules: ["forensicExplorer", "searchAnalysis", "caseManagement"],
  },
  {
    id: "reporting",
    name: "Report & Export",
    description: "Report generation, forensic image creation, and evidence export",
    modules: ["forensicExplorer", "evidenceCollection", "reportExport"],
  },
  {
    id: "custom",
    name: "Custom",
    description: "Choose exactly which features to enable",
    modules: [], // Populated from customEnabledModules preference
    isCustom: true,
  },
];

/** Get a workspace preset by ID, or the "full" preset as fallback */
export function getWorkspacePreset(id: string): WorkspaceModePreset {
  return WORKSPACE_PRESETS.find(p => p.id === id) ?? WORKSPACE_PRESETS[0];
}

// ============================================================================
// User Profile — bundles examiner info + branding + report defaults
// ============================================================================

export interface UserProfile {
  id: string;
  name: string;
  title: string;
  organization: string;
  badgeNumber: string;
  email: string;
  phone: string;
  certifications: string[];
  agency: string;
  logoPath: string;
  caseNumberPrefix: string;
  /** Optional default report preset for this user */
  defaultReportPreset?: ReportPreset;
}

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
  defaultReportPreset: ReportPreset;
  includeHashesInReports: boolean;
  includeTimestampsInReports: boolean;
  includeMetadataInReports: boolean;
  reportLogoPath: string;
  examinerName: string;
  examinerTitle: string;
  examinerEmail: string;
  examinerPhone: string;
  examinerBadge: string;
  examinerCertifications: string[];
  organizationName: string;
  defaultAgency: string;
  caseNumberPrefix: string;
  
  // =========================================================================
  // Report Numbering
  // =========================================================================
  /** Per-report-type prefix for auto-generated report numbers */
  reportNumberPrefixes: Record<string, string>;
  /** Pattern for COC numbering. Tokens: {case}, {agency}, {year}, {seq} */
  cocNumberPattern: string;
  /** Pattern for evidence item numbering. Tokens: {case}, {seq} */
  evidenceItemPattern: string;
  /** Whether to include year in auto-generated report numbers */
  reportNumberIncludeYear: boolean;
  /** Minimum digits for sequential portion of report numbers */
  reportNumberSeqDigits: number;
  /** Per-type sequential counters (persisted for uniqueness across sessions) */
  reportNumberCounters: Record<string, number>;
  
  // =========================================================================
  // Keyboard Shortcuts (customizable)
  // =========================================================================
  shortcuts: Record<string, string>;

  // =========================================================================
  // User Profiles
  // =========================================================================
  /** Saved user profiles (examiner info + branding bundled together) */
  userProfiles: UserProfile[];
  /** ID of the default/active user profile (empty = none selected) */
  defaultUserProfileId: string;
  /** Whether to show user confirmation modal on project open/create */
  confirmUserOnProjectOpen: boolean;
  
  // =========================================================================
  // Workspace Modes
  // =========================================================================
  /** Active workspace mode preset ID (e.g., "full", "forensic", "custom") */
  workspaceMode: string;
  /** Modules enabled when workspaceMode is "custom" */
  customEnabledModules: FeatureModule[];
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
  // Tuned for modern multi-core systems with NVMe storage
  lazyLoadThreshold: 200,
  maxConcurrentOperations: 0, // 0 = auto-detect (use all CPU cores)
  useHardwareAcceleration: true,
  cacheSizeMb: 512,
  maxPreviewSizeMb: 50, // Matches backend 50MB email/text limit
  chunkSizeKb: 2048, // 2MB chunks for better I/O throughput
  enableMmap: true,
  workerThreads: 0, // 0 = auto-detect (use all CPU cores)
  
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
  defaultReportPreset: "law_enforcement",
  includeHashesInReports: true,
  includeTimestampsInReports: true,
  includeMetadataInReports: true,
  reportLogoPath: "",
  examinerName: "",
  examinerTitle: "",
  examinerEmail: "",
  examinerPhone: "",
  examinerBadge: "",
  examinerCertifications: [],
  organizationName: "",
  defaultAgency: "",
  caseNumberPrefix: "",
  
  // Report Numbering
  reportNumberPrefixes: {
    forensic_examination: "FR",
    chain_of_custody: "COC",
    investigative_activity: "IAR",
    evidence_collection: "EC",
    user_activity: "UA",
    timeline: "TL",
  },
  cocNumberPattern: "{case}-COC-{seq}",
  evidenceItemPattern: "{case}-EV-{seq}",
  reportNumberIncludeYear: true,
  reportNumberSeqDigits: 4,
  reportNumberCounters: {},
  
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

  // User Profiles
  userProfiles: [],
  defaultUserProfileId: "",
  confirmUserOnProjectOpen: true,

  // Workspace Modes
  workspaceMode: "full",
  customEnabledModules: ["forensicExplorer", "evidenceCollection", "documentReview", "searchAnalysis", "reportExport", "caseManagement"],
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
// User Profile Helpers
// ============================================================================

/** Generate a unique ID for a new user profile */
export function generateProfileId(): string {
  return `profile-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
}

/** Create a blank UserProfile with a generated ID */
export function createEmptyProfile(name?: string): UserProfile {
  return {
    id: generateProfileId(),
    name: name || "",
    title: "",
    organization: "",
    badgeNumber: "",
    email: "",
    phone: "",
    certifications: [],
    agency: "",
    logoPath: "",
    caseNumberPrefix: "",
  };
}

/** Get the active/default user profile from preferences, or undefined */
export function getActiveUserProfile(): UserProfile | undefined {
  const profiles = getPreference("userProfiles");
  const defaultId = getPreference("defaultUserProfileId");
  if (!defaultId || !profiles.length) return undefined;
  return profiles.find(p => p.id === defaultId);
}

/** Apply a user profile's fields to the flat examiner preferences */
export function applyProfileToPreferences(
  profile: UserProfile,
  updatePreference: <K extends keyof AppPreferences>(key: K, value: AppPreferences[K]) => void,
): void {
  updatePreference("examinerName", profile.name);
  updatePreference("examinerTitle", profile.title);
  updatePreference("organizationName", profile.organization);
  updatePreference("examinerBadge", profile.badgeNumber);
  updatePreference("examinerEmail", profile.email);
  updatePreference("examinerPhone", profile.phone);
  updatePreference("examinerCertifications", profile.certifications);
  updatePreference("defaultAgency", profile.agency);
  updatePreference("reportLogoPath", profile.logoPath);
  updatePreference("caseNumberPrefix", profile.caseNumberPrefix);
  if (profile.defaultReportPreset) {
    updatePreference("defaultReportPreset", profile.defaultReportPreset);
  }
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
    log.warn("Failed to save last path:", e);
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
      return projects.slice(0, maxCount);
    }
  } catch (e) {
    // Ignore parse errors for recent projects
  }
  return [];
}

/**
 * Add a project to the recent projects list.
 * If the project already exists, it updates the timestamp and moves it to the top.
 */
export function addRecentProject(path: string, name: string): void {
  log.debug(`addRecentProject: path=${path}, name=${name}`);
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
    log.debug(`addRecentProject: Saved ${projects.length} projects`);
  } catch (e) {
    log.warn("Failed to save recent project:", e);
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
    log.warn("Failed to remove recent project:", e);
  }
}

/**
 * Clear all recent projects.
 */
export function clearRecentProjects(): void {
  try {
    localStorage.removeItem(RECENT_PROJECTS_KEY);
  } catch (e) {
    log.warn("Failed to clear recent projects:", e);
  }
}
