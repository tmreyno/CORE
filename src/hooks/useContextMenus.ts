// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useContextMenus - Manages context menu state and items for the application.
 * 
 * Provides context menus for:
 * - File operations (open, hash, verify, copy path, etc.)
 * - Save operations (auto-save toggle, save, save as)
 */

import type { Accessor } from "solid-js";
import { createContextMenu, type ContextMenuItem } from "../components";
import type { DiscoveredFile } from "../types";
import type { BuildProjectOptions } from "./project/types";

// =============================================================================
// Types
// =============================================================================

export interface ContextMenusDeps {
  // File manager
  fileManager: {
    activeFile: Accessor<DiscoveredFile | null>;
    scanDir: Accessor<string | null>;
    selectedFiles: Accessor<Set<string>>;
    setActiveFile: (file: DiscoveredFile) => void;
    toggleFileSelection: (path: string) => void;
  };
  
  // Hash manager
  hashManager: {
    hashSingleFile: (file: DiscoveredFile) => Promise<void>;
    verifySegments: (file: DiscoveredFile) => Promise<void>;
  };
  
  // Project manager
  projectManager: {
    projectPath: Accessor<string | null>;
    autoSaveEnabled: Accessor<boolean>;
    setAutoSaveEnabled: (enabled: boolean) => void;
    saveProject: (options: BuildProjectOptions, customPath?: string) => Promise<{ success: boolean; error?: string }>;
    saveProjectAs: (options: BuildProjectOptions) => Promise<{ success: boolean; error?: string }>;
  };
  
  // Toast notifications
  toast: {
    success: (title: string, message?: string) => void;
    error: (title: string, message?: string) => void;
    info: (title: string, message?: string) => void;
  };
  
  // Build save options function
  buildSaveOptions: () => BuildProjectOptions | null;
}

export interface ContextMenusResult {
  // File context menu
  fileContextMenu: ReturnType<typeof createContextMenu>;
  getFileContextMenuItems: (file: Accessor<DiscoveredFile | null>) => ContextMenuItem[];
  
  // Save context menu
  saveContextMenu: ReturnType<typeof createContextMenu>;
  getSaveContextMenuItems: () => ContextMenuItem[];
}

// =============================================================================
// Hook
// =============================================================================

export function useContextMenus(deps: ContextMenusDeps): ContextMenusResult {
  const { fileManager, hashManager, projectManager, toast, buildSaveOptions } = deps;
  
  // ---------------------------------------------------------------------------
  // File Context Menu
  // ---------------------------------------------------------------------------
  const fileContextMenu = createContextMenu();
  
  const getFileContextMenuItems = (file: Accessor<DiscoveredFile | null>): ContextMenuItem[] => {
    if (!file) return [];
    const f = file();
    if (!f) return [];
    
    return [
      { id: "open", label: "Open", icon: "📂", onSelect: () => fileManager.setActiveFile(f) },
      { id: "sep1", label: "", separator: true },
      { id: "hash", label: "Compute Hash", icon: "🔐", shortcut: "cmd+h", onSelect: () => hashManager.hashSingleFile(f) },
      { id: "sep2", label: "", separator: true },
      { id: "select", label: fileManager.selectedFiles().has(f.path) ? "Deselect" : "Select", icon: fileManager.selectedFiles().has(f.path) ? "☐" : "☑", onSelect: () => fileManager.toggleFileSelection(f.path) },
      { id: "sep3", label: "", separator: true },
      { id: "copy-path", label: "Copy Path", icon: "📋", onSelect: () => {
        navigator.clipboard.writeText(f.path);
        toast.success("Path copied to clipboard");
      }},
      { id: "copy-name", label: "Copy Name", icon: "📋", onSelect: () => {
        const name = f.path.split("/").pop() || f.path;
        navigator.clipboard.writeText(name);
        toast.success("Name copied to clipboard");
      }},
    ];
  };

  // ---------------------------------------------------------------------------
  // Save Context Menu
  // ---------------------------------------------------------------------------
  const saveContextMenu = createContextMenu();
  
  const getSaveContextMenuItems = (): ContextMenuItem[] => {
    const hasEvidence = !!fileManager.scanDir();
    const hasExistingProject = !!projectManager.projectPath();
    
    return [
      { 
        id: "auto-save", 
        label: "Auto-save", 
        checked: projectManager.autoSaveEnabled(),
        onSelect: () => {
          projectManager.setAutoSaveEnabled(!projectManager.autoSaveEnabled());
          if (projectManager.autoSaveEnabled()) {
            toast.success("Auto-save enabled", "Project will be saved automatically");
          } else {
            toast.info("Auto-save disabled");
          }
        }
      },
      { id: "sep1", label: "", separator: true },
      { 
        id: "save", 
        label: "Save", 
        shortcut: "cmd+s",
        disabled: !hasEvidence || !hasExistingProject,
        onSelect: async () => {
          const options = buildSaveOptions();
          if (options && projectManager.projectPath()) {
            try {
              const result = await projectManager.saveProject(options, projectManager.projectPath()!);
              if (result.success) {
                toast.success("Saved", "Project updated");
              } else if (result.error && result.error !== "Save cancelled") {
                toast.error("Save Failed", result.error);
              }
            } catch (err) {
              toast.error("Save Failed", "Could not save project");
            }
          } else if (!hasExistingProject) {
            toast.info("No project", "Use 'Save As' to create a new project");
          }
        }
      },
      { 
        id: "save-as", 
        label: "Save As...", 
        shortcut: "cmd+shift+s",
        disabled: !hasEvidence,
        onSelect: async () => {
          const options = buildSaveOptions();
          if (options) {
            try {
              const result = await projectManager.saveProjectAs(options);
              if (result.success) {
                toast.success("Project Saved", "New project created");
              } else if (result.error && result.error !== "Save cancelled") {
                toast.error("Save Failed", result.error);
              }
            } catch (err) {
              toast.error("Save Failed", "Could not save project");
            }
          } else {
            toast.error("No Evidence", "Open an evidence directory first");
          }
        }
      },
    ];
  };

  return {
    fileContextMenu,
    getFileContextMenuItems,
    saveContextMenu,
    getSaveContextMenuItems,
  };
}
