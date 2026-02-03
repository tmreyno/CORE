// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

// Re-export container utility functions
export { getContainerIconColor, getContainerIconType } from "./ui/constants";

// Re-export components
export { Toolbar } from "./Toolbar";
export { StatusBar } from "./StatusBar";
export type { ProgressItem, QuickAction, AutoSaveStatus } from "./StatusBar";
export { FilePanel } from "./FilePanel";
export { FileRow } from "./FileRow";
export { DetailPanel } from "./DetailPanel";
export { DetailPanelContent } from "./DetailPanelContent";
export { TabBar } from "./TabBar";
export type { OpenTab, TabViewMode } from "./TabBar";
export { TreePanel } from "./TreePanel";

// === Tree Components ===
// Tree primitives and shared components
export { 
  TreeRow,
  TreeIcon,
  ExpandIcon,
  TreeEmptyState,
  TreeLoadingState,
  TreeErrorState,
  ContainerHeader,
  type TreeRowProps,
  type TreeIconProps,
  type TreeEmptyStateProps,
  type TreeErrorStateProps,
  type ContainerHeaderProps,
  type TreeNodeState,
  type TreeItemData,
} from "./tree";

// Main tree components
export { EvidenceTree, type SelectedEntry } from "./EvidenceTree";
export type { TreeExpansionState } from "./EvidenceTree/types";

// Type filter component (shared by EvidenceTree and FilePanel)
export { TypeFilterBar, type TypeFilterBarProps } from "./TypeFilterBar";

export { ContainerEntryViewer, type EntryViewMode } from "./ContainerEntryViewer";
export { ProgressModal } from "./ProgressModal";
export { HexViewer } from "./HexViewer";
// Viewer types are now in types.ts but re-exported from HexViewer for backward compatibility
export type { ParsedMetadata, FileTypeInfo, HeaderRegion, MetadataField } from "../types";
export { MetadataPanel } from "./MetadataPanel";
export { TextViewer } from "./TextViewer";
export { PdfViewer } from "./PdfViewer";
export { ImageViewer } from "./ImageViewer";

// Center Pane Tabs (for switching between evidence and case documents)
// Note: Types CenterPaneTab and OpenDocumentTab are exported from hooks/useAppState
export { CenterPaneTabs } from "./CenterPaneTabs";

// Unified Center Pane (new unified tab interface)
export { CenterPane, type CenterPaneProps, type CenterTab, type CenterTabType, type CenterPaneViewMode } from "./layout";

// Case Documents Panel (COC forms, intake forms, etc.)
export { CaseDocumentsPanel, type CaseDocumentsPanelProps } from "./CaseDocumentsPanel";

// Hash display components
export { 
  HashBadge, 
  HashVerificationIndicator,
  getHashState,
  hasVerifiedMatch,
  getStoredHashCount,
  getTotalHashCount,
  isHashing,
  isCompleting,
  formatChunks,
  type HashState,
  type HashBadgeProps,
} from "./HashBadge";

// Report components (ReportWizard is lazy-loaded in App.tsx)
export type { ForensicReport as ReportData, OutputFormat as ReportFormat } from "./report";

// Project Setup
export { ProjectSetupWizard } from "./ProjectSetupWizard";
export type { ProjectLocations } from "./ProjectSetupWizard";

// UI Enhancement Components
export { 
  Skeleton, 
  SkeletonText, 
  SkeletonFileRow, 
  SkeletonTreeNode, 
  SkeletonHexView, 
  SkeletonMetadata,
  SkeletonPanel,
  SkeletonLoader 
} from "./Skeleton";

export { ToastProvider, useToast } from "./Toast";
export type { Toast, ToastType } from "./Toast";

export { ErrorBoundary, CompactErrorBoundary } from "./ErrorBoundary";

export { Tooltip, TooltipText } from "./Tooltip";
export type { TooltipPosition } from "./Tooltip";

export { Transition, Fade, SlideUp, ScaleFade, Collapse } from "./Transition";
export type { TransitionType } from "./Transition";

export { ThemeSwitcher, ThemeSelector } from "./ThemeSwitcher";

// New UI/UX Components
export { CommandPalette, createCommandPalette } from "./CommandPalette";
export type { CommandAction } from "./CommandPalette";

export { ContextMenu, createContextMenu } from "./ContextMenu";
export type { ContextMenuItem } from "./ContextMenu";

export { Breadcrumb, pathToBreadcrumbs } from "./Breadcrumb";
export type { BreadcrumbItem } from "./Breadcrumb";

