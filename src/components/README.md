# CORE-FFX Components

SolidJS UI components for CORE-FFX.

## Directory Structure

```
components/
|-- ActivityPanel.tsx          # Activity timeline panel
|-- acquire/
|   |-- AcquireLayout.tsx       # Acquire edition view router (dashboard/identify/export/browse/verify/collection/triage)
|   |-- AcquireDashboard.tsx    # Acquire dashboard with 4-phase workflow
|   |-- AcquireIdentifyView.tsx # Dedicated identify-system process view for host/drive survey
|   |-- AcquireCollectionView.tsx # Dedicated Acquire collection wrapper around the evidence form
|   |-- AcquireProcessShell.tsx # Shared header/shell wrapper for Acquire process views
|   |-- AcquireExportView.tsx   # Acquire export shell with flat workflow chrome
|   |-- AcquireVerifyView.tsx   # Acquire verification workflow shell
|   |-- AcquireTriageView.tsx   # Standalone quick triage workflow
|   |-- SystemInfoPanel.tsx     # Right-panel system info display (identity, CPU, memory, drives, network)
|   |-- acquire.css             # Acquire-only shell/layout styles (flat process sections, workflow headers)
|-- activity-panel/
|   |-- ActivityCard.tsx        # Individual activity card with EMA-smoothed progress tracker
|   |-- types.ts                # ActivityCardProps and related types
|-- bookmarks/
|   |-- BookmarksPanel.tsx      # Bookmark list with search, filter by type/color, sort
|-- casedocs/
|   |-- DocumentItem.tsx        # Compact single-line case document row
|-- drives/
|   |-- DriveSourcePanel.tsx    # Left sidebar drive/volume browser with bidirectional export sync
|-- export/
|   |-- PhysicalImageMode.tsx   # E01/Raw physical image creation UI (format toggle, compression, hash)
|   |-- LogicalImageMode.tsx    # L01 logical evidence creation UI
|   |-- Aff4ImageMode.tsx       # AFF4 forensic container creation UI
|   |-- NativeExportMode.tsx    # 7z archive / file copy export with forensic presets
|   |-- ToolsMode.tsx           # Archive test/repair/validate UI
|   |-- MemoryMode.tsx          # Live RAM capture UI (info, options, progress, results)
|   |-- TriageMode.tsx          # Triage mode UI (profile selection, category toggles, secret scan)
|   |-- DriveSelector.tsx       # Modal picker for system drives with read-only mount toggle
|   |-- SplitSizeSelector.tsx   # Shared split/segment size dropdown (9 presets + Custom)
|   |-- CaseMetadataSection.tsx # Collapsible case metadata inputs (case#, evidence#, examiner)
|   |-- CreateMode.tsx          # L01 section in export (image name, compression, hash, case metadata)
|-- export-panel/
|   |-- ExportPanelComponent.tsx # Unified acquire & export panel (8 modes: physical/logical/aff4/native/tools/memory/triage)
|   |-- ExportHeader.tsx        # Mode tab selector ("Acquire & Export" header)
|   |-- ExportSourceSection.tsx # Source file/folder picker + destination + inline drive tree
|   |-- ExportFooter.tsx        # Start/cancel buttons with mode-aware labels
|   |-- DriveTreeBrowser.tsx    # Inline drive/volume browser with lazy dir trees, 15s auto-refresh, context menu
|   |-- types.ts                # ExportPanelProps interface
|   |-- index.ts                # Barrel exports
|-- help/
|   |-- HelpPanel.tsx           # Comprehensive in-app documentation (15 searchable sections)
|-- icons/
|   |-- index.tsx               # Centralized icon exports (HiOutline* only)
|-- import/
|   |-- ImportAcquisitionsWizard.tsx # 3-step wizard for scanning + importing companion files
|-- merge/
|   |-- MergeProjectsWizard.tsx     # Multi-step merge wizard (standard + merge-into-open)
|   |-- DataCategorySelector.tsx    # 2-column checkbox grid for 12 merge data categories
|   |-- ProjectSummaryCard.tsx      # Per-project detail sections with per-item checkboxes
|   |-- SelectStep.tsx              # Step 1: file picker with pinned current project
|   |-- CollectionReconciliation.tsx # Conflict detection and reconciliation UI
|   |-- types.ts                    # MergeProjectsWizardProps, CollectionConflict, etc.
|-- notes/
|   |-- NotesPanel.tsx          # Notes list with search, filter, CRUD
|   |-- NoteItem.tsx            # Individual note display row
|   |-- NoteEditDialog.tsx      # Create/edit modal (title, content, priority, tags)
|   |-- helpers.ts              # Icon/label/formatting utilities
|   |-- types.ts                # Type definitions + NOTE_PRIORITIES
|-- project/
|   |-- ProjectCloseModal.tsx  # Close-project status modal (save, drain, checkpoint, close)
|   |-- UserConfirmModal.tsx    # Profile confirmation on project open/create
|-- report/
|   |-- ReportWizard.tsx        # Report wizard UI
|   |-- index.ts                # Barrel exports
|-- search/
|   |-- SearchFilters.tsx       # Search filters with "Search contents" toggle
|   |-- SearchResultItem.tsx    # Result item with content snippets and <mark> highlighting
|-- settings/
|   |-- AppearanceTab.tsx       # Theme, accent color, layout, font size slider
|   |-- BehaviorTab.tsx         # Confirmations, auto-save, auto-hash, logging
|   |-- DefaultsTab.tsx         # Hash algorithm, acquisition format, export, view, sort defaults
|   |-- PerformanceTab.tsx      # Loading, memory, caching, worker thread settings
|   |-- WorkspaceModeTab.tsx    # Workspace preset grid + per-module toggle list
|   |-- UserProfilesTab.tsx     # Full CRUD for examiner/user profiles
|-- toolbar/
|   |-- WorkspaceModeSelector.tsx  # Toolbar dropdown for quick workspace mode switching
|   |-- toolbarHelpers.ts          # buildProjectLocations, toolbar utility functions
|-- tree/
|   |-- ExpandIcon.tsx          # Expand/collapse chevron for tree nodes
|   |-- TreeIcon.tsx            # File/folder icon resolver for tree items
|   |-- constants.ts            # TREE_ROW_* CSS class constants, getTreeIndent()
|-- ui/
|   |-- Toggle.tsx              # Toggle switch component
|   |-- Slider.tsx              # Range slider component
|   |-- LoadingOverlay.tsx      # Small toast-style loading indicator (bottom-right)
|-- viewerMetadata/
|   |-- shared.tsx              # Shared right-panel primitives (CollapsibleGroup, MetadataRow, etc.)
|   |-- index.tsx               # Barrel re-export
|   |-- *Section.tsx            # 10 viewer metadata section files
|   |-- FileInfoTab.tsx         # File info tab with case metadata
|-- BookmarksPanel.tsx         # Evidence bookmarks panel
|-- Breadcrumb.tsx             # Navigation breadcrumb trail
|-- CaseDocumentsPanel.tsx     # Case documents panel
|-- CenterPaneTabs.tsx         # Center pane tab management
|-- CommandPalette.tsx         # Global command palette (Cmd+K)
|-- ContainerEntryViewer.tsx   # Read internal container entries (AD1/E01/ZIP/etc.)
|-- ContextMenu.tsx            # Right-click context menus
|-- DetailPanel.tsx            # Tabbed detail panel for active files
|-- DetailPanelContent.tsx     # Container info rendering
|-- DocumentViewer.tsx         # Universal document viewer
|-- DragDrop.tsx               # Drag and drop file upload zone
|-- EmptyState.tsx             # Empty state placeholders
|-- ErrorBoundary.tsx          # Error boundary with fallback UI
|-- EvidenceCollectionPanel.tsx # Tab-based on-site evidence collection form
|-- EvidenceCollectionListPanel.tsx # Browse/list all evidence collections
|-- EvidenceTree.tsx           # Unified evidence tree (AD1, E01, Archives, UFED)
|-- ExportPanel.tsx            # Export panel orchestrator (state, conversion, IPC)
|-- FilePanel.tsx              # Evidence file list
|-- FileRow.tsx                # File list row
|-- FilterPresetsDropdown.tsx  # Filter preset selector
|-- HashBadge.tsx              # Hash verification badge
|-- HelpPanel.tsx              # (moved to help/)
|-- HexViewer.tsx              # Hex dump viewer
|-- ImageViewer.tsx            # Image viewer with EXIF support
|-- KeyboardShortcutsModal.tsx # Keyboard shortcuts reference modal
|-- LinkedDataTree.tsx         # Reusable linked data tree (collections↔COC↔evidence)
|-- LinkedDataPanel.tsx        # Right-panel wrapper for linked data tree
|-- MetadataPanel.tsx          # Parsed metadata / hex navigation
|-- Onboarding.tsx             # First-run onboarding wizard
|-- PdfViewer.tsx              # PDF viewer
|-- PerformancePanel.tsx       # Performance monitoring panel
|-- ProcessedDatabasePanel.tsx # Processed DB list
|-- ProcessedDetailPanel.tsx   # Processed DB details
|-- ProgressModal.tsx          # Global progress modal
|-- ProjectSetupWizard.tsx     # Project setup wizard
|-- ProjectTemplateSelector.tsx # Project template selector
|-- QuickActionsBar.tsx        # Quick actions toolbar
|-- RecentProjectsList.tsx     # Recent projects list
|-- SearchPanel.tsx            # Global search panel
|-- SettingsPanel.tsx          # Application settings panel (10 tabs)
|-- SimpleActivityPanel.tsx    # Compact activity panel
|-- Skeleton.tsx               # Loading skeleton components
|-- SpreadsheetViewer.tsx      # Spreadsheet viewer
|-- StatusBar.tsx              # Status bar with system stats + project stats
|-- TabBar.tsx                 # Tab and view mode controls
|-- TextViewer.tsx             # Text viewer
|-- ThemeSwitcher.tsx          # Light/dark theme toggle
|-- Toast.tsx                  # Toast notification system
|-- Toolbar.tsx                # Primary toolbar
|-- Tooltip.tsx                # Custom styled tooltips
|-- Transition.tsx             # Animation transitions
|-- TreePanel.tsx              # UFED associated file tree
|-- TypeFilterBar.tsx          # Evidence type filter bar
|-- UpdateModal.tsx            # CORE-FFX updater wrapper with bundled in-app release notes
|-- VirtualList.tsx            # Virtualized list for large datasets
|-- index.ts                    # Barrel exports
```

