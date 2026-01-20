// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Preferences Types and Hook
 * 
 * Extracted to enable proper code-splitting of SettingsPanel component.
 */

import { createSignal } from "solid-js";

// ============================================================================
// Types
// ============================================================================

export type Theme = "dark" | "light" | "light-macos" | "light-windows" | "midnight" | "system";
export type TreeDensity = "compact" | "comfortable" | "spacious";
export type HashAlgorithm = "MD5" | "SHA1" | "SHA256" | "SHA512" | "Blake3" | "XXH3";
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
  
  // Defaults
  defaultHashAlgorithm: "SHA256",
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
// Load preferences from localStorage (synchronous for initial render)
// ============================================================================
function loadStoredPreferences(): AppPreferences {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      const parsed = JSON.parse(stored);
      return { ...DEFAULT_PREFERENCES, ...parsed };
    }
  } catch (e) {
    console.warn("Failed to load preferences:", e);
  }
  return DEFAULT_PREFERENCES;
}

// ============================================================================
// Preferences Hook
// ============================================================================

export function createPreferences() {
  // Load preferences synchronously to avoid flash of default values
  const [preferences, setPreferences] = createSignal<AppPreferences>(loadStoredPreferences());
  const [isDirty, setIsDirty] = createSignal(false);

  const updatePreference = <K extends keyof AppPreferences>(
    key: K,
    value: AppPreferences[K]
  ) => {
    setPreferences(prev => ({ ...prev, [key]: value }));
    setIsDirty(true);
    
    // Auto-save to localStorage
    try {
      const current = preferences();
      localStorage.setItem(STORAGE_KEY, JSON.stringify({ ...current, [key]: value }));
    } catch (e) {
      console.warn("Failed to save preference:", e);
    }
  };

  const updateShortcut = (action: string, shortcut: string) => {
    setPreferences(prev => ({
      ...prev,
      shortcuts: { ...prev.shortcuts, [action]: shortcut },
    }));
    setIsDirty(true);
    
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(preferences()));
    } catch (e) {
      console.warn("Failed to save shortcut:", e);
    }
  };

  const resetToDefaults = () => {
    setPreferences(DEFAULT_PREFERENCES);
    setIsDirty(true);
    try {
      localStorage.removeItem(STORAGE_KEY);
    } catch (e) {
      console.warn("Failed to reset preferences:", e);
    }
  };

  return {
    preferences,
    isDirty,
    updatePreference,
    updateShortcut,
    resetToDefaults,
  };
}
