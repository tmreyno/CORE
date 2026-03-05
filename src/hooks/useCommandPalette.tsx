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
  HiOutlineClipboardDocumentList,
  HiOutlineInformationCircle,
  HiOutlineCodeBracket,
  HiOutlineDocument,
  HiOutlineDocumentCheck,
  HiOutlineChevronLeft,
  HiOutlineChevronRight,
  HiOutlineFingerPrint,
  HiOutlineCog6Tooth,
  HiOutlineCommandLine,
  HiOutlineMagnifyingGlass,
  HiOutlineArchiveBoxArrowDown,
  HiOutlineQuestionMarkCircle,
  HiOutlineArrowUpTray,
  HiOutlineBolt,
  HiOutlineDocumentDuplicate,
  HiOutlineRectangleGroup,
  HiOutlineArchiveBox,
  HiOutlineChartBar,
  HiOutlineClock,
  HiOutlineBookmark,
  HiOutlineCheckBadge,
  HiOutlineXMark,
} from "../components/icons";
import type { Accessor, Setter } from "solid-js";
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
  /** Whether a project is currently open — gates project-dependent actions */
  hasProject?: Accessor<boolean>;
  /** Open evidence collection as tab (preferred over modal setter) */
  onOpenEvidenceCollection?: () => void;
  /** Open evidence collection list as tab (preferred over modal setter) */
  onOpenEvidenceCollectionList?: () => void;
  /** Unified open directory handler (shows project wizard) */
  onOpenDirectory?: () => void;
  /** Unified open project handler (shows file picker for .cffx) */
  onOpenProject?: () => void;
  /** Open help / user guide tab */
  onOpenHelp?: () => void;
  /** Open export panel as tab */
  onOpenExport?: () => void;
  /** Toggle quick actions bar */
  onToggleQuickActions?: () => void;
  /** Cycle through themes */
  onCycleTheme?: () => void;
  /** Navigate to sidebar tabs */
  onShowDashboard?: () => void;
  onShowEvidence?: () => void;
  onShowProcessed?: () => void;
  onShowCaseDocs?: () => void;
  onShowActivity?: () => void;
  onShowBookmarks?: () => void;
  /** Close tabs */
  onCloseActiveTab?: () => void;
  onCloseAllTabs?: () => void;
  /** Deduplication */
  onDeduplication?: () => void;
  /** Performance panel */
  onShowPerformance?: () => void;
  /** Open the Merge Projects wizard */
  setShowMergeWizard?: Setter<boolean>;
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
    onOpenEvidenceCollection,
    onOpenEvidenceCollectionList,
    onOpenDirectory,
    onOpenProject,
    onOpenHelp,
    onOpenExport,
    onToggleQuickActions,
    onCycleTheme,
    onShowDashboard,
    onShowEvidence,
    onShowProcessed,
    onShowCaseDocs,
    onShowActivity,
    onShowBookmarks,
    onCloseActiveTab,
    onCloseAllTabs,
    onDeduplication,
    onShowPerformance,
    setShowMergeWizard,
  } = config;

  const projectOpen = () => config.hasProject?.() ?? false;

  return () => [
    // File operations
    {
      id: "browse",
      label: "Open Directory",
      icon: <HiOutlineFolderOpen class="w-4 h-4" />,
      category: "File",
      shortcut: "cmd+shift+o",
      onSelect: () => onOpenDirectory ? onOpenDirectory() : fileManager.browseScanDir(),
    },
    {
      id: "open-project",
      label: "Open Project",
      icon: <HiOutlineDocumentCheck class="w-4 h-4" />,
      category: "File",
      shortcut: "cmd+o",
      onSelect: () => onOpenProject ? onOpenProject() : {},
    },
    {
      id: "merge-projects",
      label: "Merge Projects",
      icon: <HiOutlineDocumentDuplicate class="w-4 h-4" />,
      category: "File",
      onSelect: () => setShowMergeWizard?.(true),
    },
    // Project-dependent actions — only shown when a project is open
    ...(projectOpen() ? [
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
      icon: <HiOutlineClipboardDocumentList class="w-4 h-4" />,
      category: "File",
      shortcut: "cmd+p",
      onSelect: () => setShowReportWizard(true),
    },
    {
      id: "evidence-collection",
      label: "New Evidence Collection",
      icon: <HiOutlineArchiveBoxArrowDown class="w-4 h-4" />,
      category: "File",
      onSelect: () => onOpenEvidenceCollection?.(),
    },
    {
      id: "evidence-collection-list",
      label: "Browse Evidence Collections",
      icon: <HiOutlineClipboardDocumentList class="w-4 h-4" />,
      category: "File",
      onSelect: () => onOpenEvidenceCollectionList?.(),
    },
    {
      id: "search",
      label: "Search Files",
      icon: <HiOutlineMagnifyingGlass class="w-4 h-4" />,
      category: "Search",
      shortcut: "cmd+f",
      onSelect: () => setShowSearchPanel(true),
    },
    {
      id: "hash-compute",
      label: "Compute Hash",
      icon: <HiOutlineFingerPrint class="w-4 h-4" />,
      category: "Hash",
      shortcut: "cmd+h",
      onSelect: () => {
        const active = fileManager.activeFile();
        if (active) hashManager.hashSingleFile(active);
      },
      disabled: !fileManager.activeFile(),
    },
    {
      id: "export",
      label: "Export Files",
      icon: <HiOutlineArrowUpTray class="w-4 h-4" />,
      category: "File",
      shortcut: "cmd+e",
      onSelect: () => onOpenExport?.(),
    },
    {
      id: "verify",
      label: "Verify All Hashes",
      icon: <HiOutlineCheckBadge class="w-4 h-4" />,
      category: "Hash",
      onSelect: () => hashManager.hashAllFiles(),
    },
    {
      id: "deduplication",
      label: "File Deduplication",
      icon: <HiOutlineDocumentDuplicate class="w-4 h-4" />,
      category: "Tools",
      onSelect: () => onDeduplication?.(),
      disabled: fileManager.discoveredFiles().length === 0,
    },
    ] : []),

    // Navigation commands — always available
    {
      id: "show-dashboard",
      label: "Show Dashboard",
      icon: <HiOutlineRectangleGroup class="w-4 h-4" />,
      category: "Navigate",
      onSelect: () => onShowDashboard?.(),
    },
    {
      id: "show-evidence",
      label: "Show Evidence",
      icon: <HiOutlineArchiveBox class="w-4 h-4" />,
      category: "Navigate",
      onSelect: () => onShowEvidence?.(),
    },
    {
      id: "show-processed",
      label: "Show Processed Databases",
      icon: <HiOutlineChartBar class="w-4 h-4" />,
      category: "Navigate",
      onSelect: () => onShowProcessed?.(),
    },
    {
      id: "show-casedocs",
      label: "Show Case Documents",
      icon: <HiOutlineClipboardDocumentList class="w-4 h-4" />,
      category: "Navigate",
      onSelect: () => onShowCaseDocs?.(),
    },
    {
      id: "show-activity",
      label: "Show Activity Timeline",
      icon: <HiOutlineClock class="w-4 h-4" />,
      category: "Navigate",
      onSelect: () => onShowActivity?.(),
    },
    {
      id: "show-bookmarks",
      label: "Show Bookmarks",
      icon: <HiOutlineBookmark class="w-4 h-4" />,
      category: "Navigate",
      onSelect: () => onShowBookmarks?.(),
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
    {
      id: "toggle-quick-actions",
      label: "Toggle Quick Actions Bar",
      icon: <HiOutlineBolt class="w-4 h-4" />,
      category: "View",
      onSelect: () => onToggleQuickActions?.(),
    },
    {
      id: "cycle-theme",
      label: "Cycle Theme",
      icon: <HiOutlineBolt class="w-4 h-4" />,
      category: "View",
      onSelect: () => onCycleTheme?.(),
    },

    // Tab management
    {
      id: "close-tab",
      label: "Close Active Tab",
      icon: <HiOutlineXMark class="w-4 h-4" />,
      category: "Tabs",
      shortcut: "cmd+w",
      onSelect: () => onCloseActiveTab?.(),
    },
    {
      id: "close-all-tabs",
      label: "Close All Tabs",
      icon: <HiOutlineXMark class="w-4 h-4" />,
      category: "Tabs",
      onSelect: () => onCloseAllTabs?.(),
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
      id: "performance",
      label: "Performance Monitor",
      icon: <HiOutlineBolt class="w-4 h-4" />,
      category: "Tools",
      onSelect: () => onShowPerformance?.(),
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
      id: "user-guide",
      label: "User Guide",
      icon: <HiOutlineQuestionMarkCircle class="w-4 h-4" />,
      category: "Help",
      onSelect: () => onOpenHelp?.(),
    },
    {
      id: "project-setup",
      label: "Project Setup",
      icon: <HiOutlineCog6Tooth class="w-4 h-4" />,
      category: "Project",
      onSelect: () => setShowProjectWizard(true),
    },
  ];
}
