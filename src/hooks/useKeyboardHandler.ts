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
 * - Cmd+Z: Undo
 * - Cmd+Shift+Z / Cmd+Y: Redo
 * - Cmd+S: Save
 * - Cmd+Shift+S: Save As
 * - ?: Shortcuts help
 * - Escape: Close modals
 */

import { type Accessor, type Setter } from "solid-js";
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
  showCommandPalette: Accessor<boolean>;
  showShortcutsModal: Accessor<boolean>;
  
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
    
    // Cmd+K: Command palette
    if (meta && e.key === "k") {
      e.preventDefault();
      setShowCommandPalette(v => !v);
      return;
    }
    
    // Cmd+,: Settings
    if (meta && e.key === ",") {
      e.preventDefault();
      setShowSettingsPanel(true);
      return;
    }
    
    // Cmd+F: Search
    if (meta && e.key === "f") {
      e.preventDefault();
      setShowSearchPanel(true);
      return;
    }
    
    // Ctrl+Shift+P: Performance panel (dev)
    if (e.ctrlKey && e.shiftKey && e.key === "P") {
      e.preventDefault();
      setShowPerformancePanel(v => !v);
      return;
    }
    
    // ?: Shortcuts help (when not in input)
    if (e.key === "?" && !inInput) {
      e.preventDefault();
      setShowShortcutsModal(true);
      return;
    }
    
    // Cmd+Z: Undo
    if (meta && e.key === "z" && !e.shiftKey) {
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
    if (meta && ((e.key === "z" && e.shiftKey) || e.key === "y")) {
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
    if (meta && e.key === "s" && !e.shiftKey) {
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
    if (meta && e.key === "s" && e.shiftKey) {
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
    if (e.key === "Escape") {
      if (showCommandPalette()) { setShowCommandPalette(false); return; }
      if (showShortcutsModal()) { setShowShortcutsModal(false); return; }
    }
  };
  
  // Run the handler reactively when keyboard events change
  // The effect runs automatically whenever event() changes
  handleKeyEvent();

  return {
    // Expose the event accessor for components that need raw keyboard access
    keyEvent: event,
  };
}
