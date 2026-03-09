// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, createSignal, Show, For } from "solid-js";
import { open } from "@tauri-apps/plugin-dialog";
import { Toggle } from "../ui";
import { SettingGroup, SettingRow } from "../settings";
import {
  HiOutlineUserPlus,
  HiOutlinePencilSquare,
  HiOutlineTrash,
  HiOutlineCheckCircle,
  HiOutlineXMark,
  HiOutlineUserCircle,
  HiOutlineChevronDown,
  HiOutlineChevronRight,
} from "../icons";
import type { AppPreferences, UserProfile } from "../preferences";
import {
  createEmptyProfile,
  applyProfileToPreferences,
} from "../preferences";
import { logger } from "../../utils/logger";
const log = logger.scope("UserProfilesTab");

interface UserProfilesSettingsProps {
  preferences: AppPreferences;
  onUpdate: <K extends keyof AppPreferences>(key: K, value: AppPreferences[K]) => void;
}

export const UserProfilesSettings: Component<UserProfilesSettingsProps> = (props) => {
  const [editingProfile, setEditingProfile] = createSignal<UserProfile | null>(null);
  const [isNew, setIsNew] = createSignal(false);
  const [expandedId, setExpandedId] = createSignal<string | null>(null);

  const profiles = () => props.preferences.userProfiles || [];
  const defaultId = () => props.preferences.defaultUserProfileId || "";

  // -------------------------------------------------------------------------
  // CRUD
  // -------------------------------------------------------------------------

  const handleAdd = () => {
    const profile = createEmptyProfile("New User");
    setEditingProfile(profile);
    setIsNew(true);
  };

  const handleEdit = (profile: UserProfile) => {
    setEditingProfile({ ...profile, certifications: [...profile.certifications] });
    setIsNew(false);
  };

  const handleSave = () => {
    const profile = editingProfile();
    if (!profile || !profile.name.trim()) return;

    const list = [...profiles()];
    if (isNew()) {
      list.push(profile);
    } else {
      const idx = list.findIndex(p => p.id === profile.id);
      if (idx >= 0) list[idx] = profile;
    }
    props.onUpdate("userProfiles", list);

    // If this is the only profile and none selected yet, auto-select it
    if (list.length === 1 && !defaultId()) {
      props.onUpdate("defaultUserProfileId", profile.id);
      applyProfileToPreferences(profile, props.onUpdate);
    }

    setEditingProfile(null);
    setIsNew(false);
    log.info(`Profile saved: ${profile.name}`);
  };

  const handleCancel = () => {
    setEditingProfile(null);
    setIsNew(false);
  };

  const handleDelete = (id: string) => {
    const list = profiles().filter(p => p.id !== id);
    props.onUpdate("userProfiles", list);
    if (defaultId() === id) {
      const newDefault = list.length > 0 ? list[0].id : "";
      props.onUpdate("defaultUserProfileId", newDefault);
      if (newDefault) {
        const p = list.find(x => x.id === newDefault);
        if (p) applyProfileToPreferences(p, props.onUpdate);
      }
    }
    log.info(`Profile deleted: ${id}`);
  };

  const handleSetDefault = (id: string) => {
    props.onUpdate("defaultUserProfileId", id);
    const profile = profiles().find(p => p.id === id);
    if (profile) {
      applyProfileToPreferences(profile, props.onUpdate);
      log.info(`Default profile set: ${profile.name}`);
    }
  };

  const handleBrowseLogo = async () => {
    try {
      const selected = await open({
        multiple: false,
        title: "Select Logo Image",
        filters: [{ name: "Images", extensions: ["png", "jpg", "jpeg", "svg"] }],
      });
      if (selected && typeof selected === "string") {
        const profile = editingProfile();
        if (profile) {
          setEditingProfile({ ...profile, logoPath: selected });
        }
      }
    } catch (err) {
      log.error("Failed to open file dialog:", err);
    }
  };

  const toggleExpand = (id: string) => {
    setExpandedId(prev => prev === id ? null : id);
  };

  // Editable field helper
  const updateField = <K extends keyof UserProfile>(key: K, value: UserProfile[K]) => {
    const profile = editingProfile();
    if (profile) {
      setEditingProfile({ ...profile, [key]: value });
    }
  };

  return (
    <>
      {/* Active User */}
      <SettingGroup title="Active User" description="Default user for new projects and reports">
        <Show
          when={profiles().length > 0}
          fallback={
            <div class="text-xs text-txt-muted py-2">
              No user profiles created yet. Add a profile below to get started.
            </div>
          }
        >
          <SettingRow label="Default User" description="Used for new projects and report branding">
            <select
              class="input-inline"
              value={defaultId()}
              onChange={(e) => handleSetDefault(e.currentTarget.value)}
            >
              <option value="">— Select User —</option>
              <For each={profiles()}>
                {(profile) => (
                  <option value={profile.id}>{profile.name}</option>
                )}
              </For>
            </select>
          </SettingRow>
        </Show>

        <SettingRow label="Confirm on Project Open" description="Show user confirmation when opening or creating projects">
          <Toggle
            checked={props.preferences.confirmUserOnProjectOpen}
            onChange={(v) => props.onUpdate("confirmUserOnProjectOpen", v)}
          />
        </SettingRow>
      </SettingGroup>

      {/* Profile List */}
      <SettingGroup title="User Profiles" description="Manage examiner profiles with branding preferences">
        <div class="flex flex-col gap-2">
          <For each={profiles()}>
            {(profile) => (
              <div
                class={`rounded-lg border transition-colors ${
                  defaultId() === profile.id
                    ? "border-accent/40 bg-accent/5"
                    : "border-border bg-bg-secondary"
                }`}
              >
                {/* Profile header row */}
                <div class="flex items-center gap-2 px-3 py-2 cursor-pointer"
                  onClick={() => toggleExpand(profile.id)}
                >
                  <Show
                    when={expandedId() === profile.id}
                    fallback={<HiOutlineChevronRight class="w-3.5 h-3.5 text-txt-muted" />}
                  >
                    <HiOutlineChevronDown class="w-3.5 h-3.5 text-txt-muted" />
                  </Show>
                  <HiOutlineUserCircle class="w-5 h-5 text-txt-secondary" />
                  <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2">
                      <span class="text-sm font-medium text-txt truncate">{profile.name}</span>
                      <Show when={defaultId() === profile.id}>
                        <span class="badge badge-success text-[10px]">Default</span>
                      </Show>
                    </div>
                    <span class="text-xs text-txt-muted truncate block">
                      {[profile.title, profile.organization].filter(Boolean).join(" · ") || "No details"}
                    </span>
                  </div>
                  <div class="flex items-center gap-1" onClick={(e) => e.stopPropagation()}>
                    <Show when={defaultId() !== profile.id}>
                      <button
                        class="icon-btn-sm"
                        title="Set as default"
                        onClick={() => handleSetDefault(profile.id)}
                      >
                        <HiOutlineCheckCircle class="w-3.5 h-3.5" />
                      </button>
                    </Show>
                    <button
                      class="icon-btn-sm"
                      title="Edit profile"
                      onClick={() => handleEdit(profile)}
                    >
                      <HiOutlinePencilSquare class="w-3.5 h-3.5" />
                    </button>
                    <button
                      class="icon-btn-sm hover:text-error"
                      title="Delete profile"
                      onClick={() => handleDelete(profile.id)}
                    >
                      <HiOutlineTrash class="w-3.5 h-3.5" />
                    </button>
                  </div>
                </div>

                {/* Expanded detail */}
                <Show when={expandedId() === profile.id}>
                  <div class="px-3 pb-3 pt-1 border-t border-border/50 grid grid-cols-2 gap-x-4 gap-y-1 text-xs">
                    <div><span class="text-txt-muted">Email:</span> <span class="text-txt">{profile.email || "—"}</span></div>
                    <div><span class="text-txt-muted">Phone:</span> <span class="text-txt">{profile.phone || "—"}</span></div>
                    <div><span class="text-txt-muted">Badge:</span> <span class="text-txt">{profile.badgeNumber || "—"}</span></div>
                    <div><span class="text-txt-muted">Agency:</span> <span class="text-txt">{profile.agency || "—"}</span></div>
                    <div class="col-span-2">
                      <span class="text-txt-muted">Certifications:</span>{" "}
                      <span class="text-txt">{profile.certifications?.length ? profile.certifications.join(", ") : "—"}</span>
                    </div>
                    <div class="col-span-2">
                      <span class="text-txt-muted">Case # Prefix:</span>{" "}
                      <span class="text-txt">{profile.caseNumberPrefix || "—"}</span>
                    </div>
                  </div>
                </Show>
              </div>
            )}
          </For>

          {/* Add button */}
          <button
            class="btn btn-secondary flex items-center gap-1.5 text-xs self-start"
            onClick={handleAdd}
          >
            <HiOutlineUserPlus class="w-3.5 h-3.5" />
            Add Profile
          </button>
        </div>
      </SettingGroup>

      {/* Edit / Create Form (modal overlay within the tab) */}
      <Show when={editingProfile()}>
        {(profile) => (
          <div class="fixed inset-0 z-modal-backdrop bg-black/40 flex items-center justify-center">
            <div class="bg-bg-panel rounded-xl border border-border w-[480px] max-h-[80vh] flex flex-col shadow-xl">
              {/* Modal header */}
              <div class="modal-header shrink-0">
                <h3 class="text-sm font-semibold text-txt flex items-center gap-2">
                  <HiOutlineUserCircle class="w-4 h-4" />
                  {isNew() ? "New User Profile" : "Edit User Profile"}
                </h3>
                <button class="icon-btn-sm" onClick={handleCancel}>
                  <HiOutlineXMark class="w-3.5 h-3.5" />
                </button>
              </div>

              {/* Form body */}
              <div class="p-4 overflow-y-auto flex flex-col gap-3">
                {/* Identity */}
                <div class="text-xs font-semibold text-txt-muted uppercase tracking-wide">Identity</div>
                <div class="grid grid-cols-2 gap-3">
                  <div class="col-span-2">
                    <label class="label">Full Name *</label>
                    <input
                      type="text"
                      class="input"
                      value={profile().name}
                      onInput={(e) => updateField("name", e.currentTarget.value)}
                      placeholder="Jane Doe"
                    />
                  </div>
                  <div>
                    <label class="label">Title / Role</label>
                    <input
                      type="text"
                      class="input"
                      value={profile().title}
                      onInput={(e) => updateField("title", e.currentTarget.value)}
                      placeholder="Senior Digital Forensic Examiner"
                    />
                  </div>
                  <div>
                    <label class="label">Badge / ID Number</label>
                    <input
                      type="text"
                      class="input"
                      value={profile().badgeNumber}
                      onInput={(e) => updateField("badgeNumber", e.currentTarget.value)}
                      placeholder="12345"
                    />
                  </div>
                </div>

                {/* Organization */}
                <div class="text-xs font-semibold text-txt-muted uppercase tracking-wide mt-2">Organization</div>
                <div class="grid grid-cols-2 gap-3">
                  <div>
                    <label class="label">Organization</label>
                    <input
                      type="text"
                      class="input"
                      value={profile().organization}
                      onInput={(e) => updateField("organization", e.currentTarget.value)}
                      placeholder="Metro PD Digital Forensics Lab"
                    />
                  </div>
                  <div>
                    <label class="label">Agency</label>
                    <input
                      type="text"
                      class="input"
                      value={profile().agency}
                      onInput={(e) => updateField("agency", e.currentTarget.value)}
                      placeholder="Metro Police Department"
                    />
                  </div>
                </div>

                {/* Contact */}
                <div class="text-xs font-semibold text-txt-muted uppercase tracking-wide mt-2">Contact</div>
                <div class="grid grid-cols-2 gap-3">
                  <div>
                    <label class="label">Email</label>
                    <input
                      type="email"
                      class="input"
                      value={profile().email}
                      onInput={(e) => updateField("email", e.currentTarget.value)}
                      placeholder="jane.doe@agency.gov"
                    />
                  </div>
                  <div>
                    <label class="label">Phone</label>
                    <input
                      type="tel"
                      class="input"
                      value={profile().phone}
                      onInput={(e) => updateField("phone", e.currentTarget.value)}
                      placeholder="(555) 123-4567"
                    />
                  </div>
                </div>

                {/* Certifications */}
                <div class="text-xs font-semibold text-txt-muted uppercase tracking-wide mt-2">Certifications & Branding</div>
                <div class="flex flex-col gap-3">
                  <div>
                    <label class="label">Certifications</label>
                    <input
                      type="text"
                      class="input"
                      value={(profile().certifications || []).join(", ")}
                      onInput={(e) =>
                        updateField(
                          "certifications",
                          e.currentTarget.value.split(",").map(s => s.trim()).filter(Boolean),
                        )
                      }
                      placeholder="EnCE, GCFE, CFCE"
                    />
                    <span class="text-[10px] text-txt-muted">Comma-separated</span>
                  </div>
                  <div>
                    <label class="label">Case Number Prefix</label>
                    <input
                      type="text"
                      class="input"
                      value={profile().caseNumberPrefix}
                      onInput={(e) => updateField("caseNumberPrefix", e.currentTarget.value)}
                      placeholder="CASE-"
                    />
                  </div>
                  <div>
                    <label class="label">Report Logo</label>
                    <div class="flex items-center gap-2">
                      <input
                        type="text"
                        class="input flex-1"
                        value={profile().logoPath}
                        onInput={(e) => updateField("logoPath", e.currentTarget.value)}
                        placeholder="No logo set"
                      />
                      <button class="btn-sm" onClick={handleBrowseLogo}>Browse</button>
                    </div>
                  </div>
                </div>
              </div>

              {/* Footer */}
              <div class="modal-footer justify-end shrink-0">
                <button class="btn btn-secondary" onClick={handleCancel}>Cancel</button>
                <button
                  class="btn btn-primary"
                  disabled={!profile().name.trim()}
                  onClick={handleSave}
                >
                  {isNew() ? "Add Profile" : "Save Changes"}
                </button>
              </div>
            </div>
          </div>
        )}
      </Show>
    </>
  );
};
