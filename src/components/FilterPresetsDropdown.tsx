// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * FilterPresetsDropdown - Quick filter preset selection for evidence tree
 *
 * Features:
 * - Dropdown to select predefined filter presets
 * - Quick filtering by common file types (documents, images, databases, etc.)
 * - Shows active filter indicator
 * - Integrates with workspace profiles
 */

import { Component, For, Show, createSignal, createEffect } from "solid-js";
import type { FilterPreset } from "../hooks/useWorkspaceProfiles";
import {
  HiOutlineFunnel,
  HiOutlineChevronDown,
  HiOutlineCheck,
  HiOutlineXMark,
  HiOutlineDocument,
  HiOutlinePhoto,
  HiOutlineFilm,
  HiOutlineMusicalNote,
  HiOutlineArchiveBox,
  HiOutlineCodeBracket,
  HiOutlineCircleStack,
  HiOutlineEnvelope,
} from "./icons";

// =============================================================================
// Types
// =============================================================================

export interface FilterPresetsDropdownProps {
  /** Available filter presets from profile */
  presets?: FilterPreset[];
  /** Currently active preset ID */
  activePresetId?: string | null;
  /** Handler when preset is selected */
  onSelectPreset?: (preset: FilterPreset | null) => void;
  /** Custom quick filters (in addition to presets) */
  quickFilters?: QuickFilter[];
  /** Show quick filters section */
  showQuickFilters?: boolean;
  /** Compact mode (icon only) */
  compact?: boolean;
  /** Disabled state */
  disabled?: boolean;
}

export interface QuickFilter {
  id: string;
  name: string;
  extensions: string[];
  icon?: string;
}

// =============================================================================
// Default Quick Filters
// =============================================================================

const DEFAULT_QUICK_FILTERS: QuickFilter[] = [
  { 
    id: "documents", 
    name: "Documents", 
    extensions: [".pdf", ".doc", ".docx", ".xls", ".xlsx", ".ppt", ".pptx", ".txt", ".rtf", ".odt"],
    icon: "document" 
  },
  { 
    id: "images", 
    name: "Images", 
    extensions: [".jpg", ".jpeg", ".png", ".gif", ".bmp", ".tiff", ".webp", ".svg", ".heic", ".raw"],
    icon: "photo" 
  },
  { 
    id: "videos", 
    name: "Videos", 
    extensions: [".mp4", ".avi", ".mkv", ".mov", ".wmv", ".flv", ".webm", ".m4v"],
    icon: "film" 
  },
  { 
    id: "audio", 
    name: "Audio", 
    extensions: [".mp3", ".wav", ".aac", ".flac", ".ogg", ".wma", ".m4a"],
    icon: "music" 
  },
  { 
    id: "archives", 
    name: "Archives", 
    extensions: [".zip", ".rar", ".7z", ".tar", ".gz", ".bz2", ".ad1", ".e01", ".l01"],
    icon: "archive" 
  },
  { 
    id: "code", 
    name: "Code", 
    extensions: [".js", ".ts", ".py", ".java", ".c", ".cpp", ".h", ".cs", ".go", ".rs", ".html", ".css"],
    icon: "code" 
  },
  { 
    id: "databases", 
    name: "Databases", 
    extensions: [".db", ".sqlite", ".sqlite3", ".mdb", ".sql"],
    icon: "database" 
  },
  { 
    id: "emails", 
    name: "Emails", 
    extensions: [".eml", ".msg", ".pst", ".ost", ".mbox"],
    icon: "email" 
  },
];

// =============================================================================
// Helper Functions
// =============================================================================

const getQuickFilterIcon = (iconName?: string) => {
  switch (iconName) {
    case "document": return HiOutlineDocument;
    case "photo": return HiOutlinePhoto;
    case "film": return HiOutlineFilm;
    case "music": return HiOutlineMusicalNote;
    case "archive": return HiOutlineArchiveBox;
    case "code": return HiOutlineCodeBracket;
    case "database": return HiOutlineCircleStack;
    case "email": return HiOutlineEnvelope;
    default: return HiOutlineFunnel;
  }
};

