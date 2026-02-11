# Project Management Components

Enterprise-grade project management UI components for CORE-FFX forensic investigations.

## Components

### RecoveryModal

**File**: `RecoveryModal.tsx` (300 lines)  
**Purpose**: Backup management, crash recovery, and project health monitoring

**Features**:

- **Backups Tab**: List/create/restore/clean backups
- **Health Tab**: Project diagnostics with health score and recommendations
- Backup type badges (manual/auto/scheduled)
- File size and version tracking
- Color-coded health status (healthy/warning/critical)

**Usage**:

```tsx
import RecoveryModal from "./components/project/RecoveryModal";

const [showRecovery, setShowRecovery] = createSignal(false);

<RecoveryModal
  isOpen={showRecovery()}
  onClose={() => setShowRecovery(false)}
  projectPath="/path/to/project.cffx"
/>

```

**Props**:

- `isOpen: boolean` - Control modal visibility
- `onClose: () => void` - Close callback
- `projectPath: string` - Path to .cffx file

---

### ProfileSelector

**File**: `ProfileSelector.tsx` (250 lines)  
**Purpose**: Quick workspace layout profile switching

**Features**:

- 8 profile types with color coding
- Save current workspace dialog
- Apply/delete profile actions
- Use count and last used tracking
- Dropdown interface with search

**Profile Types**:

- **investigation** → `text-type-ad1`
- **analysis** → `text-type-e01`
- **reporting** → `text-accent`
- **review** → `text-warning`
- **triage** → `text-info`
- **acquisition** → `text-success`
- **preservation** → `text-type-ufed`
- **custom** → `text-txt-secondary`

**Usage**:

```tsx
import ProfileSelector from "./components/project/ProfileSelector";

<ProfileSelector
  onProfileChange={(profileId) => {
    console.log("Profile changed:", profileId);
  }}
/>

```

**Props**:

- `onProfileChange?: (profileId: string) => void` - Profile change callback

---

### TemplateGallery

**File**: `TemplateGallery.tsx` (580 lines)  
**Purpose**: Project template browser and management

**Features**:

- Grid/list view toggle
- Search and category filtering
- Preview template contents
- Apply template to current project
- Create template from project
- Import/export templates
- Built-in vs custom templates

**Categories**:

- Investigation
- Malware Analysis
- Incident Response
- Mobile Forensics
- Data Breach
- Custom

**Usage**:

```tsx
import TemplateGallery from "./components/project/TemplateGallery";

const [showTemplates, setShowTemplates] = createSignal(false);

<TemplateGallery
  isOpen={showTemplates()}
  onClose={() => setShowTemplates(false)}
  onTemplateApplied={(templateId) => {
    console.log("Applied template:", templateId);
  }}
/>

```

**Props**:

- `isOpen: boolean` - Control modal visibility
- `onClose: () => void` - Close callback
- `onTemplateApplied?: (templateId: string) => void` - Template applied callback

---

### ActivityHeatmap

**File**: `ActivityHeatmap.tsx` (380 lines)  
**Purpose**: 7×24 activity pattern visualization

**Features**:

- 7-day × 24-hour heatmap grid
- Color-coded activity intensity (4 levels)
- Most active days panel
- Trend analysis (peak hour/day, daily average, trend direction)
- Click cells to view details
- Export timeline data
- Hover tooltips

**Usage**:

```tsx
import ActivityHeatmap from "./components/project/ActivityHeatmap";

const [showHeatmap, setShowHeatmap] = createSignal(false);

<ActivityHeatmap
  isOpen={showHeatmap()}
  onClose={() => setShowHeatmap(false)}
  projectPath="/path/to/project.cffx"
/>

```

**Props**:

- `isOpen: boolean` - Control modal visibility
- `onClose: () => void` - Close callback
- `projectPath: string` - Path to .cffx file

---

### ComparisonView

**File**: `ComparisonView.tsx` (630 lines)  
**Purpose**: Side-by-side project comparison and merging

**Features**:

- 3-column layout (Only in A, Common, Only in B)
- 4 tabs: Bookmarks, Notes, Evidence, Activity
- Similarity score calculation
- Conflict detection and highlighting
- 5 merge strategies
- Sync bookmarks/notes actions
- Modified item indicators

**Merge Strategies**:

- **prefer_a**: Keep items from Project A on conflicts
- **prefer_b**: Keep items from Project B on conflicts
- **keep_both**: Keep all items from both projects
- **skip**: Skip conflicting items
- **manual**: Mark conflicts for manual review

**Usage**:

```tsx
import ComparisonView from "./components/project/ComparisonView";

const [showComparison, setShowComparison] = createSignal(false);

<ComparisonView
  isOpen={showComparison()}
  onClose={() => setShowComparison(false)}
  projectPathA="/path/to/projectA.cffx"
  projectPathB="/path/to/projectB.cffx"
  projectNameA="Investigation A"
  projectNameB="Investigation B"
/>

```

