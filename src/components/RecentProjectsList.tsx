// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * RecentProjectsList - Displays recently opened projects
 * 
 * Shows a list of recently opened projects with the ability to:
 * - Click to open a project
 * - Remove items from the list
 * - Clear all recent projects
 */

import { Component, For, Show, createSignal, onMount } from "solid-js";
import {
  HiOutlineFolder,
  HiOutlineClock,
  HiOutlineXMark,
  HiOutlineTrash,
  HiOutlineFolderOpen,
} from "./icons";
import { 
  getRecentProjects, 
  removeRecentProject, 
  clearRecentProjects,
  type RecentProject 
} from "./preferences";

export interface RecentProjectsListProps {
  /** Called when user clicks to open a project */
  onOpenProject: (path: string) => void;
  /** Maximum items to show (default: 5) */
  maxItems?: number;
  /** Compact mode for sidebar display */
  compact?: boolean;
  /** Class to apply to container */
  class?: string;
}

/**
 * Format relative time (e.g., "2 hours ago", "Yesterday", "Dec 15")
 */
function formatRelativeTime(isoDate: string): string {
  const date = new Date(isoDate);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / (1000 * 60));
  const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
  const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

  if (diffMins < 1) return "Just now";
  if (diffMins < 60) return `${diffMins} min ago`;
  if (diffHours < 24) return `${diffHours} hour${diffHours > 1 ? 's' : ''} ago`;
  if (diffDays === 1) return "Yesterday";
  if (diffDays < 7) return `${diffDays} days ago`;
  
  // Format as date
  return date.toLocaleDateString(undefined, { month: 'short', day: 'numeric' });
}

/**
 * Get directory name from full path
 */
function getDirectoryName(path: string): string {
  const parts = path.split('/');
  // Return the parent folder name if exists
  if (parts.length > 1) {
    return parts[parts.length - 2] || parts[parts.length - 1];
  }
  return parts[parts.length - 1];
}

export const RecentProjectsList: Component<RecentProjectsListProps> = (props) => {
  const [projects, setProjects] = createSignal<RecentProject[]>([]);
  const [hoveredProject, setHoveredProject] = createSignal<string | null>(null);

  // Load projects on mount
  onMount(() => {
    refreshProjects();
  });

  const refreshProjects = () => {
    const recent = getRecentProjects();
    const maxItems = props.maxItems ?? 5;
    setProjects(recent.slice(0, maxItems));
  };

  const handleRemove = (path: string, e: MouseEvent) => {
    e.stopPropagation();
    removeRecentProject(path);
    refreshProjects();
  };

  const handleClearAll = () => {
    clearRecentProjects();
    setProjects([]);
  };

  const handleOpenProject = (path: string) => {
    props.onOpenProject(path);
  };

  return (
    <div class={`${props.class || ''}`}>
      <Show when={projects().length > 0} fallback={
        <Show when={!props.compact}>
          <div class="text-center py-6 text-txt-muted">
            <HiOutlineClock class="w-8 h-8 mx-auto mb-2 opacity-40" />
            <p class="text-sm">No recent projects</p>
            <p class="text-xs mt-1 opacity-70">Projects you open will appear here</p>
          </div>
        </Show>
      }>
        {/* Header */}
        <Show when={!props.compact}>
          <div class="flex items-center justify-between mb-3">
            <h3 class="text-sm font-medium text-txt flex items-center gap-2">
              <HiOutlineClock class="w-4 h-4 text-txt-muted" />
              Recent Projects
            </h3>
            <button
              class="text-xs text-txt-muted hover:text-txt transition-colors flex items-center gap-1"
              onClick={handleClearAll}
              title="Clear recent projects"
            >
              <HiOutlineTrash class="w-3 h-3" />
              Clear
            </button>
          </div>
        </Show>

        {/* Projects list */}
        <div class={`space-y-1 ${props.compact ? '' : 'space-y-2'}`}>
          <For each={projects()}>
            {(project) => (
              <div
                class={`group flex items-center gap-3 rounded-lg transition-all duration-150 cursor-pointer ${
                  props.compact 
                    ? 'px-2 py-1.5 hover:bg-bg-hover' 
                    : 'p-3 bg-bg-secondary/50 hover:bg-bg-hover border border-border/30 hover:border-border/60'
                }`}
                onClick={() => handleOpenProject(project.path)}
                onMouseEnter={() => setHoveredProject(project.path)}
                onMouseLeave={() => setHoveredProject(null)}
                title={project.path}
              >
                {/* Icon */}
                <div class={`shrink-0 ${props.compact ? '' : 'p-2 bg-accent/10 rounded-lg'}`}>
                  <Show when={hoveredProject() === project.path} fallback={
                    <HiOutlineFolder class={`${props.compact ? 'w-4 h-4' : 'w-5 h-5'} text-accent`} />
                  }>
                    <HiOutlineFolderOpen class={`${props.compact ? 'w-4 h-4' : 'w-5 h-5'} text-accent`} />
                  </Show>
                </div>

                {/* Project info */}
                <div class="flex-1 min-w-0">
                  <div class={`font-medium text-txt truncate ${props.compact ? 'text-xs' : 'text-sm'}`}>
                    {project.name}
                  </div>
                  <Show when={!props.compact}>
                    <div class="flex items-center gap-2 mt-0.5">
                      <span class="text-xs text-txt-muted truncate max-w-[180px]">
                        {getDirectoryName(project.path)}
                      </span>
                      <span class="text-xs text-txt-muted/60">•</span>
                      <span class="text-xs text-txt-muted/60">
                        {formatRelativeTime(project.lastOpened)}
                      </span>
                    </div>
                  </Show>
                </div>

                {/* Remove button */}
                <button
                  class={`shrink-0 p-1 rounded transition-all ${
                    props.compact 
                      ? 'opacity-0 group-hover:opacity-60 hover:!opacity-100 hover:bg-bg-active' 
                      : 'opacity-0 group-hover:opacity-60 hover:!opacity-100 hover:bg-error/10 hover:text-error'
                  }`}
                  onClick={(e) => handleRemove(project.path, e)}
                  title="Remove from recent"
                >
                  <HiOutlineXMark class="w-3.5 h-3.5" />
                </button>
              </div>
            )}
          </For>
        </div>

        {/* Show more link if there are more projects */}
        <Show when={getRecentProjects().length > (props.maxItems ?? 5)}>
          <div class="mt-3 text-center">
            <button 
              class="text-xs text-accent hover:text-accent-hover transition-colors"
              onClick={() => {
                // Could open a full recent projects modal
                console.log("Show all recent projects");
              }}
            >
              Show all ({getRecentProjects().length} projects)
            </button>
          </div>
        </Show>
      </Show>
    </div>
  );
};

export default RecentProjectsList;
