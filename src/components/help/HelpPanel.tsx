// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * HelpPanel — Comprehensive in-app help and documentation.
 *
 * Opens as a center-pane tab. Covers all major features, supported formats,
 * keyboard shortcuts, and forensic workflows.
 *
 * Section content is split into individual files under `./sections/`.
 * The sidebar navigation is in `./HelpSidebar.tsx`.
 */

import type { Component } from "solid-js";
import { createSignal, createMemo, For, Show } from "solid-js";
import { HELP_SECTIONS } from "./sections";
import { HelpSidebar } from "./HelpSidebar";

export const HelpPanel: Component = () => {
  const [activeSection, setActiveSection] = createSignal("getting-started");
  const [searchQuery, setSearchQuery] = createSignal("");
  const [tocSide, setTocSide] = createSignal<"left" | "right">("left");

  const filteredSections = createMemo(() => {
    const query = searchQuery().toLowerCase().trim();
    if (!query) return HELP_SECTIONS;
    return HELP_SECTIONS.filter((s) => s.title.toLowerCase().includes(query));
  });

  const activeSectionIndex = createMemo(() =>
    HELP_SECTIONS.findIndex((s) => s.id === activeSection())
  );

  const goToPrev = () => {
    const idx = activeSectionIndex();
    if (idx > 0) setActiveSection(HELP_SECTIONS[idx - 1].id);
  };

  const goToNext = () => {
    const idx = activeSectionIndex();
    if (idx < HELP_SECTIONS.length - 1) setActiveSection(HELP_SECTIONS[idx + 1].id);
  };

  const sidebar = () => (
    <HelpSidebar
      sections={HELP_SECTIONS}
      filteredSections={filteredSections}
      activeSection={activeSection}
      searchQuery={searchQuery}
      tocSide={tocSide}
      activeSectionIndex={activeSectionIndex}
      onToggleSection={(id) => setActiveSection(id)}
      onSearchChange={setSearchQuery}
      onGoToPrev={goToPrev}
      onGoToNext={goToNext}
      onToggleTocSide={() => setTocSide((p) => (p === "left" ? "right" : "left"))}
    />
  );

  return (
    <div class="flex h-full overflow-hidden bg-bg">
      <Show when={tocSide() === "left"}>{sidebar()}</Show>

      {/* Content Area */}
      <div class="flex-1 overflow-y-auto">
        <div class="max-w-3xl mx-auto p-6">
          <For each={HELP_SECTIONS}>
            {(section) => (
              <Show when={activeSection() === section.id}>
                <div class="animate-fade-in">
                  <div class="flex items-center gap-3 mb-5">
                    <div class="p-2 bg-accent/10 rounded-xl text-accent">
                      <section.icon class="w-6 h-6" />
                    </div>
                    <div>
                      <h2 class="text-xl font-bold text-txt">{section.title}</h2>
                    </div>
                  </div>
                  <section.content />
                </div>
              </Show>
            )}
          </For>
        </div>
      </div>

      <Show when={tocSide() === "right"}>{sidebar()}</Show>
    </div>
  );
};
