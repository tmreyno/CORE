# UI Constants Reference Guide

This document provides a quick reference for all available UI constants in CORE-FFX.

## Import Usage

```typescript
// Import specific constants
import { BG_DARK, ROUNDED_LG, TEXT_MUTED } from '@/components/ui/constants';

// Import from tree constants
import { TREE_ROW_HEIGHT, getFileIconColor } from '@/components/tree/constants';

// Or import everything from the central index
import { BG_DARK, TREE_ROW_HEIGHT } from '@/components/constants';
```

## Available Constants

### Font Sizes
| Constant | Value | Description |
|----------|-------|-------------|
| `UI_FONT_COMPACT` | `text-[12px]` | Compact font (12px) |
| `UI_FONT_SMALL` | `text-xs` | Small font (12px Tailwind) |
| `UI_FONT_BASE` | `text-sm` | Base font (14px) |

### Icon Sizes
| Constant | Value | Description |
|----------|-------|-------------|
| `UI_ICON_COMPACT` | `w-3.5 h-3.5` | Compact icons |
| `UI_ICON_SMALL` | `w-4 h-4` | Small icons |
| `UI_ICON_BASE` | `w-5 h-5` | Base icons |
| `TREE_ICON_EXPAND` | `w-3.5 h-3.5` | Tree expand chevron |
| `TREE_ICON_ITEM` | `w-3 h-3` | Tree file/folder icon |
| `TREE_ICON_CONTAINER` | `w-3 h-3` | Container header icon |

### Gaps
| Constant | Value | Description |
|----------|-------|-------------|
| `UI_GAP_COMPACT` | `gap-0.5` | Compact gap |
| `UI_GAP_SMALL` | `gap-1` | Small gap |
| `UI_GAP_BASE` | `gap-1.5` | Base gap |

### Padding
| Constant | Value | Description |
|----------|-------|-------------|
| `UI_PADDING_COMPACT` | `px-1 py-px` | Compact padding |
| `UI_PADDING_SMALL` | `px-1.5 py-0.5` | Small padding |
| `UI_PADDING_BASE` | `px-2 py-1` | Base padding |

### Background Colors
| Constant | Value | Description |
|----------|-------|-------------|
| `BG_PRIMARY` | `bg-bg` | Primary background (CSS var) |
| `BG_SECONDARY` | `bg-bg-secondary` | Secondary background |
| `BG_PANEL` | `bg-bg-panel` | Panel background |
| `BG_HOVER` | `bg-bg-hover` | Hover state |
| `BG_ACTIVE` | `bg-bg-active` | Active/pressed state |
| `BG_DARK` | `bg-zinc-900` | Dark background |
| `BG_ELEVATED` | `bg-zinc-800` | Elevated background |
| `BG_SUBTLE` | `bg-zinc-800/50` | Subtle background |
| `BG_MUTED` | `bg-zinc-700` | Muted background |

### Border Colors
| Constant | Value | Description |
|----------|-------|-------------|
| `BORDER_DEFAULT` | `border-border` | Default border (CSS var) |
| `BORDER_SUBTLE` | `border-zinc-700` | Subtle border |
| `BORDER_STRONG` | `border-zinc-600` | Strong border |
| `BORDER_ACCENT` | `border-accent` | Accent border |
| `BORDER_SUCCESS` | `border-green-500/50` | Success border |
| `BORDER_WARNING` | `border-yellow-500/50` | Warning border |
| `BORDER_ERROR` | `border-red-500/50` | Error border |

### Border Radius
| Constant | Value | Description |
|----------|-------|-------------|
| `ROUNDED_NONE` | `rounded-none` | No radius |
| `ROUNDED_SM` | `rounded` | Small radius |
| `ROUNDED_MD` | `rounded-md` | Medium radius |
| `ROUNDED_LG` | `rounded-lg` | Large radius |
| `ROUNDED_XL` | `rounded-xl` | Extra large radius |
| `ROUNDED_2XL` | `rounded-2xl` | 2XL radius |
| `ROUNDED_FULL` | `rounded-full` | Full radius (pill) |

### Shadows
| Constant | Value | Description |
|----------|-------|-------------|
| `SHADOW_SM` | `shadow-sm` | Small shadow |
| `SHADOW` | `shadow` | Default shadow |
| `SHADOW_LG` | `shadow-lg` | Large shadow |
| `SHADOW_XL` | `shadow-xl` | Extra large shadow |
| `SHADOW_2XL` | `shadow-2xl` | 2XL shadow |

