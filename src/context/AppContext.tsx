// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * AppContext provides centralized state management for the application.
 * 
 * This context consolidates:
 * - UI state (modals, panels, view modes)
 * - Core services (file manager, hash manager, project manager, etc.)
 * - Shared actions (save, load, search)
 * 
 * Components can access any state they need via useAppContext().
 */

import { createContext, useContext, type ParentComponent } from "solid-js";
import { useFileManager, useHashManager, useDatabase, useProject, useProcessedDatabases } from "../hooks";
import { useToast } from "../components";
import { useHistoryContext } from "../hooks/useHistory";
import { createPreferences } from "../components/preferences";
import { createThemeActions } from "../hooks/useTheme";

// =============================================================================
// Types
// =============================================================================

export interface AppContextValue {
  // Core Services
  toast: ReturnType<typeof useToast>;
  history: ReturnType<typeof useHistoryContext>;
  preferences: ReturnType<typeof createPreferences>;
  themeActions: ReturnType<typeof createThemeActions>;
  db: ReturnType<typeof useDatabase>;
  fileManager: ReturnType<typeof useFileManager>;
  hashManager: ReturnType<typeof useHashManager>;
  projectManager: ReturnType<typeof useProject>;
  processedDbManager: ReturnType<typeof useProcessedDatabases>;
}

// =============================================================================
// Context
// =============================================================================

const AppContext = createContext<AppContextValue>();

// =============================================================================
// Provider Component
// =============================================================================

export const AppProvider: ParentComponent = (props) => {
  // Initialize core services
  const toast = useToast();
  const history = useHistoryContext();
  const preferences = createPreferences();
  const db = useDatabase();
  const fileManager = useFileManager();
  const hashManager = useHashManager(fileManager);
  const projectManager = useProject();
  const processedDbManager = useProcessedDatabases();
  
  // Theme actions (uses preferences as single source of truth)
  const themeActions = createThemeActions(
    () => preferences.preferences().theme,
    (theme) => preferences.updatePreference("theme", theme)
  );

  const value: AppContextValue = {
    toast,
    history,
    preferences,
    themeActions,
    db,
    fileManager,
    hashManager,
    projectManager,
    processedDbManager,
  };

  return (
    <AppContext.Provider value={value}>
      {props.children}
    </AppContext.Provider>
  );
};

// =============================================================================
// Hook
// =============================================================================

export function useAppContext(): AppContextValue {
  const context = useContext(AppContext);
  if (!context) {
    throw new Error("useAppContext must be used within an AppProvider");
  }
  return context;
}

// =============================================================================
// Convenience hooks for accessing specific parts of context
// =============================================================================

export function useServices() {
  const ctx = useAppContext();
  return {
    toast: ctx.toast,
    history: ctx.history,
    preferences: ctx.preferences,
    themeActions: ctx.themeActions,
  };
}

export function useManagers() {
  const ctx = useAppContext();
  return {
    db: ctx.db,
    fileManager: ctx.fileManager,
    hashManager: ctx.hashManager,
    projectManager: ctx.projectManager,
    processedDbManager: ctx.processedDbManager,
  };
}
