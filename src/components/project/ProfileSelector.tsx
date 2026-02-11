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
  HiOutlineArrowUpTray,
} from "../icons";
import { ProfileListItem } from "./ProfileListItem";
import { ProfileFormDialog } from "./ProfileFormDialog";
import { ImportProfileDialog } from "./ImportProfileDialog";

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

  const openCloneDialog = (profile: ProfileSummary, e?: MouseEvent) => {
    if (e) e.stopPropagation();
    setActionMenuId(null);
    setCloneSourceId(profile.id);
    setNewProfileName(`${profile.name} (Copy)`);
    setShowCloneDialog(true);
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
                <ProfileListItem
                  profile={profile}
                  actionMenuOpen={actionMenuId() === profile.id}
                  onSelect={() => handleApplyProfile(profile.id)}
                  onToggleActionMenu={() => 
                    setActionMenuId(actionMenuId() === profile.id ? null : profile.id)
                  }
                  onClone={() => openCloneDialog(profile)}
                  onExport={() => handleExportProfile(profile.id, new MouseEvent('click'))}
                  onDelete={() => handleDeleteProfile(profile.id, new MouseEvent('click'))}
                />
              )}
            </For>
          </div>
        </div>
      </Show>

      {/* Save Profile Dialog */}
      <ProfileFormDialog
        isOpen={showSaveDialog()}
        title="Save Workspace Profile"
        mode="save"
        profileName={newProfileName()}
        profileType={newProfileType()}
        profileDesc={newProfileDesc()}
        loading={profiles.loading()}
        onNameChange={setNewProfileName}
        onTypeChange={setNewProfileType}
        onDescChange={setNewProfileDesc}
        onSubmit={handleSaveProfile}
        onCancel={() => {
          setShowSaveDialog(false);
          setNewProfileName("");
          setNewProfileType("Investigation");
          setNewProfileDesc("");
        }}
      />

      {/* Clone Profile Dialog */}
      <ProfileFormDialog
        isOpen={showCloneDialog()}
        title="Clone Profile"
        mode="clone"
        profileName={newProfileName()}
        loading={profiles.loading()}
        onNameChange={setNewProfileName}
        onSubmit={handleCloneProfile}
        onCancel={() => {
          setShowCloneDialog(false);
          setCloneSourceId(null);
          setNewProfileName("");
        }}
      />

      {/* Import Profile Dialog */}
      <ImportProfileDialog
        isOpen={showImportDialog()}
        importJson={importJson()}
        loading={profiles.loading()}
        onJsonChange={setImportJson}
        onSubmit={handleImportProfile}
        onCancel={() => {
          setShowImportDialog(false);
          setImportJson("");
        }}
      />
    </div>
  );
};
