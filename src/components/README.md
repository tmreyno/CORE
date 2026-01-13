# CORE-FFX Components

SolidJS UI components for CORE-FFX.

## Directory Structure

```
components/
|-- Breadcrumb.tsx             # Navigation breadcrumb trail
|-- CommandPalette.tsx         # Global command palette (Cmd+K)
|-- ContainerEntryViewer.tsx   # Read internal container entries (AD1)
|-- ContextMenu.tsx            # Right-click context menus
|-- DetailPanel.tsx            # Tabbed detail panel for active files
|-- DetailPanelContent.tsx     # Container info rendering
|-- DragDrop.tsx               # Drag and drop file upload zone
|-- EmptyState.tsx             # Empty state placeholders
|-- ErrorBoundary.tsx          # Error boundary with fallback UI
|-- EvidenceTreeLazy.tsx       # Lazy-loaded AD1 file tree
|-- FilePanel.tsx              # Evidence file list
|-- FileRow.tsx                # File list row
|-- HexViewer.tsx              # Hex dump viewer
|-- KeyboardShortcutsModal.tsx # Keyboard shortcuts reference modal
|-- MetadataPanel.tsx          # Parsed metadata / hex navigation
|-- Onboarding.tsx             # First-run onboarding wizard
|-- ProcessedDatabasePanel.tsx # Processed DB list
|-- ProcessedDetailPanel.tsx   # Processed DB details
|-- ProgressModal.tsx          # Global progress modal
|-- ProjectSetupWizard.tsx     # Project setup wizard
|-- SearchPanel.tsx            # Global search panel
|-- SettingsPanel.tsx          # Application settings panel
|-- Skeleton.tsx               # Loading skeleton components
|-- StatusBar.tsx              # Status bar with system stats
|-- TabBar.tsx                 # Tab and view mode controls
|-- TextViewer.tsx             # Text viewer
|-- ThemeSwitcher.tsx          # Light/dark theme toggle
|-- Toast.tsx                  # Toast notification system
|-- Toolbar.tsx                # Primary toolbar
|-- Tooltip.tsx                # Custom styled tooltips
|-- Transition.tsx             # Animation transitions
|-- TreePanel.tsx              # UFED associated file tree
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

- \`TabBar\` - Tab and view mode controls
- \`TreePanel\` - UFED associated file tree
- \`EvidenceTreeLazy\` - Lazy-loaded AD1 file tree
- \`Breadcrumb\` - Navigation breadcrumb trail

### Evidence List

- \`FilePanel\` - Evidence file list panel
- \`FileRow\` - Individual file row component

### Viewers

- \`DetailPanel\` - Tabbed detail panel for active files
- \`HexViewer\` - Hex dump viewer with highlighting
- \`TextViewer\` - Plain text viewer
- \`ContainerEntryViewer\` - Internal container entries (AD1)

### Metadata

- \`MetadataPanel\` - Parsed metadata / hex navigation
- \`DetailPanelContent\` - Container info rendering

### Project & Reports

- \`ProjectSetupWizard\` - Project setup wizard
- \`ReportWizard\` - Report generation wizard

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

### Settings & Configuration

- \`SettingsPanel\` - Application settings panel
- \`SearchPanel\` - Global search panel
- \`Onboarding\` - First-run onboarding wizard

### Context Menus

- \`ContextMenu\` - Right-click context menu system

### Performance

- \`VirtualList\` - Virtualized list rendering for large datasets
- \`VirtualTree\` - Virtualized tree rendering

## Icons

All icons are centralized in \`icons/index.tsx\` using Heroicons outline style from \`solid-icons/hi\`.

## Styling

Components rely on global styles in \`src/App.css\` and \`src/index.css\`.

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
