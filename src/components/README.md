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
|-- ArchiveTreeNode (in EvidenceTree) # Archive tree node
|-- BookmarksPanel.tsx         # Evidence bookmarks panel
|-- Breadcrumb.tsx             # Navigation breadcrumb trail
|-- CaseDocumentsPanel.tsx     # Case documents panel
|-- CenterPaneTabs.tsx         # Center pane tab management
|-- CommandPalette.tsx         # Global command palette (Cmd+K)
|-- ContainerEntryViewer.tsx   # Read internal container entries (AD1)
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
|-- ExportPanel.tsx            # Evidence export panel
|-- FilePanel.tsx              # Evidence file list
|-- FileRow.tsx                # File list row
|-- FilterPresetsDropdown.tsx  # Filter preset selector
|-- HashBadge.tsx              # Hash verification badge
|-- LinkedDataTree.tsx         # Reusable linked data tree (collections↔COC↔evidence)
|-- LinkedDataPanel.tsx        # Right-panel wrapper for linked data tree
|-- HexViewer.tsx              # Hex dump viewer
|-- ImageViewer.tsx            # Image viewer with EXIF support
|-- KeyboardShortcutsModal.tsx # Keyboard shortcuts reference modal
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
|-- SettingsPanel.tsx          # Application settings panel
|-- SimpleActivityPanel.tsx    # Compact activity panel
|-- Skeleton.tsx               # Loading skeleton components
|-- SpreadsheetViewer.tsx      # Spreadsheet viewer
|-- StatusBar.tsx              # Status bar with system stats
|-- TabBar.tsx                 # Tab and view mode controls
|-- TextViewer.tsx             # Text viewer
|-- ThemeSwitcher.tsx          # Light/dark theme toggle
|-- Toast.tsx                  # Toast notification system
|-- Toolbar.tsx                # Primary toolbar
|-- Tooltip.tsx                # Custom styled tooltips
|-- Transition.tsx             # Animation transitions
|-- TreePanel.tsx              # UFED associated file tree
|-- TypeFilterBar.tsx          # Evidence type filter bar
|-- VirtualList.tsx            # Virtualized list for large datasets
|-- icons/
|   |-- index.tsx               # Centralized icon exports
|-- report/
|   |-- ReportWizard.tsx        # Report wizard UI
|   |-- index.ts                # Barrel exports
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

### Evidence Operations

- `ExportPanel` - Evidence export panel (copy, 7z archives, E01 images, L01 logical evidence)

### Acquire Layout Rule

Acquire process views already sit inside the Acquire layout shell. `AcquireExportView`, `AcquireVerifyView`, `AcquireTriageView`, and the export modes rendered inside them should use Tailwind utility classes and standard `index.css` component classes (e.g., `.callout`, `.card`, `.btn`) rather than reintroducing nested desktop-style `card`, `info-card`, or toolbar framing around each step.

Use `AcquireProcessShell` as the shared wrapper for those views so the back affordance, section label, and right-aligned status/actions stay consistent before adding view-specific body content.

In the Acquire routed export flow, `ExportPanelComponent` should hide its own triage tab because triage is exposed as a dedicated `AcquireTriageView` route.

### Activity & Progress

- \`ActivityPanel\` - Activity timeline panel
- `SimpleActivityPanel` - Compact activity panel

### Status & Feedback

- \`StatusBar\` - Status bar with system stats, progress items, quick actions
- \`ProgressModal\` - Global progress modal
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

- \`SettingsPanel\` - Application settings panel
- \`SearchPanel\` - Global search panel
- \`Onboarding\` - First-run onboarding wizard
- \`BookmarksPanel\` - Evidence bookmarks panel
- \`CaseDocumentsPanel\` - Case documents panel

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

Acquire-specific views use Tailwind utility classes and standard `index.css` component classes — no custom CSS files.

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
