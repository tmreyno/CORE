// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show, For, createSignal, onMount } from "solid-js";
import { useWorkspaceProfiles, type ProfileType, type ProfileSummary } from "../../hooks/useWorkspaceProfiles";
import { 
  HiOutlineBookmark, 
  HiOutlineChevronDown, 
  HiOutlinePlus, 
  HiOutlineTrash, 
  HiOutlineCheck,
  HiOutlineDocumentDuplicate,
  HiOutlineArrowDownTray,
  HiOutlineArrowUpTray,
  HiOutlineEllipsisVertical,
} from "../icons";

interface ProfileSelectorProps {
  onProfileChange?: (profileId: string) => void;
}

export const ProfileSelector: Component<ProfileSelectorProps> = (props) => {
  const profiles = useWorkspaceProfiles();
  const [isOpen, setIsOpen] = createSignal(false);
  const [showSaveDialog, setShowSaveDialog] = createSignal(false);
  const [showCloneDialog, setShowCloneDialog] = createSignal(false);
  const [showImportDialog, setShowImportDialog] = createSignal(false);
  const [cloneSourceId, setCloneSourceId] = createSignal<string | null>(null);
  const [newProfileName, setNewProfileName] = createSignal("");
  const [newProfileType, setNewProfileType] = createSignal<ProfileType>("Investigation");
  const [newProfileDesc, setNewProfileDesc] = createSignal("");
  const [importJson, setImportJson] = createSignal("");
  const [actionMenuId, setActionMenuId] = createSignal<string | null>(null);

  onMount(async () => {
    await profiles.listProfiles();
    await profiles.getActiveProfile();
  });

  const handleApplyProfile = async (profileId: string) => {
    const success = await profiles.setActiveProfile(profileId);
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
      setNewProfileType("Investigation");
      setNewProfileDesc("");
    }
  };

  const handleCloneProfile = async () => {
    const sourceId = cloneSourceId();
    const name = newProfileName().trim();
    if (!sourceId || !name) return;

    const newId = await profiles.cloneProfile(sourceId, name);
    if (newId) {
      setShowCloneDialog(false);
      setCloneSourceId(null);
      setNewProfileName("");
    }
  };

  const handleExportProfile = async (profileId: string, e: MouseEvent) => {
    e.stopPropagation();
    setActionMenuId(null);
    
    const json = await profiles.exportProfile(profileId);
    if (json) {
      // Create a download link
      const blob = new Blob([json], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `profile_${profileId}.json`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    }
  };

  const handleImportProfile = async () => {
    const json = importJson().trim();
    if (!json) return;

    const id = await profiles.importProfile(json);
    if (id) {
      setShowImportDialog(false);
      setImportJson("");
    }
  };

  const handleDeleteProfile = async (profileId: string, e: MouseEvent) => {
    e.stopPropagation();
    setActionMenuId(null);
    const confirmed = confirm("Delete this profile? This action cannot be undone.");
    if (!confirmed) return;
    await profiles.deleteProfile(profileId);
  };

  const openCloneDialog = (profile: ProfileSummary, e: MouseEvent) => {
    e.stopPropagation();
    setActionMenuId(null);
    setCloneSourceId(profile.id);
    setNewProfileName(`${profile.name} (Copy)`);
    setShowCloneDialog(true);
  };

  const getProfileTypeLabel = (type: ProfileType): string => {
    const labels: Record<ProfileType, string> = {
      Investigation: "Investigation",
      Analysis: "Analysis",
      Review: "Review",
      Mobile: "Mobile",
      Computer: "Computer",
      Network: "Network",
      IncidentResponse: "Incident Response",
      Custom: "Custom",
    };
    return labels[type] || type;
  };

  const getProfileTypeColor = (type: ProfileType): string => {
    const colors: Record<ProfileType, string> = {
      Investigation: "text-type-ad1",
      Analysis: "text-type-e01",
      Review: "text-warning",
      Mobile: "text-type-ufed",
      Computer: "text-info",
      Network: "text-accent",
      IncidentResponse: "text-error",
      Custom: "text-txt-secondary",
    };
    return colors[type] || "text-txt-secondary";
  };

  return (
    <div class="relative">
      {/* Trigger Button */}
      <button
        onClick={() => setIsOpen(!isOpen())}
        class="flex items-center gap-2 px-3 py-2 bg-bg-panel hover:bg-bg-hover rounded-md border border-border transition-colors"
      >
        <HiOutlineBookmark class="w-icon-sm h-icon-sm text-accent" />
        <span class="text-sm text-txt">
          {profiles.currentProfile()?.name || "Workspace Profile"}
        </span>
        <HiOutlineChevronDown
          class={`w-icon-sm h-icon-sm text-txt-muted transition-transform ${
            isOpen() ? "rotate-180" : ""
          }`}
        />
      </button>

      {/* Dropdown Menu */}
      <Show when={isOpen()}>
        <div class="absolute top-full right-0 mt-2 w-96 bg-bg rounded-md border border-border shadow-lg z-dropdown">
          {/* Header */}
          <div class="flex items-center justify-between p-3 border-b border-border">
            <h3 class="font-semibold text-txt">Workspace Profiles</h3>
            <div class="flex items-center gap-2">
              <button
                onClick={() => setShowImportDialog(true)}
                class="p-1.5 hover:bg-bg-hover rounded transition-colors"
                title="Import Profile"
              >
                <HiOutlineArrowUpTray class="w-4 h-4 text-txt-muted" />
              </button>
              <button
                onClick={() => setShowSaveDialog(true)}
                class="flex items-center gap-1 px-2 py-1 text-xs bg-accent hover:bg-accent-hover text-white rounded transition-colors"
              >
                <HiOutlinePlus class="w-3 h-3" />
                Save Current
              </button>
            </div>
          </div>

          {/* Profile List */}
          <div class="max-h-96 overflow-y-auto">
            <Show when={profiles.profiles().length === 0}>
              <div class="p-8 text-center text-txt-muted text-sm">
                No saved profiles
              </div>
            </Show>

            <For each={profiles.profiles()}>
              {(profile) => (
                <div
                  class="flex items-center gap-3 p-3 hover:bg-bg-hover cursor-pointer transition-colors group"
                  onClick={() => handleApplyProfile(profile.id)}
                >
                  <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2 mb-1">
                      <span class="font-medium text-txt truncate">
                        {profile.name}
                      </span>
                      <Show when={profile.is_active}>
                        <HiOutlineCheck class="w-icon-sm h-icon-sm text-success flex-shrink-0" />
                      </Show>
                      <Show when={profile.is_default}>
                        <span class="text-xs px-1.5 py-0.5 bg-accent/20 text-accent rounded">
                          Default
                        </span>
                      </Show>
                    </div>
                    <div class="flex items-center gap-2">
                      <span
                        class={`text-xs ${getProfileTypeColor(profile.profile_type)}`}
                      >
                        {getProfileTypeLabel(profile.profile_type)}
                      </span>
                      <span class="text-xs text-txt-muted">
                        • Used {profile.usage_count} times
                      </span>
                    </div>
                    <Show when={profile.description}>
                      <p class="text-xs text-txt-muted mt-1 truncate">
                        {profile.description}
                      </p>
                    </Show>
                  </div>
                  
                  {/* Action Menu */}
                  <div class="relative">
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        setActionMenuId(actionMenuId() === profile.id ? null : profile.id);
                      }}
                      class="opacity-0 group-hover:opacity-100 p-1 hover:bg-bg-secondary rounded transition-all"
                    >
                      <HiOutlineEllipsisVertical class="w-icon-sm h-icon-sm text-txt-muted" />
                    </button>
                    
                    <Show when={actionMenuId() === profile.id}>
                      <div class="absolute right-0 top-full mt-1 w-40 bg-bg-panel border border-border rounded shadow-lg z-10">
                        <button
                          onClick={(e) => openCloneDialog(profile, e)}
                          class="w-full flex items-center gap-2 px-3 py-2 text-sm text-txt hover:bg-bg-hover text-left"
                        >
                          <HiOutlineDocumentDuplicate class="w-4 h-4" />
                          Clone
                        </button>
                        <button
                          onClick={(e) => handleExportProfile(profile.id, e)}
                          class="w-full flex items-center gap-2 px-3 py-2 text-sm text-txt hover:bg-bg-hover text-left"
                        >
                          <HiOutlineArrowDownTray class="w-4 h-4" />
                          Export
                        </button>
                        <Show when={!profile.is_active && !profile.is_default}>
                          <button
                            onClick={(e) => handleDeleteProfile(profile.id, e)}
                            class="w-full flex items-center gap-2 px-3 py-2 text-sm text-error hover:bg-error/10 text-left"
                          >
                            <HiOutlineTrash class="w-4 h-4" />
                            Delete
                          </button>
                        </Show>
                      </div>
                    </Show>
                  </div>
                </div>
              )}
            </For>
          </div>
        </div>
      </Show>

      {/* Save Profile Dialog */}
      <Show when={showSaveDialog()}>
        <div class="modal-overlay">
          <div class="modal-content max-w-md w-full">
            <div class="modal-header">
              <h3 class="text-lg font-semibold text-txt">Save Workspace Profile</h3>
            </div>

            <div class="modal-body space-y-4">
              {/* Profile Name */}
              <div class="form-group">
                <label class="label">
                  Profile Name
                </label>
                <input
                  type="text"
                  value={newProfileName()}
                  onInput={(e) => setNewProfileName(e.currentTarget.value)}
                  placeholder="e.g., Mobile Investigation Layout"
                  class="input"
                />
              </div>

              {/* Profile Type */}
              <div>
                <label class="block text-sm font-medium text-txt mb-2">
                  Profile Type
                </label>
                <select
                  value={newProfileType()}
                  onChange={(e) => setNewProfileType(e.currentTarget.value as ProfileType)}
                  class="input"
                >
                  <option value="Investigation">Investigation</option>
                  <option value="Analysis">Analysis</option>
                  <option value="Review">Review</option>
                  <option value="Mobile">Mobile Forensics</option>
                  <option value="Computer">Computer Forensics</option>
                  <option value="Network">Network Forensics</option>
                  <option value="IncidentResponse">Incident Response</option>
                  <option value="Custom">Custom</option>
                </select>
              </div>

              {/* Description */}
              <div class="form-group">
                <label class="label">
                  Description (Optional)
                </label>
                <textarea
                  value={newProfileDesc()}
                  onInput={(e) => setNewProfileDesc(e.currentTarget.value)}
                  placeholder="Describe this workspace layout..."
                  rows={3}
                  class="textarea"
                />
              </div>
            </div>

            <div class="modal-footer justify-end">
              <button
                onClick={() => setShowSaveDialog(false)}
                class="btn btn-secondary"
              >
                Cancel
              </button>
              <button
                onClick={handleSaveProfile}
                disabled={!newProfileName().trim() || profiles.loading()}
                class="btn btn-primary"
              >
                Save Profile
              </button>
            </div>
          </div>
        </div>
      </Show>

      {/* Clone Profile Dialog */}
      <Show when={showCloneDialog()}>
        <div class="modal-overlay">
          <div class="modal-content max-w-md w-full">
            <div class="modal-header">
              <h3 class="text-lg font-semibold text-txt">Clone Profile</h3>
            </div>

            <div class="modal-body space-y-4">
              <div class="form-group">
                <label class="label">
                  New Profile Name
                </label>
                <input
                  type="text"
                  value={newProfileName()}
                  onInput={(e) => setNewProfileName(e.currentTarget.value)}
                  placeholder="e.g., My Custom Profile"
                  class="input"
                />
              </div>
            </div>

            <div class="modal-footer justify-end">
              <button
                onClick={() => {
                  setShowCloneDialog(false);
                  setCloneSourceId(null);
                  setNewProfileName("");
                }}
                class="btn btn-secondary"
              >
                Cancel
              </button>
              <button
                onClick={handleCloneProfile}
                disabled={!newProfileName().trim() || profiles.loading()}
                class="btn btn-primary"
              >
                Clone
              </button>
            </div>
          </div>
        </div>
      </Show>

      {/* Import Profile Dialog */}
      <Show when={showImportDialog()}>
        <div class="modal-overlay">
          <div class="modal-content max-w-lg w-full">
            <div class="modal-header">
              <h3 class="text-lg font-semibold text-txt">Import Profile</h3>
            </div>

            <div class="modal-body space-y-4">
              <div class="form-group">
                <label class="label">
                  Profile JSON
                </label>
                <textarea
                  value={importJson()}
                  onInput={(e) => setImportJson(e.currentTarget.value)}
                  placeholder="Paste exported profile JSON here..."
                  rows={10}
                  class="textarea font-mono text-sm"
                />
              </div>
            </div>

            <div class="modal-footer justify-end">
              <button
                onClick={() => {
                  setShowImportDialog(false);
                  setImportJson("");
                }}
                class="btn btn-secondary"
              >
                Cancel
              </button>
              <button
                onClick={handleImportProfile}
                disabled={!importJson().trim() || profiles.loading()}
                class="btn btn-primary"
              >
                Import
              </button>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
};
