// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show, For, createSignal, onMount } from "solid-js";
import { makeEventListener } from "@solid-primitives/event-listener";
import { useFocusTrap } from "../../hooks/useFocusTrap";
import {
  HiOutlineUserCircle,
  HiOutlineXMark,
  HiOutlineCheckCircle,
  HiOutlineChevronDown,
  HiOutlineChevronRight,
} from "../icons";
import type { UserProfile, AppPreferences } from "../preferences";
import { applyProfileToPreferences } from "../preferences";

export interface UserConfirmModalProps {
  isOpen: boolean;
  onClose: () => void;
  /** Current list of saved profiles */
  profiles: UserProfile[];
  /** Currently selected default profile ID */
  defaultProfileId: string;
  /** Callback when user confirms with a profile */
  onConfirm: (profile: UserProfile) => void;
  /** Callback to apply profile to preferences */
  onUpdatePreference: <K extends keyof AppPreferences>(key: K, value: AppPreferences[K]) => void;
  /** Open settings to the Users tab */
  onOpenSettings?: () => void;
  /** Action context — "create" or "open" */
  action?: "create" | "open";
  /** Project name for display */
  projectName?: string;
}

export const UserConfirmModal: Component<UserConfirmModalProps> = (props) => {
  let modalRef: HTMLDivElement | undefined;
  useFocusTrap(() => modalRef, () => props.isOpen);

  const [selectedId, setSelectedId] = createSignal(props.defaultProfileId);
  const [showDetails, setShowDetails] = createSignal(true);

  // Reset selection when modal opens
  onMount(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && props.isOpen) {
        props.onClose();
      }
    };
    makeEventListener(document, "keydown", handleKeyDown);
  });

  // Sync selected ID when modal opens or defaultProfileId changes
  const activeProfile = () => props.profiles.find(p => p.id === selectedId());

  const handleConfirm = () => {
    const profile = activeProfile();
    if (profile) {
      // Apply profile fields to preferences
      applyProfileToPreferences(profile, props.onUpdatePreference);
      // Update the default if changed
      if (selectedId() !== props.defaultProfileId) {
        props.onUpdatePreference("defaultUserProfileId", selectedId());
      }
      props.onConfirm(profile);
    }
    props.onClose();
  };

  const handleSkip = () => {
    props.onClose();
  };

  const actionLabel = () => props.action === "create" ? "Creating" : "Opening";

  return (
    <Show when={props.isOpen}>
      <div class="modal-overlay">
        <div
          ref={modalRef}
          class="modal-content w-[440px] max-w-[90vw] flex flex-col"
          role="dialog"
          aria-modal="true"
          aria-labelledby="user-confirm-title"
        >
          {/* Header */}
          <div class="modal-header">
            <h2 id="user-confirm-title" class="text-sm font-semibold text-txt flex items-center gap-2">
              <HiOutlineUserCircle class="w-4 h-4" />
              Confirm Active User
            </h2>
            <button class="icon-btn-sm" onClick={handleSkip} aria-label="Close">
              <HiOutlineXMark class="w-3.5 h-3.5" />
            </button>
          </div>

          {/* Body */}
          <div class="modal-body">
            <Show when={props.projectName}>
              <p class="text-xs text-txt-muted mb-3">
                {actionLabel()} project <span class="font-medium text-txt">{props.projectName}</span>
              </p>
            </Show>

            <Show
              when={props.profiles.length > 0}
              fallback={
                <div class="text-center py-6">
                  <HiOutlineUserCircle class="w-10 h-10 text-txt-muted mx-auto mb-2" />
                  <p class="text-sm text-txt-muted mb-3">No user profiles configured</p>
                  <p class="text-xs text-txt-muted mb-4">
                    Create a user profile in Settings → Users & Profiles to track examiner info.
                  </p>
                  <Show when={props.onOpenSettings}>
                    <button
                      class="btn btn-primary text-xs"
                      onClick={() => {
                        props.onClose();
                        props.onOpenSettings?.();
                      }}
                    >
                      Open Settings
                    </button>
                  </Show>
                </div>
              }
            >
              {/* Profile selector */}
              <div class="mb-3">
                <label class="label mb-1">Active User</label>
                <select
                  class="input w-full"
                  value={selectedId()}
                  onChange={(e) => setSelectedId(e.currentTarget.value)}
                >
                  <For each={props.profiles}>
                    {(profile) => (
                      <option value={profile.id}>{profile.name}{profile.organization ? ` (${profile.organization})` : ""}</option>
                    )}
                  </For>
                </select>
              </div>

              {/* Selected profile details */}
              <Show when={activeProfile()}>
                {(profile) => (
                  <div class="rounded-lg border border-border bg-bg-secondary">
                    <button
                      class="flex items-center gap-2 px-3 py-2 w-full text-left"
                      onClick={() => setShowDetails(prev => !prev)}
                    >
                      <Show
                        when={showDetails()}
                        fallback={<HiOutlineChevronRight class="w-3 h-3 text-txt-muted" />}
                      >
                        <HiOutlineChevronDown class="w-3 h-3 text-txt-muted" />
                      </Show>
                      <span class="text-xs font-medium text-txt">Profile Details</span>
                    </button>
                    <Show when={showDetails()}>
                      <div class="px-3 pb-3 grid grid-cols-2 gap-x-4 gap-y-1.5 text-xs border-t border-border/50 pt-2">
                        <div>
                          <span class="text-txt-muted block">Name</span>
                          <span class="text-txt">{profile().name || "—"}</span>
                        </div>
                        <div>
                          <span class="text-txt-muted block">Title</span>
                          <span class="text-txt">{profile().title || "—"}</span>
                        </div>
                        <div>
                          <span class="text-txt-muted block">Organization</span>
                          <span class="text-txt">{profile().organization || "—"}</span>
                        </div>
                        <div>
                          <span class="text-txt-muted block">Agency</span>
                          <span class="text-txt">{profile().agency || "—"}</span>
                        </div>
                        <div>
                          <span class="text-txt-muted block">Badge</span>
                          <span class="text-txt">{profile().badgeNumber || "—"}</span>
                        </div>
                        <div>
                          <span class="text-txt-muted block">Email</span>
                          <span class="text-txt">{profile().email || "—"}</span>
                        </div>
                        <div>
                          <span class="text-txt-muted block">Phone</span>
                          <span class="text-txt">{profile().phone || "—"}</span>
                        </div>
                        <div>
                          <span class="text-txt-muted block">Case # Prefix</span>
                          <span class="text-txt">{profile().caseNumberPrefix || "—"}</span>
                        </div>
                        <Show when={profile().certifications?.length}>
                          <div class="col-span-2">
                            <span class="text-txt-muted block">Certifications</span>
                            <span class="text-txt">{profile().certifications.join(", ")}</span>
                          </div>
                        </Show>
                      </div>
                    </Show>
                  </div>
                )}
              </Show>
            </Show>
          </div>

          {/* Footer */}
          <div class="modal-footer justify-end">
            <button class="btn btn-secondary" onClick={handleSkip}>
              Skip
            </button>
            <Show when={props.profiles.length > 0}>
              <button
                class="btn btn-primary flex items-center gap-1.5"
                disabled={!activeProfile()}
                onClick={handleConfirm}
              >
                <HiOutlineCheckCircle class="w-3.5 h-3.5" />
                Confirm & Continue
              </button>
            </Show>
          </div>
        </div>
      </div>
    </Show>
  );
};
