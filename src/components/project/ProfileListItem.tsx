// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ProfileListItem - Individual profile card with actions
 */

import { Component, Show } from "solid-js";
import type { ProfileSummary } from "../../hooks/useWorkspaceProfiles";
import { HiOutlineCheck, HiOutlineEllipsisVertical } from "../icons";
import { getProfileTypeLabel, getProfileTypeColor } from "./profileHelpers";
import { ProfileActionMenu } from "./ProfileActionMenu";

interface ProfileListItemProps {
  profile: ProfileSummary;
  actionMenuOpen: boolean;
  onSelect: () => void;
  onToggleActionMenu: () => void;
  onClone: () => void;
  onExport: () => void;
  onDelete: () => void;
}

export const ProfileListItem: Component<ProfileListItemProps> = (props) => {
  return (
    <div
      class="flex items-center gap-3 p-3 hover:bg-bg-hover cursor-pointer transition-colors group"
      onClick={props.onSelect}
    >
      <div class="flex-1 min-w-0">
        <div class="flex items-center gap-2 mb-1">
          <span class="font-medium text-txt truncate">
            {props.profile.name}
          </span>
          <Show when={props.profile.is_active}>
            <HiOutlineCheck class="w-icon-sm h-icon-sm text-success flex-shrink-0" />
          </Show>
          <Show when={props.profile.is_default}>
            <span class="text-xs px-1.5 py-0.5 bg-accent/20 text-accent rounded">
              Default
            </span>
          </Show>
        </div>
        <div class="flex items-center gap-2">
          <span
            class={`text-xs ${getProfileTypeColor(props.profile.profile_type)}`}
          >
            {getProfileTypeLabel(props.profile.profile_type)}
          </span>
          <span class="text-xs text-txt-muted">
            • Used {props.profile.usage_count} times
          </span>
        </div>
        <Show when={props.profile.description}>
          <p class="text-xs text-txt-muted mt-1 truncate">
            {props.profile.description}
          </p>
        </Show>
      </div>
      
      {/* Action Menu Trigger */}
      <div class="relative">
        <button
          onClick={(e) => {
            e.stopPropagation();
            props.onToggleActionMenu();
          }}
          class="opacity-0 group-hover:opacity-100 p-1 hover:bg-bg-secondary rounded transition-all"
        >
          <HiOutlineEllipsisVertical class="w-icon-sm h-icon-sm text-txt-muted" />
        </button>
        
        <ProfileActionMenu
          profile={props.profile}
          isOpen={props.actionMenuOpen}
          onClone={() => {
            props.onClone();
          }}
          onExport={() => {
            props.onExport();
          }}
          onDelete={() => {
            props.onDelete();
          }}
        />
      </div>
    </div>
  );
};
