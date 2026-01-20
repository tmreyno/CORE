# CORE-FFX Styling Guide

CSS architecture using Tailwind CSS with CSS custom properties for theming.

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
│  (@tailwind directives)                                          │
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
├── index.css               # Base styles + Tailwind directives
├── App.css                 # App-specific styles + font imports
└── styles/
    └── variables.css       # CSS custom properties (design tokens)
```

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

## Custom Components

The `index.css` defines reusable component classes in the `@layer components`:

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

*Last updated: January 18, 2026*
