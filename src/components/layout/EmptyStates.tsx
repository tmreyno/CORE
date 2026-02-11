// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * EmptyStates Component
 * 
 * Displays appropriate empty state for CenterPane:
 * - No Project: Welcome screen with New/Open buttons and recent projects
 * - Project Ready: Success message with project info
 */

import { Component, Show } from "solid-js";
import {
  HiOutlineFolderOpen,
  HiOutlineCheckCircle,
} from "../icons";
import { Shortcut, CommonShortcuts } from "../ui/Kbd";
import { RecentProjectsList } from "../RecentProjectsList";

interface EmptyStatesProps {
  hasProject: boolean;
  projectInfo?: {
    name: string;
    projectPath: string;
    evidenceCount?: number;
  };
  onNewProject?: () => void;
  onOpenProject?: () => void;
  onOpenFile?: () => void;
  onQuickAction?: () => void;
}

export const EmptyStates: Component<EmptyStatesProps> = (props) => {
  return (
    <Show
      when={props.hasProject && props.projectInfo}
      fallback={
        // No Project State
        <div class="flex flex-col items-center justify-center h-full gap-4 p-8 text-center">
          <HiOutlineFolderOpen class="w-20 h-20 text-txt-muted/40" />
          <div>
            <h3 class="text-xl font-semibold text-txt mb-1">No file selected</h3>
            <p class="text-txt-muted text-sm">Create or open a project to begin</p>
          </div>

          <div class="flex gap-2 mt-2">
            <Show when={props.onNewProject}>
              <button class="btn btn-primary" onClick={props.onNewProject}>
                New Project
              </button>
            </Show>
            <Show when={props.onOpenProject}>
              <button class="btn btn-secondary" onClick={props.onOpenProject}>
                Open Project
              </button>
            </Show>
          </div>

          <Show when={props.onOpenProject}>
            <div class="mt-4 w-full max-w-md">
              <h4 class="text-sm font-medium text-txt-secondary mb-2">Recent Projects</h4>
              <RecentProjectsList 
                onOpenProject={props.onOpenProject!}
                maxItems={4}
              />
            </div>
          </Show>

          <div class="mt-6 p-4 bg-bg-secondary rounded-lg border border-border">
            <h4 class="text-sm font-medium text-txt mb-2">Keyboard Shortcuts</h4>
            <div class="space-y-1 text-sm">
              <div class="flex items-center justify-between gap-4">
                <span class="text-txt-muted">New Project</span>
                <Shortcut {...CommonShortcuts.newProject} />
              </div>
              <div class="flex items-center justify-between gap-4">
                <span class="text-txt-muted">Open Project</span>
                <Shortcut {...CommonShortcuts.open} />
              </div>
              <div class="flex items-center justify-between gap-4">
                <span class="text-txt-muted">Quick Actions</span>
                <Shortcut {...CommonShortcuts.commandPalette} />
              </div>
            </div>
          </div>
        </div>
      }
    >
      {/* Project Ready State */}
      <div class="flex flex-col items-center justify-center h-full gap-4 p-8 text-center">
        <HiOutlineCheckCircle class="w-20 h-20 text-success" />
        <div>
          <h3 class="text-xl font-semibold text-txt mb-1">
            {props.projectInfo!.name}
          </h3>
          <p class="text-txt-muted text-sm font-mono">
            {props.projectInfo!.projectPath}
          </p>
          <Show when={props.projectInfo!.evidenceCount !== undefined}>
            <p class="text-txt-secondary text-sm mt-2">
              {props.projectInfo!.evidenceCount} evidence item
              {props.projectInfo!.evidenceCount !== 1 ? "s" : ""} in project
            </p>
          </Show>
        </div>

        <div class="mt-2 p-4 bg-bg-secondary rounded-lg border border-border">
          <p class="text-sm text-txt-muted mb-2">
            Select a file from the sidebar to begin
          </p>
          <p class="text-xs text-txt-muted/70">
            or use the toolbar to add evidence containers
          </p>
        </div>

        <div class="mt-6 p-4 bg-bg-secondary rounded-lg border border-border">
          <h4 class="text-sm font-medium text-txt mb-2">Keyboard Shortcuts</h4>
          <div class="space-y-1 text-sm">
            <Show when={props.onOpenFile}>
              <div class="flex items-center justify-between gap-4">
                <span class="text-txt-muted">Open File</span>
                <Shortcut {...CommonShortcuts.open} />
              </div>
            </Show>
            <Show when={props.onQuickAction}>
              <div class="flex items-center justify-between gap-4">
                <span class="text-txt-muted">Quick Actions</span>
                <Shortcut {...CommonShortcuts.commandPalette} />
              </div>
            </Show>
          </div>
        </div>
      </div>
    </Show>
  );
};
