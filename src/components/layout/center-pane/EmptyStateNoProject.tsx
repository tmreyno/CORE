// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show } from "solid-js";
import {
  HiOutlineDocumentText,
  HiOutlinePlusCircle,
  HiOutlineFolderOpen,
} from "../../icons";
import { Shortcut, CommonShortcuts } from "../../ui/Kbd";
import { RecentProjectsList } from "../../RecentProjectsList";

interface EmptyStateNoProjectProps {
  onOpenProject?: (path?: string) => void;
  onNewProject?: () => void;
}

/** Welcome screen shown when no project is loaded */
export function EmptyStateNoProject(props: EmptyStateNoProjectProps) {
  return (
    <div class="text-center p-8 max-w-lg">
      <div class="w-16 h-16 mx-auto mb-4 rounded-full bg-bg-secondary flex items-center justify-center">
        <HiOutlineDocumentText class="w-8 h-8 opacity-40" />
      </div>
      <h3 class="text-txt font-medium mb-2">No file selected</h3>
      <p class="text-txt-muted text-sm mb-6">
        Select an evidence container, case document, or processed database from the sidebar to view its contents here.
      </p>
      
      {/* Quick Actions */}
      <Show when={props.onNewProject || props.onOpenProject}>
        <div class="flex items-center justify-center gap-3 mb-6">
          <Show when={props.onNewProject}>
            <button onClick={props.onNewProject} class="btn btn-primary">
              <HiOutlinePlusCircle class="w-4 h-4" />
              New Project
            </button>
          </Show>
          <Show when={props.onOpenProject}>
            <button onClick={() => props.onOpenProject?.()} class="btn btn-secondary">
              <HiOutlineFolderOpen class="w-4 h-4" />
              Open Project
            </button>
          </Show>
        </div>
      </Show>
      
      {/* Recent Projects */}
      <Show when={props.onOpenProject}>
        <div class="mt-2 mb-6 text-left">
          <RecentProjectsList 
            onOpenProject={props.onOpenProject!} 
            maxItems={4}
          />
        </div>
      </Show>
      
      <div class="flex items-center justify-center gap-6 text-xs text-txt-muted">
        <span class="flex items-center gap-1.5">
          <Shortcut {...CommonShortcuts.newProject} />
          <span>New</span>
        </span>
        <span class="flex items-center gap-1.5">
          <Shortcut {...CommonShortcuts.open} />
          <span>Open</span>
        </span>
        <span class="flex items-center gap-1.5">
          <Shortcut {...CommonShortcuts.commandPalette} />
          <span>Commands</span>
        </span>
      </div>
    </div>
  );
}
