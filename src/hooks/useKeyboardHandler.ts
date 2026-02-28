// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useKeyboardHandler - Manages global keyboard shortcuts using @solid-primitives/keyboard.
 * 
 * Handles:
 * - Cmd+K: Command palette
 * - Cmd+,: Settings
 * - Cmd+F: Search
 * - Cmd+P: Generate Report
 * - Cmd+Shift+N: New Project
 * - Cmd+O: Open Project
 * - Cmd+Z: Undo
 * - Cmd+Shift+Z / Cmd+Y: Redo
 * - Cmd+S: Save
 * - Cmd+Shift+S: Save As
 * - ?: Shortcuts help
 * - Escape: Close modals
 */

import { type Accessor, type Setter, createEffect } from "solid-js";
import { useKeyDownEvent } from "@solid-primitives/keyboard";
import { announce } from "../utils/accessibility";
import { logError } from "../utils/telemetry";
import type { BuildProjectOptions } from "./project/types";

// =============================================================================
// Types
// =============================================================================

export interface KeyboardHandlerDeps {
  // Modal controls - simple setters
  setShowCommandPalette: Setter<boolean>;
  setShowSettingsPanel: Setter<boolean>;
  setShowSearchPanel: Setter<boolean>;
  setShowPerformancePanel: Setter<boolean>;
  setShowShortcutsModal: Setter<boolean>;
  setShowProjectWizard?: Setter<boolean>;  // New project wizard
  setShowReportWizard?: Setter<boolean>;   // Report wizard
  showCommandPalette: Accessor<boolean>;
  showShortcutsModal: Accessor<boolean>;
  
  // Project actions
  onLoadProject?: () => void;  // Open project file picker
  onOpenDirectory?: () => void; // Open directory (shows project wizard)
  
  // History (undo/redo)
  history: {
    state: {
      canUndo: Accessor<boolean>;
      canRedo: Accessor<boolean>;
      undoDescription: Accessor<string | null>;
      redoDescription: Accessor<string | null>;
    };
    actions: {
      undo: () => void;
      redo: () => void;
    };
  };
  
  // Toast notifications
  toast: {
    success: (title: string, message?: string) => void;
    error: (title: string, message?: string) => void;
    info: (title: string, message?: string) => void;
  };
  
  // Project manager
  projectManager: {
    projectPath: Accessor<string | null>;
    saveProject: (options: BuildProjectOptions, customPath?: string) => Promise<{ success: boolean; error?: string }>;
    saveProjectAs: (options: BuildProjectOptions) => Promise<{ success: boolean; error?: string }>;
  };
  
  // Build save options function
  buildSaveOptions: () => BuildProjectOptions | null;
}

// =============================================================================
// Hook
// =============================================================================