export { KeyboardShortcutsModal, DEFAULT_SHORTCUT_GROUPS } from "./KeyboardShortcutsModal";
export type { ShortcutGroup } from "./KeyboardShortcutsModal";

export { 
  EmptyState, 
  NoFilesEmptyState, 
  NoSearchResultsEmptyState, 
  ErrorEmptyState,
  NoDatabasesEmptyState,
  DropZoneEmptyState 
} from "./EmptyState";
export type { EmptyStateVariant } from "./EmptyState";

// Recent Projects
export { RecentProjectsList } from "./RecentProjectsList";
export type { RecentProjectsListProps } from "./RecentProjectsList";

// Project Templates
export { ProjectTemplateSelector } from "./ProjectTemplateSelector";
export type { ProjectTemplateSelectorProps, TemplateSummary, TemplateCategory } from "./ProjectTemplateSelector";

// Drag & Drop
export { DropZone, useDragDrop } from "./DragDrop";
export type { DropZoneProps, DragDropOptions, DragDropState } from "./DragDrop";

// Settings/Preferences (SettingsPanel component is lazy-loaded in App.tsx)
export { createPreferences, DEFAULT_PREFERENCES, getPreference, getLastPath, setLastPath, getRecentProjects, addRecentProject, removeRecentProject, clearRecentProjects } from "./preferences";
export type { AppPreferences, Theme, TreeDensity, HashAlgorithm, RecentProject } from "./preferences";
export type { SettingsPanelProps } from "./SettingsPanel";

// Search/Filter Panel
export { SearchPanel, useSearch } from "./SearchPanel";
export type { SearchFilter, SearchResult, SavedSearch, SearchPanelProps } from "./SearchPanel";

// Bookmarks Panel
export { BookmarksPanel } from "./BookmarksPanel";
export type { BookmarksPanelProps } from "./BookmarksPanel";

// Filter Presets Dropdown
export { FilterPresetsDropdown } from "./FilterPresetsDropdown";
export type { FilterPresetsDropdownProps, QuickFilter } from "./FilterPresetsDropdown";

// Quick Actions Bar
export { QuickActionsBar } from "./QuickActionsBar";
export type { QuickActionsBarProps } from "./QuickActionsBar";

// Onboarding/Help System
export { 
  useTour,
  TourOverlay,
  Tooltip as OnboardingTooltip,
  HelpButton,
  WelcomeModal,
  DEFAULT_TOUR_STEPS
} from "./Onboarding";
export type { TourStep, TooltipConfig, TourOverlayProps, RecentProjectInfo, WelcomeModalProps } from "./Onboarding";

// Enhanced Skeleton Variants
export { SkeletonTree, SkeletonTable, SkeletonCard, SkeletonProgress } from "./Skeleton";

// Virtualization Components
export { VirtualList, VirtualTree, flattenTree, useVirtualList } from "./VirtualList";
export type { VirtualListProps, VirtualTreeProps, UseVirtualListOptions } from "./VirtualList";

// Shared UI Components
export {
  Input,
  Textarea,
  Select,
  FormField,
  Card,
  Badge,
  SectionHeader,
  EmptyStateSimple,
  Checkbox,
  Divider,
  inputStyles,
  inputClass,
  inputClassSm,
  cardStyles,
  textStyles,
  badgeStyles,
  buttonStyles,
} from "./ui";

// Layout Components
export { 
  Sidebar, 
  AppModals,
  RightPanel,
  LeftPanelContent,
  CollapsiblePanelContent,
  type SidebarProps, 
  type LeftPanelTab,
  type LeftPanelMode,
  type AppModalsProps,
  type TourState,
  type FileContextMenuState,
  type RightPanelProps,
  type LeftPanelContentProps,
  type CollapsiblePanelContentProps,
} from "./layout";

// Project Management Components
export { ProjectToolbar } from "./project/ProjectToolbar";
export { RecoveryModal } from "./project/RecoveryModal";
export { ProfileSelector } from "./project/ProfileSelector";
export { TemplateGallery } from "./project/TemplateGallery";
export { ActivityHeatmap } from "./project/ActivityHeatmap";
export { ComparisonView } from "./project/ComparisonView";

// Export & Archive Panel
export { ExportPanel } from "./ExportPanel";
export type { ExportPanelProps, ExportMode } from "./ExportPanel";
export { SimpleActivityPanel } from "./SimpleActivityPanel";

// Icons - Heroicons Outline (re-exports for convenience)
export * from "./icons";
