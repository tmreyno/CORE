// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ProfileActionMenu - Action menu for profile operations (clone, export, delete)
 */

import { Component, Show } from "solid-js";
import type { ProfileSummary } from "../../hooks/useWorkspaceProfiles";
import {
  HiOutlineDocumentDuplicate,
  HiOutlineArrowDownTray,
  HiOutlineTrash,
} from "../icons";

interface ProfileActionMenuProps {
  profile: ProfileSummary;
  isOpen: boolean;
  onClone: () => void;
  onExport: () => void;
  onDelete: () => void;
}

export const ProfileActionMenu: Component<ProfileActionMenuProps> = (props) => {
  const canDelete = () => !props.profile.is_active && !props.profile.is_default;
  
  return (
    <Show when={props.isOpen}>
      <div class="absolute right-0 top-full mt-1 w-40 bg-bg-panel border border-border rounded shadow-lg z-10">
        <button
          onClick={props.onClone}
          class="w-full flex items-center gap-2 px-3 py-2 text-sm text-txt hover:bg-bg-hover text-left"
        >
          <HiOutlineDocumentDuplicate class="w-4 h-4" />
          Clone
        </button>
        <button
          onClick={props.onExport}
          class="w-full flex items-center gap-2 px-3 py-2 text-sm text-txt hover:bg-bg-hover text-left"
        >
          <HiOutlineArrowDownTray class="w-4 h-4" />
          Export
        </button>
        <Show when={canDelete()}>
          <button
            onClick={props.onDelete}
            class="w-full flex items-center gap-2 px-3 py-2 text-sm text-error hover:bg-error/10 text-left"
          >
            <HiOutlineTrash class="w-4 h-4" />
            Delete
          </button>
        </Show>
      </div>
    </Show>
  );
};
