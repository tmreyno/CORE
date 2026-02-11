# Quick Start Integration Guide

## Step 1: Add to Your Main App

### Option A: Use the Integrated Toolbar (Recommended)

```tsx
// src/App.tsx or src/views/ProjectView.tsx
import { Component, createSignal } from "solid-js";
import ProjectToolbar from "./components/project/ProjectToolbar";
import { useProject } from "./hooks/useProject"; // Your existing hook

const ProjectView: Component = () => {
  const project = useProject();
  
  // Get current project path from your state management
  const currentProjectPath = () => project.projectInfo()?.path || "";
  const currentProjectName = () => project.projectInfo()?.name || "Untitled Project";
  
  return (
    <div class="h-screen flex flex-col bg-bg">
      {/* Add the toolbar at the top */}
      <ProjectToolbar
        currentProjectPath={currentProjectPath()}
        currentProjectName={currentProjectName()}
      />
      
      {/* Your existing project content below */}
      <div class="flex-1 overflow-auto">
        {/* Evidence explorer, file viewer, etc. */}
      </div>
    </div>
  );
};

export default ProjectView;
```

### Option B: Use Individual Components

```tsx
// src/App.tsx
import { Component, createSignal } from "solid-js";
import RecoveryModal from "./components/project/RecoveryModal";
import ProfileSelector from "./components/project/ProfileSelector";
import { useProject } from "./hooks/useProject";

const App: Component = () => {
  const project = useProject();
  const [showRecovery, setShowRecovery] = createSignal(false);
  
  return (
    <div class="app">
      {/* Add to your existing toolbar/menu */}
      <header class="flex items-center gap-3 p-4">
        <ProfileSelector
          onProfileChange={(profileId) => {
            console.log("Switched to profile:", profileId);
          }}
        />
        
        <button
          onClick={() => setShowRecovery(true)}
          class="px-3 py-2 bg-accent hover:bg-accent-hover text-white rounded-md"
        >
          Backup & Health
        </button>
      </header>
      
      {/* Modals */}
      <RecoveryModal
        isOpen={showRecovery()}
        onClose={() => setShowRecovery(false)}
        projectPath={project.projectInfo()?.path || ""}
      />
      
      {/* Your app content */}
    </div>
  );
};
```

---

## Step 2: Update Icon Paths (If Needed)

All components import from `../icons`. If your icon file is elsewhere:

```tsx
// In each component, update the import:
// FROM:
import { HiOutlineFolder } from "../icons";

// TO:
import { HiOutlineFolder } from "../../components/icons";
// Or wherever your icons/index.tsx is located
```

---

## Step 3: Verify Backend Commands

Ensure all 30 commands are registered in `src-tauri/src/lib.rs`:

```rust
// src-tauri/src/lib.rs
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            // PROJECT RECOVERY (6 commands)
            project_recovery::project_list_backups,
            project_recovery::project_restore_backup,
            project_recovery::project_create_backup,
            project_recovery::project_check_health,
            project_recovery::project_list_versions,
            project_recovery::project_clean_backups,
            
            // WORKSPACE PROFILES (10 commands)
            workspace_profiles::profile_list,
            workspace_profiles::profile_load,
            workspace_profiles::profile_save,
            workspace_profiles::profile_apply,
            workspace_profiles::profile_delete,
            workspace_profiles::profile_import,
            workspace_profiles::profile_export,
            workspace_profiles::profile_get_by_type,
            workspace_profiles::profile_update_last_used,
            workspace_profiles::profile_update_use_count,
            
            // PROJECT TEMPLATES (7 commands)
            project_templates::template_list,
            project_templates::template_get,
            project_templates::template_apply,
            project_templates::template_create_from_project,
            project_templates::template_delete,
            project_templates::template_export,
            project_templates::template_import,
            
            // ACTIVITY TIMELINE (3 commands)
            activity_timeline::timeline_get_visualization,
            activity_timeline::timeline_export,
            activity_timeline::timeline_get_patterns,
            
            // PROJECT COMPARISON (4 commands)
            project_comparison::project_compare,
            project_comparison::project_merge,
            project_comparison::project_sync_bookmarks,
            project_comparison::project_sync_notes,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

## Step 4: Test the Integration

```bash
# Start dev server
npm run tauri dev

# Test each feature:
# 1. Click "Backup & Health" - should open RecoveryModal
# 2. Click profile dropdown - should show profiles
# 3. Click "Templates" - should open TemplateGallery
# 4. Click "Activity" - should open ActivityHeatmap
# 5. Click "Compare" - should open ComparisonView
```

---

## Step 5: Handle Real Data

### Connect to Your Project State

```tsx
import { useProject } from "./hooks/useProject";

const MyComponent = () => {
  const project = useProject();
  
  // Use real project path
  const projectPath = () => {
    const info = project.projectInfo();
    return info?.path || "";
  };
  
  return (
    <RecoveryModal
      isOpen={showModal()}
      onClose={() => setShowModal(false)}
      projectPath={projectPath()}
    />
  );
};
```

### Connect Comparison Project

```tsx
// For ComparisonView, let user select second project
import { open } from "@tauri-apps/plugin-dialog";

const selectComparisonProject = async () => {
  const selected = await open({
    filters: [{
      name: "FFX Project",
      extensions: ["ffxproj"]
    }]
  });
  
  if (selected) {
    setComparisonPath(selected);
  }
};
```

---

## Common Issues & Solutions

### Issue 1: Icons Not Found

**Error**: `Cannot find module '../icons'`

**Solution**: Update import paths in all components:
```tsx
// Change FROM:
import { HiOutlineFolder } from "../icons";