export function useKeyboardHandler(deps: KeyboardHandlerDeps) {
  const { 
    setShowCommandPalette, 
    setShowSettingsPanel, 
    setShowSearchPanel, 
    setShowPerformancePanel, 
    setShowShortcutsModal,
    setShowProjectWizard,
    setShowReportWizard,
    onLoadProject,
    onOpenDirectory,
    showCommandPalette,
    showShortcutsModal,
    history, 
    toast, 
    projectManager, 
    buildSaveOptions 
  } = deps;
  
  // Use solid-primitives keyboard hook for reactive key event handling
  const event = useKeyDownEvent();
  
  // Helper to check if we're in an input field
  const isInInput = (e: KeyboardEvent) => 
    ["INPUT", "TEXTAREA"].includes((e.target as HTMLElement)?.tagName);
  
  // Create reactive effect that handles all keyboard shortcuts
  const handleKeyEvent = () => {
    const e = event();
    if (!e) return;
    
    const meta = e.metaKey || e.ctrlKey;
    const inInput = isInInput(e);
    
    // Normalize key to lowercase for consistent matching
    // (Shift+key produces uppercase, e.g., Shift+S = "S")
    const key = e.key.toLowerCase();
    
    // Cmd+K: Command palette
    if (meta && key === "k") {
      e.preventDefault();
      setShowCommandPalette(v => !v);
      return;
    }
    
    // Cmd+,: Settings
    if (meta && key === ",") {
      e.preventDefault();
      setShowSettingsPanel(true);
      return;
    }
    
    // Cmd+F: Search (requires open project)
    if (meta && key === "f") {
      e.preventDefault();
      if (projectManager.projectPath()) {
        setShowSearchPanel(true);
      }
      return;
    }
    
    // Cmd+P: Generate Report (requires open project)
    if (meta && !e.shiftKey && key === "p") {
      e.preventDefault();
      if (setShowReportWizard && projectManager.projectPath()) {
        setShowReportWizard(true);
        announce("Report wizard opened");
      }
      return;
    }
    
    // Cmd+Shift+N: New Project
    if (meta && e.shiftKey && key === "n") {
      e.preventDefault();
      if (setShowProjectWizard) {
        setShowProjectWizard(true);
        announce("New Project wizard opened");
      }
      return;
    }
    
    // Cmd+Shift+O: Open Directory (shows project wizard)
    if (meta && e.shiftKey && key === "o") {
      e.preventDefault();
      if (onOpenDirectory) {
        onOpenDirectory();
        announce("Opening directory...");
      }
      return;
    }
    
    // Cmd+O: Open Project
    if (meta && !e.shiftKey && key === "o") {
      e.preventDefault();
      if (onLoadProject) {
        onLoadProject();
        announce("Opening project...");
      }
      return;
    }
    
    // Ctrl+Shift+P: Performance panel (dev)
    if (e.ctrlKey && e.shiftKey && key === "p") {
      e.preventDefault();
      setShowPerformancePanel(v => !v);
      return;
    }
    
    // ?: Shortcuts help (when not in input)
    if (key === "?" && !inInput) {
      e.preventDefault();
      setShowShortcutsModal(true);
      return;
    }
    
    // Cmd+Z: Undo
    if (meta && key === "z" && !e.shiftKey) {
      e.preventDefault();
      if (history.state.canUndo()) {
        history.actions.undo();
        const desc = history.state.undoDescription() || "Action undone";
        toast.info("Undo", desc);
        announce(`Undo: ${desc}`);
      }
      return;
    }
    
    // Cmd+Shift+Z or Cmd+Y: Redo
    if (meta && ((key === "z" && e.shiftKey) || key === "y")) {
      e.preventDefault();
      if (history.state.canRedo()) {
        history.actions.redo();
        const desc = history.state.redoDescription() || "Action redone";
        toast.info("Redo", desc);
        announce(`Redo: ${desc}`);
      }
      return;
    }
    
    // Cmd+S: Save (update existing project) or Save As (if no project)
    if (meta && key === "s" && !e.shiftKey) {
      e.preventDefault();
      const options = buildSaveOptions();
      if (options) {
        // If we have an existing project, update it directly
        if (projectManager.projectPath()) {
          projectManager.saveProject(options, projectManager.projectPath()!).then((result) => {
            if (result.success) {
              toast.success("Saved", "Project updated");
              announce("Project saved");
            } else if (result.error && result.error !== "Save cancelled") {
              toast.error("Save Failed", result.error);
            }
          }).catch((err) => {
            logError(err instanceof Error ? err : new Error("Save failed"), { source: "keyboard.save" });
            toast.error("Save Failed", "Could not save project");
          });
        } else {
          // No existing project - show Save As dialog
          projectManager.saveProjectAs(options).then((result) => {
            if (result.success) {
              toast.success("Project Saved", "New project created");
              announce("Project saved");
            } else if (result.error && result.error !== "Save cancelled") {
              toast.error("Save Failed", result.error);
            }
          }).catch((err) => {
            logError(err instanceof Error ? err : new Error("Save failed"), { source: "keyboard.save" });
            toast.error("Save Failed", "Could not save project");
          });
        }
      }
      return;
    }
    
    // Cmd+Shift+S: Save As (always shows dialog)
    if (meta && key === "s" && e.shiftKey) {
      e.preventDefault();
      const options = buildSaveOptions();
      if (options) {
        projectManager.saveProjectAs(options).then((result) => {
          if (result.success) {
            toast.success("Project Saved", "New project created");
            announce("Project saved");
          } else if (result.error && result.error !== "Save cancelled") {
            toast.error("Save Failed", result.error);
          }
        }).catch((err) => {
          logError(err instanceof Error ? err : new Error("Save As failed"), { source: "keyboard.saveAs" });
          toast.error("Save Failed", "Could not save project");
        });
      } else {
        toast.error("No Evidence", "Open an evidence directory first");
      }
      return;
    }
    
    // Escape: Close modals
    if (key === "escape") {
      if (showCommandPalette()) { setShowCommandPalette(false); return; }
      if (showShortcutsModal()) { setShowShortcutsModal(false); return; }
    }
  };
  
  // Run the handler reactively when keyboard events change
  // IMPORTANT: Must be wrapped in createEffect to react to event() changes
  createEffect(() => {
    handleKeyEvent();
  });

  return {
    // Expose the event accessor for components that need raw keyboard access
    keyEvent: event,
  };
}
