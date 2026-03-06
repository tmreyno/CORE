// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Types for the Sidebar and its sub-components.
 */

import type { Accessor } from "solid-js";
import type { Theme, ResolvedTheme } from "../../../hooks/useTheme";
import type { ReportType } from "../../report/types";

export type LeftPanelTab = "dashboard" | "evidence" | "processed" | "casedocs" | "activity" | "bookmarks";
export type LeftPanelMode = "tabs" | "unified";

export interface SidebarProps {
  // Current tab
  activeTab: Accessor<LeftPanelTab>;
  onTabChange: (tab: LeftPanelTab) => void;

  // View mode (tabs vs unified collapsible)
  viewMode: Accessor<LeftPanelMode>;
  onViewModeChange: (mode: LeftPanelMode) => void;

  // State
  busy: Accessor<boolean>;
  hasEvidence: Accessor<boolean>;
  hasDiscoveredFiles: Accessor<boolean>;

  // Project info
  projectName?: Accessor<string | null>;
  projectPath?: Accessor<string | null>;
  hasProject?: Accessor<boolean>;

  // Bookmark count for badge
  bookmarkCount?: Accessor<number>;

  // Actions
  onExport: () => void;
  onReport: () => void;
  onSearch: () => void;
  onSettings: () => void;

  // Context menu actions
  onReportType?: (type: ReportType) => void;
  onExportSelected?: () => void;
  onClearBookmarks?: () => void;
  onExportBookmarks?: () => void;

  // Optional new actions
  onDeduplication?: () => void;
  onCommandPalette?: () => void;
  onHelp?: () => void;
  onEvidenceCollection?: () => void;
  onEvidenceCollectionList?: () => void;

  // Navigation context menu actions
  onScanEvidence?: () => void;
  onSelectAllEvidence?: () => void;
  onLoadAllInfo?: () => void;
  onRefreshProcessed?: () => void;
  onRefreshCaseDocs?: () => void;
  onToggleSidebar?: () => void;
  onToggleRightPanel?: () => void;
  onToggleQuickActions?: () => void;
  onOpenHelpTab?: () => void;
  onShowPerformance?: () => void;

  // Theme
  theme: Accessor<Theme>;
  resolvedTheme: Accessor<ResolvedTheme>;
  cycleTheme: () => void;
}
