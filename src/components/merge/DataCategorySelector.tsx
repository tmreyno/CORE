// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * DataCategorySelector — Category-level toggle panel for selecting which
 * data categories to include/exclude in a project merge.
 *
 * Shows a compact grid of checkboxes, one per merge data category.
 * All categories are enabled by default. Unchecking a category adds it
 * to the `skipCategories` list in MergeExclusions.
 */

import { Component, For, createMemo } from "solid-js";
import {
  HiOutlineArchiveBox,
  HiOutlineBookmark,
  HiOutlineClock,
  HiOutlineShieldCheck,
  HiOutlineArchiveBoxArrowDown,
  HiOutlineClipboardDocumentList,
  HiOutlineDocumentText,
  HiOutlineMagnifyingGlass,
  HiOutlineArrowUpTray,
  HiOutlineChartBar,
  HiOutlineTag,
} from "../icons";
import type { MergeDataCategory, ProjectMergeSummary } from "./types";
import { MERGE_CATEGORIES } from "../../api/projectMerge";

/** Icon mapping for categories */
const CATEGORY_ICONS: Record<MergeDataCategory, typeof HiOutlineArchiveBox> = {
  evidence: HiOutlineArchiveBox,
  bookmarks_notes: HiOutlineBookmark,
  activity: HiOutlineClock,
  coc: HiOutlineShieldCheck,
  collections: HiOutlineArchiveBoxArrowDown,
  forms: HiOutlineClipboardDocumentList,
  reports: HiOutlineClipboardDocumentList,
  tags: HiOutlineTag,
  searches: HiOutlineMagnifyingGlass,
  documents: HiOutlineDocumentText,
  exports: HiOutlineArrowUpTray,
  processed: HiOutlineChartBar,
};

export interface DataCategorySelectorProps {
  /** Set of currently skipped category IDs */
  skippedCategories: Set<string>;
  /** Toggle a category on/off */
  onToggleCategory: (categoryId: MergeDataCategory) => void;
  /** All project summaries (for computing counts) */
  summaries: ProjectMergeSummary[];
}

export const DataCategorySelector: Component<DataCategorySelectorProps> = (props) => {
  /** Compute total count for a category across all projects */
  const getCategoryCount = (cat: typeof MERGE_CATEGORIES[number]): number => {
    if (cat.statKeys.length === 0) {
      // Use array lengths for categories that map to detail arrays
      switch (cat.id) {
        case "coc": return props.summaries.reduce((a, s) => a + s.cocItems.length, 0);
        case "collections": return props.summaries.reduce((a, s) => a + s.collections.length, 0);
        case "forms": return props.summaries.reduce((a, s) => a + s.formSubmissions.length, 0);
        default: return 0;
      }
    }
    return cat.statKeys.reduce((total, key) => {
      return total + props.summaries.reduce((a, s) => a + ((s as unknown as Record<string, number>)[key as string] || 0), 0);
    }, 0);
  };

  const enabledCount = createMemo(() =>
    MERGE_CATEGORIES.filter((c) => !props.skippedCategories.has(c.id)).length,
  );

  return (
    <div class="p-3 rounded-lg bg-bg-panel border border-border">
      <div class="flex items-center justify-between mb-2">
        <h3 class="text-sm font-semibold text-txt">Data Categories</h3>
        <span class="text-xs text-txt-muted">
          {enabledCount()}/{MERGE_CATEGORIES.length} enabled
        </span>
      </div>
      <p class="text-xs text-txt-muted mb-2">
        Uncheck categories to exclude them entirely from the merge.
      </p>
      <div class="grid grid-cols-2 gap-x-4 gap-y-1">
        <For each={MERGE_CATEGORIES}>
          {(cat) => {
            const Icon = CATEGORY_ICONS[cat.id];
            const count = () => getCategoryCount(cat);
            const isEnabled = () => !props.skippedCategories.has(cat.id);
            return (
              <label
                class="flex items-center gap-2 py-1 px-1.5 rounded hover:bg-bg-hover cursor-pointer select-none"
                classList={{ "opacity-50": !isEnabled() }}
              >
                <input
                  type="checkbox"
                  checked={isEnabled()}
                  onChange={() => props.onToggleCategory(cat.id)}
                  class="accent-accent w-3.5 h-3.5"
                />
                <Icon class="w-3.5 h-3.5 text-txt-muted shrink-0" />
                <span class="text-xs text-txt truncate flex-1">{cat.label}</span>
                <span class="text-2xs text-txt-muted tabular-nums">{count()}</span>
              </label>
            );
          }}
        </For>
      </div>
    </div>
  );
};
