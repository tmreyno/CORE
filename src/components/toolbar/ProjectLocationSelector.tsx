// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ProjectLocationSelector - Dropdown selector for project locations
 * 
 * Features:
 * - Evidence path
 * - Processed database path
 * - Case documents path
 * - Empty state when no project open
 */

import { Component, For, Show } from "solid-js";
import { splitPath } from "../../utils/pathUtils";
import { HiOutlineFolderOpen, HiOutlineChevronDown } from "../icons";

export interface ProjectLocation {
  id: string;
  label: string;
  path: string | null;
  icon: "evidence" | "database" | "documents";
}

interface ProjectLocationSelectorProps {
  locations: ProjectLocation[];
  selectedPath: string;
  onPathSelect: (path: string, locationId?: string) => void;
  compact?: boolean;
}

// Get the folder name from a path
const getFolderName = (path: string | null): string => {
  if (!path) return "";
  const parts = splitPath(path).filter(Boolean);
  return parts[parts.length - 1] || path;
};

export const ProjectLocationSelector: Component<ProjectLocationSelectorProps> = (props) => {
  const compact = () => props.compact ?? false;
  const hasLocations = () => props.locations.length > 0;
  
  return (
    <Show 
      when={hasLocations()} 
      fallback={
        <div class={`flex items-center gap-2 px-3 py-1.5 text-sm text-txt-muted border border-border rounded-md bg-bg ${compact() ? 'w-[140px]' : 'w-[240px]'}`}>
          <HiOutlineFolderOpen class="w-4 h-4 shrink-0" />
          <span class="truncate">{compact() ? "No project" : "No project open"}</span>
        </div>
      }
    >
      <div class={`relative ${compact() ? 'w-[180px]' : 'w-[280px]'}`}>
        <select
          class="w-full appearance-none px-3 py-1.5 pr-8 text-sm bg-bg border border-border rounded-md text-txt focus:outline-none focus:ring-2 focus:ring-accent/50 focus:border-accent transition-colors cursor-pointer"
          value={props.selectedPath || ""}
          onChange={(e) => {
            const newPath = e.currentTarget.value;
            if (newPath) {
              const location = props.locations.find(l => l.path === newPath);
              props.onPathSelect(newPath, location?.id);
            }
          }}
          title={props.selectedPath || "Select project location"}
        >
          <option value="" disabled>Select location...</option>
          <For each={props.locations}>
            {(location) => (
              <option value={location.path || ""} title={location.path || ""}>
                {location.label}: {getFolderName(location.path)}
              </option>
            )}
          </For>
        </select>
        <HiOutlineChevronDown class="absolute right-2 top-1/2 -translate-y-1/2 w-4 h-4 text-txt-muted pointer-events-none" />
      </div>
    </Show>
  );
};
