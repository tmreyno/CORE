// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useAppActions - Provides action handlers and menu builders for the App.
 * 
 * Consolidates:
 * - Search handlers (handleSearch, handleSearchResultSelect)
 * - Context menu item builders (getFileContextMenuItems, getSaveContextMenuItems)
 */

import type { Accessor, Setter } from "solid-js";
import type { ContextMenuItem, SearchFilter, SearchResult, TabViewMode } from "../components";
import type { DiscoveredFile } from "../types";
import type { useFileManager, useHashManager, useProject, BuildProjectOptions } from "./index";
import { announce } from "../utils/accessibility";

// =============================================================================
// Types
// =============================================================================

export interface AppActionsDeps {
  fileManager: ReturnType<typeof useFileManager>;
  hashManager: ReturnType<typeof useHashManager>;
  projectManager: ReturnType<typeof useProject>;
  toast: {
    success: (title: string, message?: string) => void;
    error: (title: string, message?: string) => void;
    info: (title: string, message?: string) => void;
  };
  setShowReportWizard: Setter<boolean>;
  setCurrentViewMode: Setter<TabViewMode>;
  setLeftCollapsed: Setter<boolean>;
  setRightCollapsed: Setter<boolean>;
  setShowSettingsPanel: Setter<boolean>;
  setShowShortcutsModal: Setter<boolean>;
  setShowPerformancePanel: Setter<boolean>;
  setShowProjectWizard: Setter<boolean>;
  setShowSearchPanel: Setter<boolean>;
  buildSaveOptions: () => BuildProjectOptions | null;
}

// =============================================================================
// Search Handlers
// =============================================================================

export function createSearchHandlers(deps: Pick<AppActionsDeps, 'fileManager'>) {
  const { fileManager } = deps;
  
  /**
   * Search handler for SearchPanel - searches both file names and container contents.
   */
  const handleSearch = async (query: string, _filters: SearchFilter): Promise<SearchResult[]> => {
    const lowerQuery = query.toLowerCase();
    const results: SearchResult[] = [];
    const files = fileManager.discoveredFiles();
    
    // 1. Search through discovered files (container files themselves)
    for (const file of files) {
      const name = file.path.split("/").pop() || file.path;
      const matchesName = name.toLowerCase().includes(lowerQuery);
      const matchesPath = file.path.toLowerCase().includes(lowerQuery);
      
      if (matchesName || matchesPath) {
        results.push({
          id: file.path,
          path: file.path,
          name,
          size: file.size || 0,
          isDir: false,
          score: matchesName ? 100 : 50,
          matchType: matchesName ? "name" : "path",
        });
      }
    }
    
    // 2. Search INSIDE containers using backend (for queries >= 2 chars)
    if (query.length >= 2) {
      const { invoke } = await import("@tauri-apps/api/core");
      
      // Build list of containers to search
      const containers = files
        .filter(f => ["ad1", "zip", "7z", "rar", "tar", "tgz"].some(
          ext => f.container_type.toLowerCase().includes(ext)
        ))
        .map(f => [f.path, f.container_type.toLowerCase()] as [string, string]);
      
      if (containers.length > 0) {
        try {
          const containerResults = await invoke<Array<{
            containerPath: string;
            containerType: string;
            entryPath: string;
            name: string;
            isDir: boolean;
            size: number;
            score: number;
            matchType: string;
          }>>("search_all_containers", {
            containers,
            query,
            options: { maxResults: 200, includeDirs: false }
          });
          
          // Convert backend results to SearchResult format
          for (const r of containerResults) {
            results.push({
              id: `${r.containerPath}::${r.entryPath}`,
              path: r.entryPath,
              name: r.name,
              size: r.size,
              isDir: r.isDir,
              score: r.score,
              containerPath: r.containerPath,
              containerType: r.containerType,
              matchType: r.matchType,
            });
          }
        } catch (err) {
          console.error("Container search failed:", err);
        }
      }
    }
    
    // Sort by score (highest first) and limit results
    return results.sort((a, b) => b.score - a.score).slice(0, 300);
  };
  
  /**
   * Handle search result selection - navigates to file or container entry.
   */
  const handleSearchResultSelect = (result: SearchResult) => {
    if (result.containerPath) {
      // Result is inside a container - find container and select entry
      const containerFile = fileManager.discoveredFiles().find(f => f.path === result.containerPath);
      if (containerFile) {
        fileManager.setActiveFile(containerFile);
        announce(`Found ${result.name} in ${containerFile.path.split("/").pop()}`);
      }
    } else {
      // Result is a top-level file
      const file = fileManager.discoveredFiles().find(f => f.path === result.path);
      if (file) {
        fileManager.setActiveFile(file);
        announce(`Selected ${result.name}`);
      }
    }
  };
  
  return { handleSearch, handleSearchResultSelect };
}

// =============================================================================
// Context Menu Builders
// =============================================================================

export function createContextMenuBuilders(deps: Pick<AppActionsDeps, 'fileManager' | 'hashManager' | 'projectManager' | 'toast' | 'buildSaveOptions'>) {
  const { fileManager, hashManager, projectManager, toast, buildSaveOptions } = deps;
  
  /**
   * Get context menu items for a file.
   */
  const getFileContextMenuItems = (file: Accessor<DiscoveredFile | null>): ContextMenuItem[] => {
    const f = file();
    if (!f) return [];
    
    return [
      { id: "open", label: "Open", icon: "📂", onSelect: () => fileManager.setActiveFile(f) },
      { id: "sep1", label: "", separator: true },
      { id: "hash", label: "Compute Hash", icon: "🔐", shortcut: "cmd+h", onSelect: () => hashManager.hashSingleFile(f) },
      { id: "verify", label: "Verify Segments", icon: "✓", shortcut: "cmd+shift+h", onSelect: () => hashManager.verifySegments(f) },
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
  
  /**
   * Get context menu items for the save button.
   */
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
                toast.success("Project Saved", "Your project has been saved");
              } else if (result.error && result.error !== "Save cancelled") {
                toast.error("Save Failed", result.error);
              }
            } catch (err) {
              toast.error("Save Failed", "Could not save the project");
            }
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
                toast.success("Project Saved", "New project file created");
              } else if (result.error && result.error !== "Save cancelled") {
                toast.error("Save Failed", result.error);
              }
            } catch (err) {
              toast.error("Save Failed", "Could not save the project");
            }
          }
        }
      },
    ];
  };
  
  return { getFileContextMenuItems, getSaveContextMenuItems };
}
