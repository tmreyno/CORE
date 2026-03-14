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

      {/* Panel Toggle Icons — three-section layout icon, left/right clickable */}
      <div class="ml-auto mr-2 flex items-center gap-0.5">
        <div class="flex items-center justify-center p-1.5 rounded-md text-txt-muted">
          <svg class="w-7 h-4" viewBox="0 0 30 20" fill="none" xmlns="http://www.w3.org/2000/svg">
            {/* Left sidebar — clickable */}
            <rect x="1" y="3" width="6" height="14" rx="1"
              fill={props.leftCollapsed() ? "none" : "currentColor"}
              stroke="currentColor" stroke-width="1.2"
              opacity={props.leftCollapsed() ? "0.4" : "1"}
              pointer-events="all"
              class="cursor-pointer transition-all duration-150"
              style={{ color: props.leftCollapsed() ? "var(--color-txt-muted)" : "var(--color-accent)" }}
              onClick={() => props.setLeftCollapsed((prev) => !prev)}
            >
              <title>{props.leftCollapsed() ? "Show Left Panel" : "Hide Left Panel"}</title>
            </rect>
            {/* Center panel — click toggles both sides */}
            <rect x="9" y="3" width="12" height="14"
              fill="currentColor"
              stroke="currentColor" stroke-width="1.2" opacity="0.5"
              pointer-events="all"
              class="cursor-pointer transition-all duration-150"
              onClick={() => {
                const bothVisible = !props.leftCollapsed() && !props.rightCollapsed();
                props.setLeftCollapsed(bothVisible);
                props.setRightCollapsed(bothVisible);
              }}
            >
              <title>{!props.leftCollapsed() && !props.rightCollapsed() ? "Hide Both Panels" : "Show Both Panels"}</title>
            </rect>
            {/* Right sidebar — clickable */}
            <rect x="23" y="3" width="6" height="14" rx="1"
              fill={props.rightCollapsed() ? "none" : "currentColor"}
              stroke="currentColor" stroke-width="1.2"
              opacity={props.rightCollapsed() ? "0.4" : "1"}
              pointer-events="all"
              class="cursor-pointer transition-all duration-150"
              style={{ color: props.rightCollapsed() ? "var(--color-txt-muted)" : "var(--color-accent)" }}
              onClick={() => props.setRightCollapsed((prev) => !prev)}
            >
              <title>{props.rightCollapsed() ? "Show Right Panel" : "Hide Right Panel"}</title>
            </rect>
          </svg>
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
