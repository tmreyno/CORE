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
import { searchQuery, type TantivySearchOptions } from "../api/search";
import { announce } from "../utils/accessibility";
import { logger } from "../utils/logger";
import { getBasename } from "../utils/pathUtils";
const log = logger.scope("AppActions");

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

export function createSearchHandlers(deps: Pick<AppActionsDeps, 'fileManager' | 'projectManager'>) {
  const { fileManager, projectManager } = deps;
  
  /**
   * Search handler for SearchPanel — searches Tantivy index + FTS5 cross-entity.
   *
   * Tier 1: Tantivy full-text search (filenames, paths, content inside containers)
   * Tier 2: FTS5 cross-entity search (bookmarks, notes, activity log)
   * Falls back to in-memory filename search when no index is available.
   */
  const handleSearch = async (query: string, filters: SearchFilter): Promise<SearchResult[]> => {
    if (!query || query.length < 1) return [];

    const results: SearchResult[] = [];
    let tanvitySearched = false;

    // Tier 1: Tantivy index search
    if (query.length >= 2) {
      try {
        const opts: TantivySearchOptions = {
          query,
          limit: 200,
          searchContent: filters.searchContent ?? true,
          includeDirs: filters.includeDirs ?? false,
        };
        // Map filter file types to extensions
        if (filters.fileTypes && filters.fileTypes.length > 0) {
          opts.extensions = filters.fileTypes.map(t => t.replace(/^\./, ""));
        }
        if (filters.sizeRange?.min !== undefined) opts.minSize = filters.sizeRange.min;
        if (filters.sizeRange?.max !== undefined) opts.maxSize = filters.sizeRange.max;

        const searchResults = await searchQuery(opts);
        tanvitySearched = true;

        for (const hit of searchResults.hits) {
          results.push({
            id: hit.docId,
            path: hit.entryPath || hit.containerPath,
            name: hit.filename,
            matchContext: hit.snippet || undefined,
            size: hit.size,
            isDir: hit.isDir,
            score: Math.round(hit.score * 100),
            containerPath: hit.containerPath || undefined,
            containerType: hit.containerType || undefined,
            matchType: hit.contentMatch ? "content" : "name",
          });
        }
      } catch (err) {
        log.warn("Tantivy search failed (index may not be open), falling back:", err);
      }
    }

    // Fallback: in-memory filename search when Tantivy not available
    if (!tanvitySearched) {
      const lowerQuery = query.toLowerCase();
      const files = fileManager.discoveredFiles();
      for (const file of files) {
        const name = getBasename(file.path) || file.path;
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
    }

    // Tier 2: FTS5 cross-entity search (bookmarks, notes, activity log)
    if (query.length >= 2 && projectManager.project()) {
      try {
        const { invoke } = await import("@tauri-apps/api/core");
        await invoke("project_db_rebuild_fts").catch(() => {});
        const ftsResults = await invoke<Array<{
          source: string;
          id: string;
          snippet: string;
          rank: number;
        }>>("project_db_fts_search", { query, limit: 30 });

        for (const r of ftsResults) {
          const sourceLabel = r.source === "activity_log" ? "activity" : r.source;
          const cleanSnippet = r.snippet.replace(/<\/?mark>/g, "");
          results.push({
            id: `fts:${r.source}:${r.id}`,
            path: r.id,
            name: `[${sourceLabel}] ${cleanSnippet}`.slice(0, 120),
            matchContext: r.snippet,
            size: 0,
            isDir: false,
            score: Math.max(1, 80 + Math.round(r.rank * -10)),
            matchType: sourceLabel,
          });
        }
      } catch (err) {
        log.error("FTS cross-entity search failed:", err);
      }
    }
    
    // Sort by score (highest first) and limit results
    return results.sort((a, b) => b.score - a.score).slice(0, 300);
  };
  
  /**
   * Handle search result selection - navigates to file or container entry.
   */
  const handleSearchResultSelect = (result: SearchResult) => {
    // FTS cross-entity result — notify with match info
    if (result.id.startsWith("fts:")) {
      const source = result.matchType || "unknown";
      announce(`Found ${source} match: ${result.name}`);
      return;
    }

    if (result.containerPath) {
      // Result is inside a container - find container and select entry
      const containerFile = fileManager.discoveredFiles().find(f => f.path === result.containerPath);
      if (containerFile) {
        fileManager.setActiveFile(containerFile);
        announce(`Found ${result.name} in ${getBasename(containerFile.path)}`);
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
      { id: "sep2", label: "", separator: true },
      { id: "select", label: fileManager.selectedFiles().has(f.path) ? "Deselect" : "Select", icon: fileManager.selectedFiles().has(f.path) ? "☐" : "☑", onSelect: () => fileManager.toggleFileSelection(f.path) },
      { id: "sep3", label: "", separator: true },
      { id: "copy-path", label: "Copy Path", icon: "📋", onSelect: () => {
        navigator.clipboard.writeText(f.path);
        toast.success("Path copied to clipboard");
      }},
      { id: "copy-name", label: "Copy Name", icon: "📋", onSelect: () => {
        const name = getBasename(f.path) || f.path;
        navigator.clipboard.writeText(name);
        toast.success("Name copied to clipboard");
      }},
      { id: "sep4", label: "", separator: true },
      { id: "bookmark", label: "Bookmark", icon: "📑", onSelect: () => {
        const name = getBasename(f.path) || f.path;
        projectManager.addBookmark({
          target_type: "file",
          target_path: f.path,
          name,
        });
        toast.success("Bookmark added", name);
      }},
      { id: "add-note", label: "Add Note", icon: "📝", onSelect: () => {
        const name = getBasename(f.path) || f.path;
        projectManager.addNote({
          target_type: "file",
          target_path: f.path,
          title: `Note on ${name}`,
          content: "",
        });
        toast.success("Note created", `Note added for ${name}`);
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
