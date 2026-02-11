// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, For, type Accessor, type Setter } from "solid-js";
import {
  HiOutlineMagnifyingGlass,
  HiOutlineViewGrid,
  HiOutlineViewList,
  HiOutlineArrowUpTray,
  HiOutlineDocumentText,
} from "../../icons";
import type { TemplateCategory } from "../../../hooks/useProjectTemplates";

type ViewMode = "grid" | "list";

interface TemplateFiltersProps {
  searchQuery: Accessor<string>;
  setSearchQuery: Setter<string>;
  selectedCategory: Accessor<TemplateCategory | "all">;
  setSelectedCategory: Setter<TemplateCategory | "all">;
  viewMode: Accessor<ViewMode>;
  setViewMode: Setter<ViewMode>;
  categories: Array<{ value: TemplateCategory | "all"; label: string }>;
  onImport: () => void;
  onCreateFromProject: () => void;
}

export const TemplateFilters: Component<TemplateFiltersProps> = (props) => {
  return (
    <div class="flex items-center gap-3">
      {/* Search */}
      <div class="relative flex-1">
        <HiOutlineMagnifyingGlass class="absolute left-3 top-1/2 -translate-y-1/2 w-icon-sm h-icon-sm text-txt-muted" />
        <input
          type="text"
          placeholder="Search templates..."
          value={props.searchQuery()}
          onInput={(e) => props.setSearchQuery(e.currentTarget.value)}
          class="w-full pl-9 pr-3 py-2 bg-bg-secondary border border-border rounded-md text-sm text-txt placeholder-txt-muted focus:outline-none focus:ring-1 focus:ring-accent"
        />
      </div>

      {/* Category Filter */}
      <select
        value={props.selectedCategory()}
        onChange={(e) =>
          props.setSelectedCategory(
            e.currentTarget.value as TemplateCategory | "all"
          )
        }
        class="px-3 py-2 bg-bg-secondary border border-border rounded-md text-sm text-txt focus:outline-none focus:ring-1 focus:ring-accent"
      >
        <For each={props.categories}>
          {(cat) => <option value={cat.value}>{cat.label}</option>}
        </For>
      </select>

      {/* View Mode Toggle */}
      <div class="flex items-center gap-1 bg-bg-secondary border border-border rounded-md p-1">
        <button
          onClick={() => props.setViewMode("grid")}
          class={`p-1.5 rounded ${
            props.viewMode() === "grid"
              ? "bg-accent text-white"
              : "text-txt-secondary hover:text-txt hover:bg-bg-hover"
          }`}
        >
          <HiOutlineViewGrid class="w-icon-sm h-icon-sm" />
        </button>
        <button
          onClick={() => props.setViewMode("list")}
          class={`p-1.5 rounded ${
            props.viewMode() === "list"
              ? "bg-accent text-white"
              : "text-txt-secondary hover:text-txt hover:bg-bg-hover"
          }`}
        >
          <HiOutlineViewList class="w-icon-sm h-icon-sm" />
        </button>
      </div>

      {/* Action Buttons */}
      <button
        onClick={props.onImport}
        class="px-3 py-2 bg-bg-secondary hover:bg-bg-hover border border-border rounded-md text-sm text-txt flex items-center gap-2"
      >
        <HiOutlineArrowUpTray class="w-icon-sm h-icon-sm" />
        Import
      </button>
      <button
        onClick={props.onCreateFromProject}
        class="px-3 py-2 bg-accent hover:bg-accent-hover text-white rounded-md text-sm flex items-center gap-2"
      >
        <HiOutlineDocumentText class="w-icon-sm h-icon-sm" />
        Create from Project
      </button>
    </div>
  );
};
