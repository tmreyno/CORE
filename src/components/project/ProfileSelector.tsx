// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show, For, createSignal, onMount } from "solid-js";
import { useWorkspaceProfiles, type ProfileType } from "../../hooks/useWorkspaceProfiles";
import { HiOutlineBookmark, HiOutlineChevronDown, HiOutlinePlus, HiOutlineTrash, HiOutlineCheck } from "../icons";

interface ProfileSelectorProps {
  onProfileChange?: (profileId: string) => void;
}

export const ProfileSelector: Component<ProfileSelectorProps> = (props) => {
  const profiles = useWorkspaceProfiles();
  const [isOpen, setIsOpen] = createSignal(false);
  const [showSaveDialog, setShowSaveDialog] = createSignal(false);
  const [newProfileName, setNewProfileName] = createSignal("");
  const [newProfileType, setNewProfileType] = createSignal<ProfileType>("investigation");
  const [newProfileDesc, setNewProfileDesc] = createSignal("");

  onMount(() => {
    profiles.listProfiles();
  });

  const handleApplyProfile = async (profileId: string) => {
    const success = await profiles.applyProfile(profileId);
    if (success) {
      setIsOpen(false);
      props.onProfileChange?.(profileId);
    }
  };

  const handleSaveProfile = async () => {
    const name = newProfileName().trim();
    if (!name) return;

    const profileId = await profiles.saveProfile(
      name,
      newProfileType(),
      newProfileDesc()
    );

    if (profileId) {
      setShowSaveDialog(false);
      setNewProfileName("");
      setNewProfileType("investigation");
      setNewProfileDesc("");
    }
  };

  const handleDeleteProfile = async (profileId: string, e: MouseEvent) => {
    e.stopPropagation();
    const confirmed = confirm("Delete this profile? This action cannot be undone.");
    if (!confirmed) return;
    await profiles.deleteProfile(profileId);
  };

  const getProfileTypeLabel = (type: ProfileType): string => {
    const labels: Record<ProfileType, string> = {
      investigation: "Investigation",
      analysis: "Analysis",
      reporting: "Reporting",
      review: "Review",
      triage: "Triage",
      acquisition: "Acquisition",
      preservation: "Preservation",
      custom: "Custom",
    };
    return labels[type];
  };

  const getProfileTypeColor = (type: ProfileType): string => {
    const colors: Record<ProfileType, string> = {
      investigation: "text-type-ad1",
      analysis: "text-type-e01",
      reporting: "text-accent",
      review: "text-warning",
      triage: "text-info",
      acquisition: "text-success",
      preservation: "text-type-ufed",
      custom: "text-txt-secondary",
    };
    return colors[type];
  };

  return (
    <div className="relative">
      {/* Trigger Button */}
      <button
        onClick={() => setIsOpen(!isOpen())}
        className="flex items-center gap-2 px-3 py-2 bg-bg-panel hover:bg-bg-hover rounded-md border border-border transition-colors"
      >
        <HiOutlineBookmark className="w-icon-sm h-icon-sm text-accent" />
        <span className="text-sm text-txt">
          {profiles.currentProfile()?.name || "Workspace Profile"}
        </span>
        <HiOutlineChevronDown
          className={`w-icon-sm h-icon-sm text-txt-muted transition-transform ${
            isOpen() ? "rotate-180" : ""
          }`}
        />
      </button>

      {/* Dropdown Menu */}
      <Show when={isOpen()}>
        <div className="absolute top-full right-0 mt-2 w-80 bg-bg rounded-md border border-border shadow-lg z-dropdown">
          {/* Header */}
          <div className="flex items-center justify-between p-3 border-b border-border">
            <h3 className="font-semibold text-txt">Workspace Profiles</h3>
            <button
              onClick={() => setShowSaveDialog(true)}
              className="flex items-center gap-1 px-2 py-1 text-xs bg-accent hover:bg-accent-hover text-white rounded transition-colors"
            >
              <HiOutlinePlus className="w-3 h-3" />
              Save Current
            </button>
          </div>

          {/* Profile List */}
          <div className="max-h-96 overflow-y-auto">
            <Show when={profiles.profiles().length === 0}>
              <div className="p-8 text-center text-txt-muted text-sm">
                No saved profiles
              </div>
            </Show>

            <For each={profiles.profiles()}>
              {(profile) => (
                <div
                  className="flex items-center gap-3 p-3 hover:bg-bg-hover cursor-pointer transition-colors group"
                  onClick={() => handleApplyProfile(profile.id)}
                >
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 mb-1">
                      <span className="font-medium text-txt truncate">
                        {profile.name}
                      </span>
                      <Show when={profiles.currentProfile()?.id === profile.id}>
                        <HiOutlineCheck className="w-icon-sm h-icon-sm text-success flex-shrink-0" />
                      </Show>
                    </div>
                    <div className="flex items-center gap-2">
                      <span
                        className={`text-xs ${getProfileTypeColor(
                          profile.profile_type
                        )}`}
                      >
                        {getProfileTypeLabel(profile.profile_type)}
                      </span>
                      <span className="text-xs text-txt-muted">
                        • Used {profile.use_count} times
                      </span>
                    </div>
                    <p className="text-xs text-txt-muted mt-1">
                      Last used: {new Date(profile.last_used).toLocaleDateString()}
                    </p>
                  </div>
                  <button
                    onClick={(e) => handleDeleteProfile(profile.id, e)}
                    className="opacity-0 group-hover:opacity-100 p-1 hover:bg-error/20 rounded transition-all"
                  >
                    <HiOutlineTrash className="w-icon-sm h-icon-sm text-error" />
                  </button>
                </div>
              )}
            </For>
          </div>
        </div>
      </Show>

      {/* Save Profile Dialog */}
      <Show when={showSaveDialog()}>
        <div className="fixed inset-0 z-modal-backdrop bg-black/50 flex items-center justify-center p-4">
          <div className="bg-bg rounded-lg shadow-xl max-w-md w-full">
            <div className="p-6 border-b border-border">
              <h3 className="text-lg font-semibold text-txt">Save Workspace Profile</h3>
            </div>

            <div className="p-6 space-y-4">
              {/* Profile Name */}
              <div>
                <label className="block text-sm font-medium text-txt mb-2">
                  Profile Name
                </label>
                <input
                  type="text"
                  value={newProfileName()}
                  onInput={(e) => setNewProfileName(e.currentTarget.value)}
                  placeholder="e.g., Mobile Investigation Layout"
                  className="w-full px-3 py-2 bg-bg-panel border border-border rounded-md text-txt placeholder-txt-muted focus:outline-none focus:border-accent"
                />
              </div>

              {/* Profile Type */}
              <div>
                <label className="block text-sm font-medium text-txt mb-2">
                  Profile Type
                </label>
                <select
                  value={newProfileType()}
                  onChange={(e) => setNewProfileType(e.currentTarget.value as ProfileType)}
                  className="w-full px-3 py-2 bg-bg-panel border border-border rounded-md text-txt focus:outline-none focus:border-accent"
                >
                  <option value="investigation">Investigation</option>
                  <option value="analysis">Analysis</option>
                  <option value="reporting">Reporting</option>
                  <option value="review">Review</option>
                  <option value="triage">Triage</option>
                  <option value="acquisition">Acquisition</option>
                  <option value="preservation">Preservation</option>
                  <option value="custom">Custom</option>
                </select>
              </div>

              {/* Description */}
              <div>
                <label className="block text-sm font-medium text-txt mb-2">
                  Description (Optional)
                </label>
                <textarea
                  value={newProfileDesc()}
                  onInput={(e) => setNewProfileDesc(e.currentTarget.value)}
                  placeholder="Describe this workspace layout..."
                  rows={3}
                  className="w-full px-3 py-2 bg-bg-panel border border-border rounded-md text-txt placeholder-txt-muted focus:outline-none focus:border-accent resize-none"
                />
              </div>
            </div>

            <div className="flex justify-end gap-3 p-6 border-t border-border">
              <button
                onClick={() => setShowSaveDialog(false)}
                className="px-4 py-2 bg-bg-secondary hover:bg-bg-hover text-txt rounded-md transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={handleSaveProfile}
                disabled={!newProfileName().trim() || profiles.loading()}
                className="px-4 py-2 bg-accent hover:bg-accent-hover text-white rounded-md transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Save Profile
              </button>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
};
