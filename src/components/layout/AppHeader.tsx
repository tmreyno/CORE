// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Application header / title bar.
 *
 * Contains: logo, project badge (name + modified indicator), panel layout
 * toggle SVG, and quick-actions toggle button.
 */

import { Show } from "solid-js";
import type { Accessor, Setter, Component } from "solid-js";
import { HiOutlineBolt } from "../icons";
import ffxLogo from "../../assets/branding/core-logo-48.png";
import { APP_SHORT } from "../../utils/edition";

export interface AppHeaderProps {
  /** Project name (undefined when no project is loaded) */
  projectName: Accessor<string | undefined>;
  /** Whether the project has unsaved changes */
  projectModified: Accessor<boolean>;
  /** Left panel collapsed state */
  leftCollapsed: Accessor<boolean>;
  setLeftCollapsed: Setter<boolean>;
  /** Right panel collapsed state */
  rightCollapsed: Accessor<boolean>;
  setRightCollapsed: Setter<boolean>;
  /** Quick Actions bar visibility */
  showQuickActions: Accessor<boolean>;
  setShowQuickActions: Setter<boolean>;
}

export const AppHeader: Component<AppHeaderProps> = (props) => {
  return (
    <header class="app-header">
      <div class="brand">
        <img src={ffxLogo} alt={`${APP_SHORT} Logo`} class="brand-logo" />
        <span class="text-xs font-bold tracking-wider text-txt-muted ml-1 select-none">{APP_SHORT}</span>
      </div>

      {/* Project Badge */}
      <Show when={props.projectName()}>
        <div
          class="flex items-center gap-1.5 px-2.5 py-1 text-sm font-medium text-accent bg-accent/10 rounded-md border border-accent/20 truncate max-w-[220px]"
          title={`Project: ${props.projectName()!}`}
        >
          <span class="truncate">{props.projectName()!}</span>
          <Show when={props.projectModified()}>
            <span class="w-1.5 h-1.5 rounded-full bg-warning shrink-0" title="Unsaved changes" />
          </Show>
        </div>
      </Show>

      {/* Panel Toggle — three-section layout icon, keyboard accessible */}
      <div class="ml-auto mr-2 flex items-center gap-0.5">
        <div role="toolbar" aria-label="Panel layout" class="flex items-center gap-[2px] p-1.5 rounded-md">
          {/* Left panel toggle */}
          <button
            class={`w-[6px] h-[14px] rounded-[1px] border transition-all duration-150 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent/50 ${
              props.leftCollapsed() ? "border-txt-muted/40 bg-transparent" : "border-accent bg-accent"
            }`}
            aria-label={props.leftCollapsed() ? "Show left panel" : "Hide left panel"}
            aria-pressed={!props.leftCollapsed()}
            onClick={() => props.setLeftCollapsed((prev) => !prev)}
            title={props.leftCollapsed() ? "Show Left Panel" : "Hide Left Panel"}
          />
          {/* Center — toggles both panels */}
          <button
            class="w-[12px] h-[14px] bg-txt-muted/50 border border-txt-muted/30 transition-all duration-150 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent/50"
            aria-label={!props.leftCollapsed() && !props.rightCollapsed() ? "Hide both panels" : "Show both panels"}
            onClick={() => {
              const bothVisible = !props.leftCollapsed() && !props.rightCollapsed();
              props.setLeftCollapsed(bothVisible);
              props.setRightCollapsed(bothVisible);
            }}
            title={!props.leftCollapsed() && !props.rightCollapsed() ? "Hide Both Panels" : "Show Both Panels"}
          />
          {/* Right panel toggle */}
          <button
            class={`w-[6px] h-[14px] rounded-[1px] border transition-all duration-150 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent/50 ${
              props.rightCollapsed() ? "border-txt-muted/40 bg-transparent" : "border-accent bg-accent"
            }`}
            aria-label={props.rightCollapsed() ? "Show right panel" : "Hide right panel"}
            aria-pressed={!props.rightCollapsed()}
            onClick={() => props.setRightCollapsed((prev) => !prev)}
            title={props.rightCollapsed() ? "Show Right Panel" : "Hide Right Panel"}
          />
        </div>
        <div class="w-px h-4 bg-border mx-1" />
        <button
          class={`flex items-center justify-center p-1.5 rounded-md transition-all duration-150 ${props.showQuickActions() ? 'bg-accent/20 text-accent' : 'text-txt-muted hover:text-txt hover:bg-bg-hover'}`}
          onClick={() => props.setShowQuickActions(!props.showQuickActions())}
          title={props.showQuickActions() ? "Hide Quick Actions" : "Show Quick Actions"}
          aria-label={props.showQuickActions() ? "Hide quick actions bar" : "Show quick actions bar"}
          aria-pressed={props.showQuickActions()}
        >
          <HiOutlineBolt class="w-4 h-4" />
        </button>
      </div>
    </header>
  );
};
