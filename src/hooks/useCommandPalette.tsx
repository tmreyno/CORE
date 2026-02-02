// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================
// Command palette actions factory
// =============================================================================

import {
  HiOutlineFolderOpen,
  HiOutlineArrowPath,
  HiOutlineDocumentText,
  HiOutlineInformationCircle,
  HiOutlineCodeBracket,
  HiOutlineDocument,
  HiOutlineChevronLeft,
  HiOutlineChevronRight,
  HiOutlineLockClosed,
  HiOutlineCog6Tooth,
  HiOutlineCommandLine,
  HiOutlineMagnifyingGlass,
  HiOutlineChartBar,
} from "solid-icons/hi";
import type { Setter } from "solid-js";
import type { CommandAction } from "../components";
import type { useFileManager } from "./useFileManager";
import type { useHashManager } from "./useHashManager";

/** View modes supported by command palette - subset of TabViewMode */
export type CommandPaletteViewMode = "info" | "text" | "hex";

export interface CommandPaletteConfig {
  fileManager: ReturnType<typeof useFileManager>;
  hashManager: ReturnType<typeof useHashManager>;
  setCurrentViewMode: (mode: CommandPaletteViewMode) => void;
  setLeftCollapsed: Setter<boolean>;
  setRightCollapsed: Setter<boolean>;
  setShowReportWizard: Setter<boolean>;
  setShowSettingsPanel: Setter<boolean>;
  setShowShortcutsModal: Setter<boolean>;
  setShowProjectWizard: Setter<boolean>;
  setShowSearchPanel: Setter<boolean>;
  setShowPerformancePanel: Setter<boolean>;
}

/**
 * Creates the command palette actions array.
 * Returns a function that generates fresh actions each time (for reactivity).
 */
export function createCommandPaletteActions(config: CommandPaletteConfig): () => CommandAction[] {
  const {
    fileManager,
    hashManager,
    setCurrentViewMode,
    setLeftCollapsed,
    setRightCollapsed,
    setShowReportWizard,
    setShowSettingsPanel,
    setShowShortcutsModal,
    setShowProjectWizard,
    setShowSearchPanel,
    setShowPerformancePanel,
  } = config;

  return () => [
    // File operations
    {
      id: "browse",
      label: "Browse Folder",
      icon: <HiOutlineFolderOpen class="w-4 h-4" />,
      category: "File",
      shortcut: "cmd+o",
      onSelect: () => fileManager.browseScanDir(),
    },
    {
      id: "scan",
      label: "Scan for Files",
      icon: <HiOutlineArrowPath class="w-4 h-4" />,
      category: "File",
      shortcut: "cmd+r",
      onSelect: () => fileManager.scanForFiles(),
    },
    {
      id: "report",
      label: "Generate Report",
      icon: <HiOutlineDocumentText class="w-4 h-4" />,
      category: "File",
      shortcut: "cmd+p",
      onSelect: () => setShowReportWizard(true),
    },

    // View operations
    {
      id: "view-info",
      label: "Show Info View",
      icon: <HiOutlineInformationCircle class="w-4 h-4" />,
      category: "View",
      shortcut: "cmd+1",
      onSelect: () => setCurrentViewMode("info"),
    },
    {
      id: "view-hex",
      label: "Show Hex View",
      icon: <HiOutlineCodeBracket class="w-4 h-4" />,
      category: "View",
      shortcut: "cmd+2",
      onSelect: () => setCurrentViewMode("hex"),
    },
    {
      id: "view-text",
      label: "Show Text View",
      icon: <HiOutlineDocument class="w-4 h-4" />,
      category: "View",
      shortcut: "cmd+3",
      onSelect: () => setCurrentViewMode("text"),
    },
    {
      id: "toggle-left",
      label: "Toggle Left Panel",
      icon: <HiOutlineChevronLeft class="w-4 h-4" />,
      category: "View",
      shortcut: "cmd+b",
      onSelect: () => setLeftCollapsed((v) => !v),
    },
    {
      id: "toggle-right",
      label: "Toggle Right Panel",
      icon: <HiOutlineChevronRight class="w-4 h-4" />,
      category: "View",
      shortcut: "cmd+shift+b",
      onSelect: () => setRightCollapsed((v) => !v),
    },

    // Hash operations
    {
      id: "hash-compute",
      label: "Compute Hash",
      icon: <HiOutlineLockClosed class="w-4 h-4" />,
      category: "Hash",
      shortcut: "cmd+h",
      onSelect: () => {
        const active = fileManager.activeFile();
        if (active) hashManager.hashSingleFile(active);
      },
      disabled: !fileManager.activeFile(),
    },

    // Settings & Help
    {
      id: "settings",
      label: "Settings",
      icon: <HiOutlineCog6Tooth class="w-4 h-4" />,
      category: "Settings",
      shortcut: "cmd+,",
      onSelect: () => setShowSettingsPanel(true),
    },
    {
      id: "shortcuts",
      label: "Keyboard Shortcuts",
      icon: <HiOutlineCommandLine class="w-4 h-4" />,
      category: "Help",
      shortcut: "?",
      onSelect: () => setShowShortcutsModal(true),
    },
    {
      id: "project-setup",
      label: "Project Setup",
      icon: <HiOutlineCog6Tooth class="w-4 h-4" />,
      category: "Project",
      onSelect: () => setShowProjectWizard(true),
    },
    {
      id: "search",
      label: "Search Files",
      icon: <HiOutlineMagnifyingGlass class="w-4 h-4" />,
      category: "Search",
      shortcut: "cmd+f",
      onSelect: () => setShowSearchPanel(true),
    },

    // Developer tools
    {
      id: "dev-performance",
      label: "Performance Monitor",
      icon: <HiOutlineChartBar class="w-4 h-4" />,
      category: "Developer",
      shortcut: "ctrl+shift+p",
      onSelect: () => setShowPerformancePanel((v) => !v),
    },
  ];
}