## Component Categories

### Navigation

- `TabBar` - Tab and view mode controls
- `TreePanel` - UFED associated file tree
- `EvidenceTree` - Unified evidence tree (AD1, E01, Archives, UFED)
- `Breadcrumb` - Navigation breadcrumb trail

### Evidence List

- \`FilePanel\` - Evidence file list panel
- \`FileRow\` - Individual file row component

### Viewers

- \`DetailPanel\` - Tabbed detail panel for active files
- \`HexViewer\` - Hex dump viewer with highlighting
- \`TextViewer\` - Plain text viewer
- \`DocumentViewer\` - Universal document viewer
- \`ImageViewer\` - Image viewer with EXIF support
- \`PdfViewer\` - PDF viewer
- \`SpreadsheetViewer\` - Spreadsheet viewer
- \`ContainerEntryViewer\` - Internal container entries (AD1)

### Metadata

- \`MetadataPanel\` - Parsed metadata / hex navigation
- \`DetailPanelContent\` - Container info rendering
- \`HashBadge\` - Hash verification badge

### Project & Reports

- \`ProjectSetupWizard\` - Project setup wizard
- \`ConfigureLocationsStep\` - Wizard step: project locations + Case Identification (case number, case name)
- \`ProjectTemplateSelector\` - Project template selector
- \`RecentProjectsList\` - Recent projects list
- \`ReportWizard\` - Report generation wizard
- `EvidenceCollectionPanel` - Tab-based on-site evidence collection form (schema-driven, independent of Report Wizard)
- `EvidenceCollectionListPanel` - Browse/list all evidence collections
- `LinkedDataTree` - Reusable tree showing collection↔COC↔evidence file relationships
- `LinkedDataPanel` - Right-panel wrapper with Linked Data & Summary tabs

### Acquire Workflows

- `AcquireLayout` - Acquire edition view router for dashboard, identify, export, browse, verify, collection, and triage
- `AcquireDashboard` - Top-level 4-phase workflow dashboard for field use, now acting as the launcher rather than embedding the full Identify workflow
- `AcquireIdentifyView` - Dedicated identify-system workflow that captures system stats, drive metadata, and the evidence item folder, then exposes a clear continue-to-collection handoff inside the same screen
- `AcquireCollectionView` - Dedicated Acquire collection workflow wrapper with shared shell chrome and optional system-info side panel; can open a specific saved collection in review or edit mode
- `AcquireProcessShell` - Shared process-view wrapper for consistent back navigation, title chrome, and header actions
- `AcquireExportView` - Acquire/export shell that wraps `ExportPanelComponent` with lightweight header chrome
- `AcquireVerifyView` - Verification flow with minimal header chrome and flat process body
- `AcquireTriageView` - Standalone quick triage flow with profile selection, live progress, and result summary
- `MemoryMode` - Live RAM capture mode rendered inside the Acquire/export process shell
- `TriageMode` - Embedded triage mode rendered inside the Acquire/export process shell
- `PhysicalImageMode` - Physical imaging options with lightweight forensic callouts
- `LogicalImageMode` - L01 logical imaging options with lightweight forensic callouts

### Merge

- `MergeProjectsWizard` - Multi-step merge wizard (dual-mode: standard merge + merge-into-open)
- `DataCategorySelector` - 2-column checkbox grid for toggling 12 merge data categories
- `ProjectSummaryCard` - Expandable per-project detail sections with per-item inclusion checkboxes
- `SelectStep` - Step 1: file picker with pinned current project support
- `CollectionReconciliation` - Conflict detection and side-by-side reconciliation UI

### Import

- `ImportAcquisitionsWizard` - 3-step modal wizard for scanning and importing acquisitions from companion files

### Export & Acquisition

- `ExportPanelComponent` - Unified acquire & export panel shared by both editions (8 modes: physical/logical/aff4/native/tools/memory/triage)
- `ExportHeader` - Mode tab selector ("Acquire & Export" header with mode buttons)
- `ExportSourceSection` - Source file/folder picker with destination selector and inline drive tree
- `ExportFooter` - Start/cancel buttons with mode-aware labels and validation
- `DriveTreeBrowser` - Inline drive/volume browser with lazy-loaded directory trees, 15-second auto-refresh for hot-plug detection, and right-click context menu for acquisition mode selection
- `ExportPanel` - Orchestrator that manages export state, format conversion, and IPC
- `DriveSelector` - Modal picker for system drives with read-only mount toggle and system disk warnings
- `SplitSizeSelector` - Shared split/segment size dropdown (9 presets from 100 MB to 25 GB + Custom)
- `CaseMetadataSection` - Collapsible case info inputs (case number, evidence number, examiner, description, notes)
- `PhysicalImageMode` - E01/Raw physical image creation with format toggle, compression, and hash selection
- `LogicalImageMode` - L01 logical evidence creation UI
- `Aff4ImageMode` - AFF4 forensic container creation (compression + multi-select hash + case metadata)
- `NativeExportMode` - 7z archive / file copy export with forensic presets (Standard, Court, Transfer, Long-term)
- `ToolsMode` - Archive test/repair/validate utility UI
- `MemoryMode` - Live RAM capture UI (system info, options, progress, results)
- `TriageMode` - Triage mode (profile selection, category toggles, secrets scan, results)

### Drive Source Panel

- `DriveSourcePanel` - Left sidebar drive/volume browser with live bidirectional sync to the Export Panel. Checking/unchecking items instantly adds/removes them from the export source list.

### Settings Tabs

- `SettingsPanel` - Application settings panel with 10 tabs
- `AppearanceTab` - Theme, accent color, layout, sidebar position, font size slider
- `BehaviorTab` - Confirmations, auto-save, auto-hash, logging level
- `DefaultsTab` - Hash algorithm, acquisition format/compression/segment, export format, view mode, sort order, date format
- `PerformanceTab` - Loading thresholds, concurrent operations, worker threads, memory/cache, index content toggle
- `WorkspaceModeTab` - Workspace preset grid (7 presets) + per-module toggle list (6 modules)
- `UserProfilesTab` - Full CRUD for examiner/user profiles (name, title, organization, certifications, logo)

### Notes

- `NotesPanel` - Notes list with search, filter by target type, create/edit/remove
- `NoteItem` - Individual note display row with priority colors and tags
- `NoteEditDialog` - Create/edit modal with title, content textarea, priority selector, comma-separated tags

### Search

- `SearchPanel` - Global search panel with Tantivy full-text search + in-memory fallback
- `SearchFilters` - Search filters with "Search contents" toggle for content vs. filename search
- `SearchResultItem` - Result item with content snippets and `<mark>` highlighting

### Viewer Metadata

- `shared.tsx` - Shared right-panel primitives: `CollapsibleGroup`, `MetadataRow`, `OptionalMetadataRow`, `SectionHeader`, `SummaryRow`, `StatusBadge`
- `FileInfoTab` - File info tab with case metadata section
- `*Section.tsx` - 10 viewer metadata section files for different content types
- `SystemInfoPanel` - Right-panel system info display (system identity, CPU, memory, drives, network interfaces)

### Tree Primitives

- `ExpandIcon` - Expand/collapse chevron for tree nodes
- `TreeIcon` - File/folder icon resolver for tree items
- `constants.ts` - `TREE_ROW_BASE_CLASSES`, `TREE_ROW_SELECTED_CLASSES`, `getTreeIndent()` shared constants

### UI Primitives

- `Toggle` - Toggle switch component
- `Slider` - Range slider component
- `LoadingOverlay` - Small toast-style loading indicator (bottom-right, auto-dismiss)

### Updates

- `UpdateModal` - CORE-FFX config wrapper over the shared updater modal and hook, with bundled release notes from `docs/releases`

### Help

- `HelpPanel` - Comprehensive in-app documentation system (15 searchable sections, opens as center-pane tab)

### Acquire Layout Rule

Acquire process views already sit inside the Acquire layout shell. `AcquireExportView`, `AcquireVerifyView`, `AcquireTriageView`, and the export modes rendered inside them should use Tailwind utility classes and standard `index.css` component classes (e.g., `.callout`, `.card`, `.btn`) rather than reintroducing nested desktop-style `card`, `info-card`, or toolbar framing around each step.

Use `AcquireProcessShell` as the shared wrapper for those views so the back affordance, section label, and right-aligned status/actions stay consistent before adding view-specific body content.

In the Acquire routed export flow, `ExportPanelComponent` should hide its own triage tab because triage is exposed as a dedicated `AcquireTriageView` route.

### Activity & Progress

- `ActivityPanel` - Activity timeline panel
- `SimpleActivityPanel` - Compact activity panel
- `ActivityCard` - Individual activity card with EMA-smoothed progress tracking via `createProgressTracker()` (throughput, smoothed ETA, time remaining)

### Status & Feedback

- \`StatusBar\` - Status bar with system stats, progress items, quick actions
- \`ProgressModal\` - Global progress modal
- \`ProjectCloseModal\` - Step-by-step project close status modal for save/drain/checkpoint/close
- \`Toast\` - Toast notification system
- \`Skeleton\` - Loading skeleton components

### UI Enhancement

- \`ErrorBoundary\` - Error boundary with fallback UI
- \`Tooltip\` - Custom styled tooltips
- \`Transition\` - Animation transitions (Fade, SlideUp, Collapse)
- \`ThemeSwitcher\` - Light/dark theme toggle
- \`EmptyState\` - Empty state placeholders
- \`DragDrop\` - Drag and drop file upload zone

### Commands & Shortcuts

- \`CommandPalette\` - Global command palette (Cmd+K)
- \`KeyboardShortcutsModal\` - Keyboard shortcuts reference modal
- \`QuickActionsBar\` - Quick actions toolbar

### Settings & Configuration

- `SettingsPanel` - Application settings panel (10 tabs: Workspace Mode, Appearance, Behavior, Defaults, Performance, Reports, Keyboard Shortcuts, Users & Profiles, About)
- `WorkspaceModeSelector` - Toolbar dropdown for quick workspace mode switching
- `SearchPanel` - Global search panel
- `Onboarding` - First-run onboarding wizard
- `BookmarksPanel` - Evidence bookmarks panel
- `NotesPanel` - Notes list with CRUD and priority/tags
- `CaseDocumentsPanel` - Case documents panel
- `UserConfirmModal` - Profile confirmation modal on project open/create

### Filtering

- \`TypeFilterBar\` - Evidence type filter bar
- \`FilterPresetsDropdown\` - Filter preset selector

### Tab Management

- \`CenterPaneTabs\` - Center pane tab management
- \`TabBar\` - Tab and view mode controls

### Context Menus

- \`ContextMenu\` - Right-click context menu system

### Performance

- \`VirtualList\` - Virtualized list rendering for large datasets
- \`PerformancePanel\` - Performance monitoring panel

## Icons

All icons are centralized in \`icons/index.tsx\` using Heroicons outline style from \`solid-icons/hi\`.

### Canonical Feature → Icon Mapping

Each feature uses **one canonical outline icon** everywhere it appears (sidebar, command palette, quick actions, modal headers):

| Feature | Icon Component | QuickActions Key | Context Menu Emoji |
|---------|---------------|------------------|-------------------|
| Evidence Containers | `HiOutlineArchiveBox` | — | — |
| Evidence Collection | `HiOutlineArchiveBoxArrowDown` | `evidence` | 📦 |
| Generate Report | `HiOutlineClipboardDocumentList` | `report` | 📝 |
| Open Project | `HiOutlineDocumentCheck` | — | — |
| Compute Hash | `HiOutlineFingerPrint` | `fingerprint` | — |
| Search | `HiOutlineMagnifyingGlass` | `search` | — |
| Export | `HiOutlineArrowUpTray` | `export` | 📤 |
| Verify | `HiOutlineCheckBadge` | `verify` | — |
| Settings | `HiOutlineCog6Tooth` | `settings` | ⚙️ |
| Bookmarks | `HiOutlineBookmark` | `bookmark` | 📑 |
| Activity Timeline | `HiOutlineClock` | — | — |
| Case Documents | `HiOutlineClipboardDocumentList` | — | — |
| Processed DBs | `HiOutlineChartBar` | `chart` | — |
| Command Palette | `HiOutlineCommandLine` | — | 🔧 |
| Deduplication | `HiOutlineDocumentDuplicate` | `duplicate` | — |
| Performance | `HiOutlineBolt` | `bolt` | — |
| Lock/Encryption | `HiOutlineLockClosed` | — | — |

**Rules:**
- All icon imports go through `src/components/icons/index.tsx` — never import directly from `solid-icons/hi`
- Only `HiOutline*` variants are used — no `HiSolid*` or `HiMini*`
- Context menus use emoji strings (ContextMenuItem.icon is `string`)
- Modal headers, sidebar buttons, command palette, and QuickActionsBar use JSX icon components

## Styling

Components rely on global styles in `src/App.css` and `src/index.css`.

Acquire-specific views use Tailwind utility classes and standard `index.css` component classes. Acquire layout/shell styles live in `acquire/acquire.css`.

### Theme Support

The app supports light and dark themes:

- Dark theme (default)
- Light theme via \`[data-theme="light"]\` on \`<html>\`
- System preference detection via \`prefers-color-scheme\`

## Conventions

- SolidJS reactive patterns (\`createSignal\`, \`createMemo\`, \`createEffect\`)
- Typed props and exports
- Small, composable components
- Accessibility: ARIA attributes, keyboard navigation, focus management
- Consistent icon usage from centralized \`icons/index.tsx\`
