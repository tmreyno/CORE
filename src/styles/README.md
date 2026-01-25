# CORE-FFX Styling Guide

CSS architecture using Tailwind CSS with CSS custom properties for theming.

> **Quick Reference:** See [UI Standards Quick Reference](#ui-standards-quick-reference) for component patterns.

## Architecture

```text
┌─────────────────────────────────────────────────────────────────┐
│                    Style Pipeline                                │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  src/styles/variables.css   ──┐                                  │
│  (Design Tokens)              │                                  │
│                               ▼                                  │
│  tailwind.config.js    ───────┼───►  PostCSS  ───►  Bundle      │
│  (Theme Extension)            │                                  │
│                               │                                  │
│  src/index.css         ───────┘                                  │
│  (Component Classes)                                             │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## File Structure

```text
/                           # Root
├── tailwind.config.js      # Tailwind theme extension
├── postcss.config.js       # PostCSS plugins
│
src/
├── index.css               # Base styles + component classes (@layer)
├── App.css                 # App-specific styles + font imports
└── styles/
    └── variables.css       # CSS custom properties (design tokens)
```

---

## UI Standards Quick Reference

### Border Radius Standards

| Component Type | Class | Use Case |
|----------------|-------|----------|
| **Buttons** | `rounded-lg` | All buttons (btn, btn-sm, icon-btn) |
| **Inputs** | `rounded-lg` | Text inputs, selects, textareas |
| **Cards/Panels** | `rounded-lg` | Standard panels, info-cards |
| **Modals/Dialogs** | `rounded-xl` | Modal containers, command palette |
| **Toasts** | `rounded-xl` | Toast notifications |
| **Badges/Pills** | `rounded` | Small inline badges |
| **Chips** | `rounded` | Clickable status chips |
| **Dropdowns** | `rounded-lg` | Context menus, popovers |
| **Large Hero** | `rounded-2xl` | Onboarding, drag-drop zones |
| **Avatars/Icons** | `rounded-full` | Circular elements |

### Padding Standards

| Component Type | Padding | Example |
|----------------|---------|---------|
| **Modal Header/Footer** | `px-5 py-4` | `.modal-header`, `.modal-footer` |
| **Modal Body** | `p-5` | `.modal-body` |
| **Panel Header** | `px-3 py-2` | `.panel-header` |
| **Card Content** | `p-4` | `.card`, `.info-card` |
| **Button Base** | `px-4 py-2` | `.btn`, `.btn-primary` |
| **Button Small** | `px-3 py-1.5` | `.btn-sm`, `.btn-action` |
| **Button Text** | `px-2 py-1` | `.btn-text`, `.btn-ghost` |
| **Icon Button** | `p-2` | `.icon-btn` |
| **Icon Button Small** | `p-1.5` | `.icon-btn-sm` |
| **Input Base** | `px-3 py-2.5` | `.input` |
| **Input Small** | `px-2.5 py-2` | `.input-sm` |
| **Input Inline** | `px-2 py-1.5` | `.input-inline` (flex layouts) |
| **Input Extra Small** | `px-1 py-0.5` | `.input-xs` (toolbar inputs) |
| **Badge** | `px-2 py-0.5` | `.badge` |
| **Chip** | `px-1.5 py-0.5` | `.chip` |
| **Toolbar** | `px-3 py-2` | `.toolbar` |

### Gap Standards

| Context | Gap | Use Case |
|---------|-----|----------|
| **Compact** | `gap-1` | Icon + text in buttons |
| **Base** | `gap-2` | Default flex layouts |
| **Relaxed** | `gap-3` | Section spacing |
| **Spacious** | `gap-4` | Form fields, card grids |

### Button Patterns

```tsx
// Primary - main actions
<button class="btn btn-primary">Save</button>

// Secondary - alternative actions  
<button class="btn btn-secondary">Cancel</button>

// Ghost - minimal emphasis
<button class="btn btn-ghost">Skip</button>

// Small action buttons
<button class="btn-action-primary">
  <Icon class="w-4 h-4" /> Export
</button>

// Icon-only buttons
<button class="icon-btn">
  <Icon class="w-5 h-5" />
</button>

// Text/link buttons
<button class="btn-text">Learn more</button>
<button class="btn-text-danger">Delete</button>
```

### Input Patterns

```tsx
// Standard input
<input class="input" placeholder="Enter value..." />

// Small input
<input class="input-sm" placeholder="Search..." />

// Textarea
<textarea class="textarea" rows="4" />

// With label
<div class="form-group">
  <label class="label">Field Name</label>
  <input class="input" />
</div>
```

### Card/Panel Patterns

```tsx
// Standard card
<div class="card">Content</div>

// Interactive card (clickable)
<div class="card-interactive">Clickable content</div>

// Info card (detail panels)
<div class="info-card">
  <div class="info-card-title">
    <Icon class="w-4 h-4" /> Title
  </div>
  Content...
</div>

// Panel with header
<div class="bg-bg-panel rounded-lg border border-border">
  <div class="panel-header">
    <span class="text-sm font-medium">Panel Title</span>
  </div>
  <div class="p-4">Content</div>
</div>
```

### Modal Patterns

```tsx
// Modal structure
<div class="modal-overlay">
  <div class="modal-content w-[500px]">
    <div class="modal-header">
      <h2 class="text-lg font-semibold">Title</h2>
      <button class="icon-btn-sm"><X /></button>
    </div>
    <div class="modal-body">Content</div>
    <div class="modal-footer justify-end">
      <button class="btn btn-secondary">Cancel</button>
      <button class="btn btn-primary">Confirm</button>
    </div>
  </div>
</div>
```

### Badge/Chip Patterns

```tsx
// Status badges
<span class="badge badge-success">Verified</span>
<span class="badge badge-warning">Pending</span>
<span class="badge badge-error">Failed</span>

// Clickable chips
<button class="chip chip-cyan">Active</button>
<button class="chip chip-neutral">Inactive</button>
```

---

## Design Tokens (`variables.css`)

Single source of truth for all design values. Edit here to change the design system.

### Colors

```css
/* Background */
--color-bg            /* Primary background (#18181b) */
--color-bg-secondary  /* Secondary panels (#27272a) */
--color-bg-panel      /* Panel backgrounds (#1f1f23) */
--color-bg-hover      /* Hover states (#3f3f46) */
--color-bg-active     /* Active states (#52525b) */

/* Text */
--color-txt           /* Primary text (#e4e4e7) */
--color-txt-secondary /* Secondary text (#a1a1aa) */
--color-txt-muted     /* Muted text (#71717a) */

/* Accent */
--color-accent        /* Primary accent (#06b6d4 - cyan) */
--color-accent-hover  /* Accent hover (#22d3ee) */

/* Status */
--color-success       /* Success green (#22c55e) */
--color-warning       /* Warning yellow (#facc15) */
--color-error         /* Error red (#ef4444) */
--color-info          /* Info blue (#3b82f6) */

/* Container Types */
--color-type-ad1      /* AD1 blue (#60a5fa) */
--color-type-e01      /* E01 green (#4ade80) */
--color-type-l01      /* L01 yellow (#facc15) */
--color-type-raw      /* Raw purple (#a78bfa) */
--color-type-ufed     /* UFED cyan (#06b6d4) */
--color-type-archive  /* Archive orange (#fb923c) */
```

### Sizing

```css
/* Icons */
--icon-size-micro     /* 12px */
--icon-size-compact   /* 14px */
--icon-size-small     /* 16px */
--icon-size-base      /* 20px */
--icon-size-lg        /* 24px */

/* Spacing */
--gap-compact         /* 4px */
--gap-small           /* 6px */
--gap-base            /* 8px */

/* Border Radius */
--radius-sm           /* 4px */
--radius-base         /* 6px */
--radius-md           /* 8px */
--radius-lg           /* 12px */
```

## Tailwind Usage

### Semantic Color Classes

Use semantic classes that reference CSS variables:

```tsx
// ✅ Good - semantic colors
<div className="bg-bg text-txt border-border" />
<div className="bg-bg-secondary text-txt-secondary" />
<div className="text-accent hover:text-accent-hover" />

// ❌ Avoid - hardcoded colors
<div className="bg-zinc-900 text-zinc-100" />
```

### Container Type Colors

```tsx
// Color by container type
<span className="text-type-ad1">AD1</span>
<span className="text-type-e01">E01</span>
<span className="text-type-ufed">UFED</span>
```

### Status Colors

```tsx
<span className="text-success">Verified</span>
<span className="text-warning">Pending</span>
<span className="text-error">Failed</span>
<span className="bg-success-soft">Success background</span>
```

### Icon Sizing

```tsx
<Icon className="w-icon-sm h-icon-sm" />   // 16px
<Icon className="w-icon-base h-icon-base" /> // 20px
<Icon className="w-icon-lg h-icon-lg" />   // 24px
```

### Spacing

```tsx
<div className="gap-compact" />  // 4px
<div className="gap-small" />    // 6px
<div className="gap-base" />     // 8px
```

## Component Classes (index.css)

The `index.css` defines reusable component classes in the `@layer components`. **Always prefer these over inline Tailwind when available:**

### Available CSS Classes

| Category | Classes |
|----------|---------|
| **Buttons** | `.btn`, `.btn-primary`, `.btn-secondary`, `.btn-ghost`, `.btn-sm`, `.btn-action`, `.btn-action-primary`, `.btn-action-secondary`, `.btn-text`, `.btn-text-danger`, `.btn-nav` |
| **Icon Buttons** | `.icon-btn`, `.icon-btn-sm` |
| **Inputs** | `.input`, `.input-sm`, `.textarea`, `.label`, `.form-group`, `.form-row` |
| **Cards** | `.card`, `.card-interactive`, `.info-card`, `.stat-card`, `.progress-card` |
| **Modals** | `.modal-overlay`, `.modal-content`, `.modal-header`, `.modal-body`, `.modal-footer` |
| **Panels** | `.panel-header`, `.panel-header-sm`, `.toolbar` |
| **Badges** | `.badge`, `.badge-accent`, `.badge-success`, `.badge-warning`, `.badge-error`, `.badge-info` |
| **Chips** | `.chip`, `.chip-amber`, `.chip-green`, `.chip-red`, `.chip-cyan`, `.chip-orange`, `.chip-purple`, `.chip-neutral` |
| **Layout** | `.row`, `.row-between`, `.col`, `.section`, `.section-title` |
| **Text** | `.text-muted`, `.truncate-text`, `.empty-message` |
| **Icons** | `.icon-xs`, `.icon-sm`, `.icon-md`, `.icon-lg` |
| **Tree** | `.tree-row`, `.tree-node`, `.tree-node-row` |
| **Keyboard** | `.kbd`, `.kbd-sm` |
| **Menus** | `.context-menu`, `.menu-item`, `.popover`, `.tooltip` |

### Usage Priority

1. **First choice:** Use CSS classes from `index.css` (e.g., `.btn-primary`)
2. **Second choice:** Use semantic Tailwind classes (e.g., `bg-bg-panel`)
3. **Last resort:** Inline Tailwind utilities for one-off styling

```css
@layer components {
  .btn-primary { /* Primary button styles */ }
  .btn-secondary { /* Secondary button styles */ }
  .input-field { /* Input field styles */ }
  .panel { /* Panel container styles */ }
}
```

## Animations

Predefined animations in `tailwind.config.js`:

```tsx
<div className="animate-fade-in" />      // Fade in
<div className="animate-slide-up" />     // Slide up + fade
<div className="animate-slide-in" />     // Slide in from right
<div className="animate-pulse-slow" />   // Slow pulse
<div className="animate-indeterminate" /> // Progress bar
```

## Z-Index Scale

```tsx
className="z-dropdown"       // Dropdowns
className="z-sticky"         // Sticky elements
className="z-modal-backdrop" // Modal backdrop
className="z-modal"          // Modal content
className="z-tooltip"        // Tooltips
className="z-notification"   // Toast notifications
```

## Typography

```tsx
// Font families
className="font-sans"  // Inter
className="font-mono"  // JetBrains Mono

// Font sizes
className="text-2xs"   // 10px
className="text-xs"    // 11px
className="text-sm"    // 12px
className="text-base"  // 13px
className="text-lg"    // 15px
```

## Dark Theme

The app uses a dark theme by default. All colors in `variables.css` are dark-mode first.

For light theme support (if needed):

```css
[data-theme="light"] {
  --color-bg: #ffffff;
  --color-txt: #18181b;
  /* ... */
}
```

## Adding New Tokens

1. Add CSS variable to `src/styles/variables.css`
2. Reference in `tailwind.config.js` theme extension
3. Use the Tailwind class in components

**Example:**

```css
/* variables.css */
--color-highlight: #fbbf24;
```

```javascript
// tailwind.config.js
colors: {
  highlight: 'var(--color-highlight)',
}
```

```tsx
// Component
<span className="bg-highlight">Highlighted</span>
```

---

## Anti-Patterns to Avoid

### ❌ Hardcoded Colors

```tsx
// Bad
<div className="bg-zinc-800 text-gray-300" />

// Good  
<div className="bg-bg-panel text-txt-secondary" />
```

### ❌ Inconsistent Border Radius

```tsx
// Bad - mixing radius sizes
<button className="rounded-md" />  // One button
<button className="rounded-xl" />  // Another button

// Good - consistent pattern
<button className="btn btn-primary" />  // Uses rounded-lg
```

### ❌ Ad-hoc Padding

```tsx
// Bad - arbitrary padding
<div className="p-7 px-9" />

// Good - use standard values
<div className="p-5" />  // Modal body
<div className="p-4" />  // Cards
<div className="px-3 py-2" />  // Panels
```

### ❌ Inline Button Styles

```tsx
// Bad - inline everything
<button className="px-4 py-2 bg-cyan-500 hover:bg-cyan-600 text-white rounded-lg transition-colors">
  Save
</button>

// Good - use component class
<button className="btn btn-primary">Save</button>
```

---

## Migration Checklist

When standardizing existing components:

- [ ] Replace hardcoded colors with semantic tokens
- [ ] Use CSS component classes where available
- [ ] Ensure consistent border-radius per component type
- [ ] Standardize padding using the spacing scale
- [ ] Use semantic gap values (gap-2, gap-3, gap-4)
- [ ] Replace inline button styles with `.btn-*` classes
- [ ] Replace inline input styles with `.input` classes
- [ ] Use `.modal-*` classes for modal structure

---

Last updated: January 2026
