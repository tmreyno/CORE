// =============================================================================
// useGlobalKeyboard - Global keyboard shortcut handling
// =============================================================================

import { onMount, onCleanup, Accessor, Setter } from "solid-js";
import type { OpenTab, TabViewMode } from "../components";
import type { HistoryState, HistoryActions } from "./useHistory.tsx";
import { announce } from "../utils/accessibility";
import { logError } from "../utils/telemetry";

export interface KeyboardHandlerDeps {
  // Toast notifications
  toast: {
    success: (title: string, message?: string) => void;
    info: (title: string, message?: string) => void;
    error: (title: string, message?: string) => void;
  };
  
  // History state for undo/redo
  history: {
    state: HistoryState;
    actions: HistoryActions;
  };
  
  // Modal visibility setters
  setShowCommandPalette: Setter<boolean>;
  setShowSettingsPanel: Setter<boolean>;
  setShowSearchPanel: Setter<boolean>;
  setShowPerformancePanel: Setter<boolean>;
  setShowShortcutsModal: Setter<boolean>;
  
  // Modal visibility getters
  showCommandPalette: Accessor<boolean>;
  showShortcutsModal: Accessor<boolean>;
  
  // File manager state
  fileManager: {
    scanDir: Accessor<string>;
    activeFile: Accessor<{ path: string } | null>;
  };
  
  // Hash manager state
  hashManager: {
    hashHistory: Accessor<unknown[]>;
  };
  
  // Processed databases
  processedDbManager: {
    databases: Accessor<unknown[]>;
    selectedDatabase: Accessor<unknown | null>;
  };
  
  // Project manager
  projectManager: {
    saveProject: (data: unknown) => Promise<void>;
  };
  
  // UI state
  leftWidth: Accessor<number>;
  rightWidth: Accessor<number>;
  leftCollapsed: Accessor<boolean>;
  rightCollapsed: Accessor<boolean>;
  leftPanelTab: Accessor<"evidence" | "processed" | "casedocs">;
  currentViewMode: Accessor<TabViewMode>;
  openTabs: Accessor<OpenTab[]>;
}

/**
 * Hook to handle global keyboard shortcuts
 */
export function useGlobalKeyboard(deps: KeyboardHandlerDeps): void {
  onMount(() => {
    const handleGlobalKeyDown = (e: KeyboardEvent) => {
      // Cmd+K: Open command palette
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        deps.setShowCommandPalette(v => !v);
        return;
      }
      
      // Cmd+,: Open settings
      if ((e.metaKey || e.ctrlKey) && e.key === ",") {
        e.preventDefault();
        deps.setShowSettingsPanel(true);
        return;
      }
      
      // Cmd+F: Open search panel
      if ((e.metaKey || e.ctrlKey) && e.key === "f") {
        e.preventDefault();
        deps.setShowSearchPanel(true);
        return;
      }
      
      // Ctrl+Shift+P: Toggle performance panel (dev mode)
      if ((e.ctrlKey) && e.shiftKey && e.key === "P") {
        e.preventDefault();
        deps.setShowPerformancePanel(v => !v);
        return;
      }
      
      // ?: Show keyboard shortcuts (when not in input)
      if (e.key === "?" && !["INPUT", "TEXTAREA"].includes((e.target as HTMLElement)?.tagName)) {
        e.preventDefault();
        deps.setShowShortcutsModal(true);
        return;
      }
      
      // Cmd+Z: Undo
      if ((e.metaKey || e.ctrlKey) && e.key === "z" && !e.shiftKey) {
        e.preventDefault();
        if (deps.history.state.canUndo()) {
          deps.history.actions.undo();
          const desc = deps.history.state.undoDescription() || "Action undone";
          deps.toast.info("Undo", desc);
          announce(`Undo: ${desc}`);
        }
        return;
      }
      
      // Cmd+Shift+Z or Cmd+Y: Redo
      if ((e.metaKey || e.ctrlKey) && ((e.key === "z" && e.shiftKey) || e.key === "y")) {
        e.preventDefault();
        if (deps.history.state.canRedo()) {
          deps.history.actions.redo();
          const desc = deps.history.state.redoDescription() || "Action redone";
          deps.toast.info("Redo", desc);
          announce(`Redo: ${desc}`);
        }
        return;
      }
      
      // Cmd+S: Save project
      if ((e.metaKey || e.ctrlKey) && e.key === "s") {
        e.preventDefault();
        const scanDir = deps.fileManager.scanDir();
        if (scanDir) {
          deps.projectManager.saveProject({
            rootPath: scanDir,
            openTabs: deps.openTabs(),
            activeTabPath: deps.fileManager.activeFile()?.path || null,
            hashHistory: deps.hashManager.hashHistory(),
            processedDatabases: deps.processedDbManager.databases(),
            selectedProcessedDb: deps.processedDbManager.selectedDatabase(),
            uiState: {
              left_panel_width: deps.leftWidth(),
              right_panel_width: deps.rightWidth(),
              left_panel_collapsed: deps.leftCollapsed(),
              right_panel_collapsed: deps.rightCollapsed(),
              left_panel_tab: deps.leftPanelTab(),
              detail_view_mode: deps.currentViewMode(),
            },
          }).then(() => {
            deps.toast.success("Saved", "Project saved");
            announce("Project saved");
          }).catch((err) => {
            logError(err instanceof Error ? err : new Error("Save failed"), { source: "keyboard.save" });
            deps.toast.error("Save Failed", "Could not save project");
          });
        }
        return;
      }
      
      // Escape: Close modals
      if (e.key === "Escape") {
        if (deps.showCommandPalette()) {
          deps.setShowCommandPalette(false);
          return;
        }
        if (deps.showShortcutsModal()) {
          deps.setShowShortcutsModal(false);
          return;
        }
      }
    };
    
    window.addEventListener("keydown", handleGlobalKeyDown);
    onCleanup(() => window.removeEventListener("keydown", handleGlobalKeyDown));
  });
}