**Props**:

- `isOpen: boolean` - Control modal visibility
- `onClose: () => void` - Close callback
- `projectPathA: string` - Path to first project
- `projectPathB: string` - Path to second project
- `projectNameA?: string` - Display name for first project
- `projectNameB?: string` - Display name for second project

---

## Hooks Integration

All components use custom hooks for data management:

### useProjectRecovery

- `listBackups()` - Get all backups
- `restoreBackup(path)` - Restore from backup
- `createBackup()` - Manual backup
- `checkHealth()` - Run diagnostics
- `cleanBackups(days)` - Remove old backups

### useWorkspaceProfiles

- `listProfiles()` - Get all profiles
- `loadProfile(id)` - Get profile details
- `saveProfile(name, type, desc)` - Save current workspace
- `applyProfile(id)` - Apply profile layout
- `deleteProfile(id)` - Remove profile

### useProjectTemplates

- `listTemplates()` - Get all templates
- `getTemplate(id)` - Get full template
- `applyTemplate(projectPath, templateId)` - Apply to project
- `createFromProject(projectPath, name, category)` - Create template
- `exportTemplate(id, path)` - Export to file
- `importTemplate(path)` - Import from file

### useActivityTimeline

- `getVisualization(projectPath)` - Get timeline data
- `exportTimeline(projectPath, path)` - Export timeline
- `getHeatmapData()` - Get 7×24 grid data
- `getMostActivePeriods()` - Get top active days
- `getTrends()` - Get trend analysis

### useProjectComparison

- `compareProjects(pathA, pathB)` - Compare two projects
- `mergeProjects(pathA, pathB, strategy)` - Merge projects
- `syncBookmarks(pathA, pathB)` - Sync bookmarks only
- `syncNotes(pathA, pathB)` - Sync notes only
- `getUniqueToA/B()` - Get unique items
- `getCommonItems()` - Get shared items
- `getModifiedItems()` - Get changed items
- `getConflicts()` - Get merge conflicts

---

## Common Patterns

### Modal Pattern

All components follow the same modal pattern:

```tsx
const [isOpen, setIsOpen] = createSignal(false);

// Open button
<button onClick={() => setIsOpen(true)}>
  Open Modal
</button>

// Modal component
<Component
  isOpen={isOpen()}
  onClose={() => setIsOpen(false)}
  // ... other props
/>

```

### Loading States

Components show loading indicators while fetching data:

```tsx
<Show when={!loading()} fallback={<LoadingSpinner />}>
  {/* Content */}
</Show>

```

### Error Handling

Hooks expose error state:

```tsx
const hook = useProjectRecovery();
<Show when={hook.error()}>
  <div className="text-error">{hook.error()}</div>
</Show>

```

---

## Styling

All components use the design system from `src/styles/variables.css`:

**Background Colors**:

- `bg-bg` - Main background
- `bg-bg-secondary` - Secondary panels
- `bg-bg-panel` - Modal/panel backgrounds
- `bg-bg-hover` - Hover states

**Text Colors**:

- `text-txt` - Primary text
- `text-txt-secondary` - Secondary text
- `text-txt-muted` - Muted text

**Accent Colors**:

- `text-accent` - Accent highlights
- `text-success` - Success states
- `text-warning` - Warning states
- `text-error` - Error states

**Icon Sizes**:

- `w-icon-sm h-icon-sm` - 16px
- `w-icon-base h-icon-base` - 20px
- `w-icon-lg h-icon-lg` - 24px

**Z-Index**:

- `z-dropdown` - Dropdown menus
- `z-modal-backdrop` - Modal backdrop
- `z-modal` - Modal content
- `z-tooltip` - Tooltips

---

## Integration Example

