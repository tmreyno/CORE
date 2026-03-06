// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * HelpSidebar — Table-of-contents navigation for the Help panel.
 * Renders section list with search, prev/next navigation, and left/right toggle.
 */

import type { Component, Accessor } from "solid-js";
import { For, Show } from "solid-js";
import {
  HiOutlineQuestionMarkCircle,
  HiOutlineChevronUp,
  HiOutlineChevronDown,
  HiOutlineBars3BottomLeft,
  HiOutlineBars3BottomRight,
} from "../icons";
import type { HelpSection } from "./types";

interface HelpSidebarProps {
  sections: HelpSection[];
  filteredSections: Accessor<HelpSection[]>;
  activeSection: Accessor<string>;
  searchQuery: Accessor<string>;
  tocSide: Accessor<"left" | "right">;
  activeSectionIndex: Accessor<number>;
  onToggleSection: (id: string) => void;
  onSearchChange: (query: string) => void;
  onGoToPrev: () => void;
  onGoToNext: () => void;
  onToggleTocSide: () => void;
}

export const HelpSidebar: Component<HelpSidebarProps> = (props) => (
  <div
    class="w-56 flex-shrink-0 bg-bg-panel overflow-y-auto"
    classList={{
      "border-r border-border": props.tocSide() === "left",
      "border-l border-border": props.tocSide() === "right",
    }}
  >
    <div class="p-3">
      <div class="flex items-center gap-2 mb-3">
        <HiOutlineQuestionMarkCircle class="w-5 h-5 text-accent" />
        <h2 class="font-semibold text-txt text-sm flex-1">Help & Docs</h2>
        <button
          class="icon-btn-sm"
          onClick={props.onGoToPrev}
          disabled={props.activeSectionIndex() <= 0}
          title="Previous section"
        >
          <HiOutlineChevronUp class="w-4 h-4" />
        </button>
        <button
          class="icon-btn-sm"
          onClick={props.onGoToNext}
          disabled={props.activeSectionIndex() >= props.sections.length - 1}
          title="Next section"
        >
          <HiOutlineChevronDown class="w-4 h-4" />
        </button>
        <button
          class="icon-btn-sm"
          onClick={props.onToggleTocSide}
          title={`Move contents to ${props.tocSide() === "left" ? "right" : "left"}`}
        >
          <Show
            when={props.tocSide() === "left"}
            fallback={<HiOutlineBars3BottomLeft class="w-4 h-4" />}
          >
            <HiOutlineBars3BottomRight class="w-4 h-4" />
          </Show>
        </button>
      </div>

      {/* Search */}
      <input
        type="text"
        placeholder="Search topics…"
        class="input-sm w-full mb-3"
        value={props.searchQuery()}
        onInput={(e) => props.onSearchChange(e.currentTarget.value)}
      />

      {/* Section List */}
      <nav class="space-y-0.5">
        <For each={props.filteredSections()}>
          {(section) => (
            <button
              class="w-full flex items-center gap-2 px-2.5 py-2 rounded-lg text-sm transition-colors duration-150"
              classList={{
                "bg-accent/10 text-accent font-medium": props.activeSection() === section.id,
                "text-txt-secondary hover:bg-bg-hover hover:text-txt": props.activeSection() !== section.id,
              }}
              onClick={() => props.onToggleSection(section.id)}
            >
              <section.icon class="w-4 h-4 flex-shrink-0" />
              <span class="truncate text-left">{section.title}</span>
            </button>
          )}
        </For>
      </nav>
    </div>
  </div>
);
