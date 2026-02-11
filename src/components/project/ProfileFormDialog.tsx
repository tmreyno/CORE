// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ProfileFormDialog - Reusable form dialog for save/clone profile operations
 */

import { Component, Show, For } from "solid-js";
import type { ProfileType } from "../../hooks/useWorkspaceProfiles";
import { PROFILE_TYPE_OPTIONS } from "./profileHelpers";

interface ProfileFormDialogProps {
  isOpen: boolean;
  title: string;
  mode: "save" | "clone";
  profileName: string;
  profileType?: ProfileType;
  profileDesc?: string;
  loading?: boolean;
  onNameChange: (name: string) => void;
  onTypeChange?: (type: ProfileType) => void;
  onDescChange?: (desc: string) => void;
  onSubmit: () => void;
  onCancel: () => void;
}

export const ProfileFormDialog: Component<ProfileFormDialogProps> = (props) => {
  const showTypeField = () => props.mode === "save";
  const showDescField = () => props.mode === "save";
  const submitLabel = () => props.mode === "save" ? "Save Profile" : "Clone";
  
  return (
    <Show when={props.isOpen}>
      <div class="modal-overlay">
        <div class="modal-content max-w-md w-full">
          <div class="modal-header">
            <h3 class="text-lg font-semibold text-txt">{props.title}</h3>
          </div>

          <div class="modal-body space-y-4">
            {/* Profile Name */}
            <div class="form-group">
              <label class="label">
                {props.mode === "clone" ? "New Profile Name" : "Profile Name"}
              </label>
              <input
                type="text"
                value={props.profileName}
                onInput={(e) => props.onNameChange(e.currentTarget.value)}
                placeholder={
                  props.mode === "clone" 
                    ? "e.g., My Custom Profile" 
                    : "e.g., Mobile Investigation Layout"
                }
                class="input"
              />
            </div>

            {/* Profile Type (Save mode only) */}
            <Show when={showTypeField()}>
              <div class="form-group">
                <label class="label">Profile Type</label>
                <select
                  value={props.profileType || "Investigation"}
                  onChange={(e) => props.onTypeChange?.(e.currentTarget.value as ProfileType)}
                  class="input"
                >
                  <For each={PROFILE_TYPE_OPTIONS}>
                    {(option) => (
                      <option value={option.value}>{option.label}</option>
                    )}
                  </For>
                </select>
              </div>
            </Show>

            {/* Description (Save mode only) */}
            <Show when={showDescField()}>
              <div class="form-group">
                <label class="label">Description (Optional)</label>
                <textarea
                  value={props.profileDesc || ""}
                  onInput={(e) => props.onDescChange?.(e.currentTarget.value)}
                  placeholder="Describe this workspace layout..."
                  rows={3}
                  class="textarea"
                />
              </div>
            </Show>
          </div>

          <div class="modal-footer justify-end">
            <button
              onClick={props.onCancel}
              class="btn-sm"
            >
              Cancel
            </button>
            <button
              onClick={props.onSubmit}
              disabled={!props.profileName.trim() || props.loading}
              class="btn-sm-primary"
            >
              {submitLabel()}
            </button>
          </div>
        </div>
      </div>
    </Show>
  );
};