```tsx
// App.tsx or ProjectView.tsx
import { createSignal } from "solid-js";
import RecoveryModal from "./components/project/RecoveryModal";
import ProfileSelector from "./components/project/ProfileSelector";
import TemplateGallery from "./components/project/TemplateGallery";
import ActivityHeatmap from "./components/project/ActivityHeatmap";
import ComparisonView from "./components/project/ComparisonView";

function ProjectManager() {
  const [showRecovery, setShowRecovery] = createSignal(false);
  const [showTemplates, setShowTemplates] = createSignal(false);
  const [showHeatmap, setShowHeatmap] = createSignal(false);
  const [showComparison, setShowComparison] = createSignal(false);
  
  const currentProject = "/path/to/current.cffx";

  return (
    <div className="p-4 space-y-4">
      {/* Toolbar */}
      <div className="flex items-center gap-3">
        <ProfileSelector
          onProfileChange={(id) => console.log("Profile:", id)}
        />
        
        <button
          onClick={() => setShowRecovery(true)}
          className="px-3 py-2 bg-accent hover:bg-accent-hover text-white rounded-md"
        >
          Backup & Health
        </button>
        
        <button
          onClick={() => setShowTemplates(true)}
          className="px-3 py-2 bg-bg-secondary hover:bg-bg-hover text-txt rounded-md"
        >
          Templates
        </button>
        
        <button
          onClick={() => setShowHeatmap(true)}
          className="px-3 py-2 bg-bg-secondary hover:bg-bg-hover text-txt rounded-md"
        >
          Activity
        </button>
        
        <button
          onClick={() => setShowComparison(true)}
          className="px-3 py-2 bg-bg-secondary hover:bg-bg-hover text-txt rounded-md"
        >
          Compare
        </button>
      </div>

      {/* Modals */}
      <RecoveryModal
        isOpen={showRecovery()}
        onClose={() => setShowRecovery(false)}
        projectPath={currentProject}
      />

      <TemplateGallery
        isOpen={showTemplates()}
        onClose={() => setShowTemplates(false)}
        onTemplateApplied={(id) => {
          console.log("Applied:", id);
          setShowTemplates(false);
        }}
      />

      <ActivityHeatmap
        isOpen={showHeatmap()}
        onClose={() => setShowHeatmap(false)}
        projectPath={currentProject}
      />

      <ComparisonView
        isOpen={showComparison()}
        onClose={() => setShowComparison(false)}
        projectPathA={currentProject}
        projectPathB="/path/to/other.cffx"
        projectNameA="Current Investigation"
        projectNameB="Previous Case"
      />
    </div>
  );
}

export default ProjectManager;

```

---

## Backend Integration

Components require corresponding Tauri commands to be registered in `src-tauri/src/lib.rs`:

```rust
tauri::generate_handler![
    // Project recovery
    project_list_backups,
    project_restore_backup,
    project_create_backup,
    project_check_health,
    project_list_versions,
    project_clean_backups,
    
    // Workspace profiles
    profile_list,
    profile_load,
    profile_save,
    profile_apply,
    profile_delete,
    
    // Project templates
    template_list,
    template_get,
    template_apply,
    template_create_from_project,
    template_delete,
    template_export,
    template_import,
    
    // Activity timeline
    timeline_get_visualization,
    timeline_export,
    timeline_get_patterns,
    
    // Project comparison
    project_compare,
    project_merge,
    project_sync_bookmarks,
    project_sync_notes,
]

```

---

## Testing

Test components with mock data:

```tsx
import { createSignal } from "solid-js";

// Mock hook for testing
function useMockRecovery() {
  const [backups] = createSignal([
    {
      path: "/backup1.bak",
      timestamp: new Date().toISOString(),
      size_bytes: 1024000,
      backup_type: "auto",
      version: "1.0",
    }
  ]);
  
  return {
    backups,
    loading: () => false,
    error: () => null,
    listBackups: async () => {},
    restoreBackup: async (path: string) => true,
  };
}

```

---

## Performance Considerations

1. **Lazy Loading**: Components only load data when opened
2. **Pagination**: Large lists use pagination (template list, backup list)
3. **Memoization**: Expensive computations are memoized with `createMemo`
4. **Debouncing**: Search inputs should be debounced (300ms recommended)
5. **Virtual Scrolling**: Consider for lists >100 items

---

## Accessibility

Components include:

- Semantic HTML elements
- Keyboard navigation support
- Focus management in modals
- ARIA labels (can be enhanced)
- Color contrast compliance

Future enhancements:

- Add `aria-label` attributes
- Improve keyboard shortcuts
- Screen reader announcements
- Focus trapping in modals

---

## Browser Support

- Chrome 90+
- Firefox 88+
- Safari 14+
- Edge 90+

Uses modern CSS features:

- CSS Grid
- Flexbox
- CSS Custom Properties
- backdrop-filter (for modal backdrops)

---

## File Structure

```text
src/components/project/
├── README.md                 # This file
├── RecoveryModal.tsx         # Backup & health management
├── ProfileSelector.tsx       # Workspace profile switcher
├── TemplateGallery.tsx       # Template browser
├── ActivityHeatmap.tsx       # Activity visualization
└── ComparisonView.tsx        # Project comparison

```

Related files:

```text
src/hooks/
├── useProjectRecovery.ts
├── useWorkspaceProfiles.ts
├── useProjectTemplates.ts
├── useActivityTimeline.ts
└── useProjectComparison.ts

src-tauri/src/
├── project_recovery.rs
├── workspace_profiles.rs
├── project_templates.rs
├── activity_timeline.rs
└── project_comparison.rs

```