// TO:
import { HiOutlineFolder } from "../../components/icons";
```

### Issue 2: TypeScript Errors for className

**Error**: `Property 'className' does not exist on type 'IconProps'`

**Solution**: Use `class` instead of `className`:
```tsx
// Change FROM:
<HiOutlineFolder className="w-icon-sm h-icon-sm" />

// TO:
<HiOutlineFolder class="w-icon-sm h-icon-sm" />
```

### Issue 3: Backend Commands Not Found

**Error**: `failed to invoke command: project_list_backups`

**Solution**: Register the command in `lib.rs`:
```rust
.invoke_handler(tauri::generate_handler![
    project_recovery::project_list_backups,
    // ... other commands
])
```

### Issue 4: Modal Not Showing

**Checklist**:
1. ✅ `isOpen` prop is true?
2. ✅ Modal has `z-modal` class?
3. ✅ Backdrop has `z-modal-backdrop` class?
4. ✅ Check browser DevTools for errors

---

## Customization Examples

### Change Profile Colors

```tsx
// In ProfileSelector.tsx, update getCategoryColor():
const getCategoryColor = (type: string) => {
  switch (type) {
    case "investigation":
      return "text-blue-500";  // Your custom color
    case "analysis":
      return "text-green-500";
    // ...
  }
};
```

### Add Custom Template Category

```tsx
// In TemplateGallery.tsx, add to categories array:
const categories = [
  // ... existing categories
  { value: "my_category", label: "My Category" },
];
```

### Customize Heatmap Colors

```tsx
// In ActivityHeatmap.tsx, update getIntensityColor():
const getIntensityColor = (count: number, maxCount: number) => {
  if (count === 0) return "bg-bg";
  const intensity = count / maxCount;
  
  // Your custom gradient
  if (intensity > 0.75) return "bg-purple-600";
  if (intensity > 0.5) return "bg-purple-400";
  if (intensity > 0.25) return "bg-purple-200";
  return "bg-purple-100";
};
```

---

## Performance Tips

### 1. Lazy Load Modals

```tsx
// Only import when needed
const RecoveryModal = lazy(() => import("./components/project/RecoveryModal"));

<Suspense fallback={<LoadingSpinner />}>
  <RecoveryModal isOpen={show()} onClose={close} />
</Suspense>
```

### 2. Debounce Search Inputs

```tsx
import { createSignal } from "solid-js";
import { debounce } from "@solid-primitives/scheduled";

const [search, setSearch] = createSignal("");
const debouncedSearch = debounce((value: string) => {
  // Perform search
}, 300);

<input
  onInput={(e) => {
    setSearch(e.currentTarget.value);
    debouncedSearch(e.currentTarget.value);
  }}
/>
```

### 3. Virtualize Long Lists

For lists with >100 items, use virtualization:

```tsx
import { VirtualList } from "@solid-primitives/virtual";

<VirtualList
  items={largeItemList()}
  itemHeight={40}
  overscan={10}
>
  {(item) => <ItemRow item={item} />}
</VirtualList>
```

---

## Next Steps

1. ✅ **Test with real data** - Use actual project files
2. ✅ **Add keyboard shortcuts** - Ctrl+B for backup, etc.
3. ✅ **Add toast notifications** - Success/error messages
4. ✅ **Improve accessibility** - ARIA labels, focus management
5. ✅ **Write tests** - Unit tests for hooks, integration tests

---

## Support

- **Component Docs**: `src/components/project/README.md`
- **Implementation Details**: `src/components/project/IMPLEMENTATION_SUMMARY.md`
- **Backend Docs**: `src-tauri/src/README.md`
- **Design System**: `src/styles/README.md`

---

## Quick Reference

### All Components

```tsx
import RecoveryModal from "./components/project/RecoveryModal";
import ProfileSelector from "./components/project/ProfileSelector";
import TemplateGallery from "./components/project/TemplateGallery";
import ActivityHeatmap from "./components/project/ActivityHeatmap";
import ComparisonView from "./components/project/ComparisonView";
import ProjectToolbar from "./components/project/ProjectToolbar";
```

### All Hooks

```tsx
import { useProjectRecovery } from "./hooks/useProjectRecovery";
import { useWorkspaceProfiles } from "./hooks/useWorkspaceProfiles";
import { useProjectTemplates } from "./hooks/useProjectTemplates";
import { useActivityTimeline } from "./hooks/useActivityTimeline";
import { useProjectComparison } from "./hooks/useProjectComparison";
```

### Component Props Quick Reference

```tsx
// RecoveryModal
<RecoveryModal
  isOpen={boolean}
  onClose={() => void}
  projectPath={string}
/>

// ProfileSelector
<ProfileSelector
  onProfileChange={(id: string) => void}
/>

// TemplateGallery
<TemplateGallery
  isOpen={boolean}
  onClose={() => void}
  onTemplateApplied={(id: string) => void}
/>

// ActivityHeatmap
<ActivityHeatmap
  isOpen={boolean}
  onClose={() => void}
  projectPath={string}
/>

// ComparisonView
<ComparisonView
  isOpen={boolean}
  onClose={() => void}
  projectPathA={string}
  projectPathB={string}
  projectNameA?={string}
  projectNameB?={string}
/>

// ProjectToolbar
<ProjectToolbar
  currentProjectPath={string}
  comparisonProjectPath?={string}
  currentProjectName?={string}
  comparisonProjectName?={string}
/>
```

---

**Ready to go! 🚀**