### Text Colors
| Constant | Value | Description |
|----------|-------|-------------|
| `TEXT_PRIMARY` | `text-txt` | Primary text |
| `TEXT_SECONDARY` | `text-txt-secondary` | Secondary text |
| `TEXT_MUTED` | `text-txt-muted` | Muted text |
| `TEXT_ACCENT` | `text-accent` | Accent text |
| `TEXT_SUCCESS` | `text-green-400` | Success text |
| `TEXT_WARNING` | `text-yellow-400` | Warning text |
| `TEXT_ERROR` | `text-red-400` | Error text |

### Hover States
| Constant | Value | Description |
|----------|-------|-------------|
| `HOVER_BG` | `hover:bg-zinc-800` | Standard hover |
| `HOVER_BG_SUBTLE` | `hover:bg-zinc-800/50` | Subtle hover |
| `HOVER_BG_STRONG` | `hover:bg-zinc-700` | Strong hover |
| `HOVER_ACCENT` | `hover:bg-accent/10` | Accent hover |
| `HOVER_TEXT` | `hover:text-zinc-200` | Text hover |
| `ICON_BUTTON_HOVER` | Combined | Icon button hover |

## Composite Patterns

### Modal/Dialog
```typescript
// Use for modal containers
class={`${MODAL_CONTENT} flex flex-col`}

// Modal sections
<div class={MODAL_HEADER}>...</div>
<div class={MODAL_BODY}>...</div>
<div class={MODAL_FOOTER}>...</div>
```

### Panel
```typescript
// Full panel layout
class={PANEL_BASE}

// Panel sections
<div class={PANEL_HEADER}>...</div>
<div class={PANEL_CONTENT}>...</div>
<div class={PANEL_FOOTER}>...</div>
```

### Cards
```typescript
// Static card
class={CARD_BASE}

// Interactive card
class={CARD_HOVER}

// Selected card
class={CARD_SELECTED}
```

### Inputs
```typescript
// Standard input
<input class={INPUT_BASE} />

// Small input
<input class={INPUT_SMALL} />
```

### Popover/Tooltip
```typescript
// Popover container
class={POPOVER_BASE}

// Tooltip
class={TOOLTIP_BASE}
```

## Badge Constants
| Constant | Usage |
|----------|-------|
| `BADGE_BASE` | Base badge styling |
| `BADGE_INFO` | Info badge (cyan) |
| `BADGE_SUCCESS` | Success badge (green) |
| `BADGE_WARNING` | Warning badge (yellow) |
| `BADGE_ERROR` | Error badge (red) |
| `BADGE_MUTED` | Muted badge |

## Button Constants
| Constant | Usage |
|----------|-------|
| `BUTTON_COMPACT` | Compact button base |
| `BUTTON_SMALL` | Small button base |
| `BUTTON_PRIMARY` | Primary button state |
| `BUTTON_SECONDARY` | Secondary button state |
| `BUTTON_GHOST` | Ghost button state |
| `BUTTON_DANGER` | Danger button state |

## Tree Constants (from tree/constants.ts)

### Layout
| Constant | Value | Description |
|----------|-------|-------------|
| `TREE_ROW_HEIGHT` | `18` | Standard row height (px) |
| `TREE_ROW_HEIGHT_COMPACT` | `16` | Compact row height (px) |
| `TREE_ROW_HEIGHT_COMFORTABLE` | `22` | Comfortable row height (px) |
| `TREE_INDENT_SIZE` | `10` | Indent per depth level (px) |

### Row Classes
| Constant | Usage |
|----------|-------|
| `TREE_ROW_BASE_CLASSES` | Base row styling |
| `TREE_ROW_SELECTED_CLASSES` | Selected row |
| `TREE_ROW_NORMAL_CLASSES` | Normal row |
| `TREE_ROW_ACTIVE_CLASSES` | Active/focused row |
| `TREE_ROW_DISABLED_CLASSES` | Disabled row |

### Helper Functions
```typescript
// Get container colors by type
const colors = getContainerColors('ad1');
// Returns: { icon: 'text-blue-400', badge: '...', border: '...' }

// Get file icon color by extension
const color = getFileIconColor('document.pdf');
// Returns: 'text-blue-400'

// Calculate tree indent
const padding = getTreeIndent(depth);
// Returns: '20px' for depth=1

// Get row classes by state
const classes = getTreeRowClasses({ isSelected: true });
```
