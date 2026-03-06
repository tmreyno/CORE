// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show } from "solid-js";
import { PrimaryTabBar } from "./PrimaryTabBar";
import { SecondaryTabBar } from "./SecondaryTabBar";
import { EmptyStateNoProject } from "./EmptyStateNoProject";
import { EmptyStateProjectActive } from "./EmptyStateProjectActive";
import { useCenterPaneLogic } from "./useCenterPaneLogic";
import type { CenterPaneProps } from "./types";

export const CenterPaneComponent: Component<CenterPaneProps> = (props) => {
  const logic = useCenterPaneLogic(props);

  return (
    <div class="flex flex-col h-full overflow-hidden bg-bg">
      {/* Primary Tab Bar - Container-level tabs */}
      <PrimaryTabBar
        containerTabs={logic.containerTabs}
        activeTabId={props.activeTabId}
        activeContainerTabId={() => logic.activeContainerTab()?.id}
        hasMultipleTabs={logic.hasMultipleTabs}
        tabCount={logic.tabCount}
        onSelect={logic.handleTabSelect}
        onClose={logic.handleTabClose}
      />
      
      {/* Secondary Tab Bar - Entry tabs from current container */}
      <SecondaryTabBar
        entries={logic.entriesForActiveContainer}
        activeTabId={props.activeTabId}
        activeContainerTab={logic.activeContainerTab}
        isViewingEntry={logic.isViewingEntry}
        onSelect={logic.handleTabSelect}
        onClose={logic.handleTabClose}
      />

      {/* Content area */}
      <div class="flex-1 overflow-hidden">
        <Show when={props.tabs().length > 0} fallback={
          <div class="flex items-center justify-center h-full text-txt-muted text-sm">
            <Show when={props.projectName?.() || props.projectRoot?.()} fallback={
              <EmptyStateNoProject
                onOpenProject={props.onOpenProject}
                onNewProject={props.onNewProject}
              />
            }>
              <EmptyStateProjectActive
                projectName={props.projectName}
                projectRoot={props.projectRoot}
                evidenceCount={props.evidenceCount}
              />
            </Show>
          </div>
        }>
          {props.children}
        </Show>
      </div>
    </div>
  );
};