// =============================================================================
// FilterPresetsDropdown Component
// =============================================================================

export const FilterPresetsDropdown: Component<FilterPresetsDropdownProps> = (props) => {
  const [isOpen, setIsOpen] = createSignal(false);
  const [activeQuickFilter, setActiveQuickFilter] = createSignal<string | null>(null);

  // Combine default quick filters with custom ones
  const quickFilters = () => {
    const defaults = props.showQuickFilters !== false ? DEFAULT_QUICK_FILTERS : [];
    return [...defaults, ...(props.quickFilters || [])];
  };

  // Check if any filter is active
  const hasActiveFilter = () => {
    return props.activePresetId !== null && props.activePresetId !== undefined 
      || activeQuickFilter() !== null;
  };

  // Handle preset selection
  const handlePresetSelect = (preset: FilterPreset | null) => {
    setActiveQuickFilter(null);
    props.onSelectPreset?.(preset);
    setIsOpen(false);
  };

  // Handle quick filter selection
  const handleQuickFilterSelect = (filter: QuickFilter) => {
    // Convert quick filter to a pseudo-preset
    const preset: FilterPreset = {
      id: `quick-${filter.id}`,
      name: filter.name,
      description: `Quick filter for ${filter.name.toLowerCase()}`,
      file_types: [],
      extensions: filter.extensions,
      size_range: null,
      date_range: null,
      search_terms: [],
      include_hidden: false,
      include_system: false,
    };
    
    if (activeQuickFilter() === filter.id) {
      // Toggle off
      setActiveQuickFilter(null);
      props.onSelectPreset?.(null);
    } else {
      setActiveQuickFilter(filter.id);
      props.onSelectPreset?.(preset);
    }
    setIsOpen(false);
  };

  // Clear all filters
  const handleClearFilters = () => {
    setActiveQuickFilter(null);
    props.onSelectPreset?.(null);
    setIsOpen(false);
  };

  // Close dropdown when clicking outside
  let dropdownRef: HTMLDivElement | undefined;
  createEffect(() => {
    if (isOpen()) {
      const handleClickOutside = (e: MouseEvent) => {
        if (dropdownRef && !dropdownRef.contains(e.target as Node)) {
          setIsOpen(false);
        }
      };
      document.addEventListener("mousedown", handleClickOutside);
      return () => document.removeEventListener("mousedown", handleClickOutside);
    }
  });

  // Find active preset name
  const activePresetName = () => {
    if (activeQuickFilter()) {
      const filter = quickFilters().find(f => f.id === activeQuickFilter());
      return filter?.name;
    }
    if (props.activePresetId) {
      const preset = props.presets?.find(p => p.id === props.activePresetId);
      return preset?.name;
    }
    return null;
  };

  return (
    <div class="relative" ref={dropdownRef}>
      {/* Trigger Button */}
      <button
        onClick={() => !props.disabled && setIsOpen(!isOpen())}
        disabled={props.disabled}
        class={`flex items-center gap-1.5 px-2 py-1.5 rounded-md border transition-colors ${
          hasActiveFilter()
            ? "bg-accent/10 border-accent/30 text-accent"
            : "bg-bg-secondary border-border text-txt-secondary hover:text-txt hover:border-border-strong"
        } ${props.disabled ? "opacity-50 cursor-not-allowed" : "cursor-pointer"}`}
        title={hasActiveFilter() ? `Filter: ${activePresetName()}` : "Filter presets"}
      >
        <HiOutlineFunnel class="w-4 h-4" />
        <Show when={!props.compact}>
          <span class="text-sm">
            {hasActiveFilter() ? activePresetName() : "Filter"}
          </span>
        </Show>
        <HiOutlineChevronDown
          class={`w-3.5 h-3.5 transition-transform ${isOpen() ? "rotate-180" : ""}`}
        />
        <Show when={hasActiveFilter()}>
          <button
            onClick={(e) => {
              e.stopPropagation();
              handleClearFilters();
            }}
            class="ml-0.5 p-0.5 hover:bg-accent/20 rounded"
            title="Clear filter"
          >
            <HiOutlineXMark class="w-3 h-3" />
          </button>
        </Show>
      </button>

      {/* Dropdown Menu */}
      <Show when={isOpen()}>
        <div class="absolute top-full left-0 mt-1 w-64 bg-bg-panel border border-border rounded-lg shadow-lg z-dropdown overflow-hidden animate-fade-in">
          {/* Quick Filters Section */}
          <Show when={quickFilters().length > 0}>
            <div class="p-2 border-b border-border/50">
              <div class="text-xs font-medium text-txt-muted uppercase mb-2 px-1">
                Quick Filters
              </div>
              <div class="grid grid-cols-2 gap-1">
                <For each={quickFilters()}>
                  {(filter) => {
                    const Icon = getQuickFilterIcon(filter.icon);
                    const isActive = activeQuickFilter() === filter.id;
                    return (
                      <button
                        onClick={() => handleQuickFilterSelect(filter)}
                        class={`flex items-center gap-2 px-2 py-1.5 rounded-md text-left transition-colors ${
                          isActive
                            ? "bg-accent/15 text-accent"
                            : "hover:bg-bg-hover text-txt-secondary"
                        }`}
                      >
                        <Icon class="w-4 h-4 flex-shrink-0" />
                        <span class="text-xs truncate">{filter.name}</span>
                        <Show when={isActive}>
                          <HiOutlineCheck class="w-3 h-3 ml-auto flex-shrink-0" />
                        </Show>
                      </button>
                    );
                  }}
                </For>
              </div>
            </div>
          </Show>

          {/* Saved Presets Section */}
          <Show when={props.presets && props.presets.length > 0}>
            <div class="p-2">
              <div class="text-xs font-medium text-txt-muted uppercase mb-2 px-1">
                Saved Presets
              </div>
              <div class="space-y-1">
                <For each={props.presets}>
                  {(preset) => {
                    const isActive = props.activePresetId === preset.id;
                    return (
                      <button
                        onClick={() => handlePresetSelect(isActive ? null : preset)}
                        class={`flex items-center gap-2 w-full px-2 py-2 rounded-md text-left transition-colors ${
                          isActive
                            ? "bg-accent/15 text-accent"
                            : "hover:bg-bg-hover text-txt"
                        }`}
                      >
                        <HiOutlineFunnel class="w-4 h-4 flex-shrink-0 text-txt-muted" />
                        <div class="flex-1 min-w-0">
                          <div class="text-sm truncate">{preset.name}</div>
                          <Show when={preset.description}>
                            <div class="text-xs text-txt-muted truncate">
                              {preset.description}
                            </div>
                          </Show>
                        </div>
                        <Show when={isActive}>
                          <HiOutlineCheck class="w-4 h-4 flex-shrink-0" />
                        </Show>
                      </button>
                    );
                  }}
                </For>
              </div>
            </div>
          </Show>

          {/* No presets message */}
          <Show when={(!props.presets || props.presets.length === 0) && quickFilters().length === 0}>
            <div class="p-4 text-center text-txt-muted text-sm">
              No filter presets available
            </div>
          </Show>

          {/* Clear Filter */}
          <Show when={hasActiveFilter()}>
            <div class="p-2 border-t border-border/50">
              <button
                onClick={handleClearFilters}
                class="flex items-center gap-2 w-full px-2 py-1.5 rounded-md text-left hover:bg-bg-hover text-txt-secondary transition-colors"
              >
                <HiOutlineXMark class="w-4 h-4" />
                <span class="text-sm">Clear filter</span>
              </button>
            </div>
          </Show>
        </div>
      </Show>
    </div>
  );
};

export default FilterPresetsDropdown;
