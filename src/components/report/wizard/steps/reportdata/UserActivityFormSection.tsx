// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * UserActivityFormSection - Document user activity analysis
 * including timeline entries, categories, and significance ratings.
 */

import { For, Show } from "solid-js";
import { HiOutlinePlus, HiOutlineTrash } from "../../../../icons";
import { useWizard } from "../../WizardContext";
import { USER_ACTIVITY_CATEGORIES } from "../../../constants";
import type { UserActivityData, UserActivityEntry, Severity } from "../../../types";

const SEVERITY_OPTIONS: { value: Severity; label: string; color: string }[] = [
  { value: "Informational", label: "Informational", color: "text-info" },
  { value: "Low", label: "Low", color: "text-txt-muted" },
  { value: "Medium", label: "Medium", color: "text-warning" },
  { value: "High", label: "High", color: "text-error" },
  { value: "Critical", label: "Critical", color: "text-error font-bold" },
];

export function UserActivityFormSection() {
  const ctx = useWizard();

  const data = (): UserActivityData => ctx.userActivityData();
  const update = (patch: Partial<UserActivityData>) => {
    ctx.setUserActivityData({ ...data(), ...patch });
  };

  const addEntry = () => {
    const entry: UserActivityEntry = {
      id: crypto.randomUUID(),
      timestamp: new Date().toISOString().slice(0, 16),
      category: "general",
      description: "",
      source_artifact: "",
      significance: "Informational" as Severity,
      notes: "",
    };
    update({ activity_entries: [...data().activity_entries, entry] });
  };

  const updateEntry = (id: string, field: keyof UserActivityEntry, value: unknown) => {
    update({
      activity_entries: data().activity_entries.map((e) =>
        e.id === id ? { ...e, [field]: value } : e
      ),
    });
  };

  const removeEntry = (id: string) => {
    update({
      activity_entries: data().activity_entries.filter((e) => e.id !== id),
    });
  };

  return (
    <div class="space-y-6">
      {/* Header */}
      <div class="flex items-center gap-3">
        <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-accent/20 to-accent/5 flex items-center justify-center">
          <span class="text-xl">👤</span>
        </div>
        <div>
          <h3 class="text-base font-semibold">User Activity Report</h3>
          <p class="text-sm text-txt/60">
            Document and categorize user activity found during analysis
          </p>
        </div>
      </div>

      {/* Target user info */}
      <div class="border border-border/50 rounded-xl p-4 space-y-4">
        <h4 class="font-medium text-sm">Target User Information</h4>
        <div class="grid grid-cols-2 gap-3">
          <div class="form-group">
            <label class="label">Target User</label>
            <input
              class="input-sm"
              value={data().target_user}
              placeholder="Primary username or account"
              onInput={(e) => update({ target_user: e.currentTarget.value })}
            />
          </div>
          <div class="form-group">
            <label class="label">Known Aliases (comma-separated)</label>
            <input
              class="input-sm"
              value={(data().user_aliases ?? []).join(", ")}
              placeholder="Other usernames, emails, screen names"
              onInput={(e) =>
                update({
                  user_aliases: e.currentTarget.value.split(",").map((a) => a.trim()).filter(Boolean),
                })
              }
            />
          </div>
        </div>
        <div class="grid grid-cols-2 gap-3">
          <div class="form-group">
            <label class="label">Time Range Start</label>
            <input
              type="datetime-local"
              class="input-sm"
              value={data().time_range_start || ""}
              onInput={(e) => update({ time_range_start: e.currentTarget.value })}
            />
          </div>
          <div class="form-group">
            <label class="label">Time Range End</label>
            <input
              type="datetime-local"
              class="input-sm"
              value={data().time_range_end || ""}
              onInput={(e) => update({ time_range_end: e.currentTarget.value })}
            />
          </div>
        </div>
        <div class="form-group">
          <label class="label">Summary</label>
          <textarea
            class="textarea text-sm"
            rows={3}
            value={data().summary || ""}
            placeholder="High-level summary of the user's activity during the analyzed timeframe..."
            onInput={(e) => update({ summary: e.currentTarget.value })}
          />
        </div>
      </div>

      {/* Activity entries */}
      <div class="space-y-3">
        <div class="flex items-center justify-between">
          <h4 class="font-medium text-sm">
            Activity Entries
            <Show when={data().activity_entries.length > 0}>
              <span class="text-txt-muted font-normal ml-2">({data().activity_entries.length})</span>
            </Show>
          </h4>
          <button type="button" class="btn-action-primary" onClick={addEntry}>
            <HiOutlinePlus class="w-4 h-4" />
            Add Entry
          </button>
        </div>

        <Show
          when={data().activity_entries.length > 0}
          fallback={
            <div class="text-center py-10 text-txt/50 border border-dashed border-border rounded-xl">
              <p class="text-lg mb-1">No activity entries yet</p>
              <p class="text-sm">Click "Add Entry" to document each user activity finding.</p>
            </div>
          }
        >
          <div class="space-y-3">
            <For each={data().activity_entries}>
              {(entry) => (
                <div class="border border-border/50 rounded-xl p-4 bg-surface/50 space-y-3">
                  <div class="flex items-center justify-between">
                    <span class="text-xs text-txt-muted font-mono">{entry.id.slice(0, 8)}</span>
                    <button type="button" class="icon-btn-sm text-txt-muted hover:text-error" onClick={() => removeEntry(entry.id)}>
                      <HiOutlineTrash class="w-4 h-4" />
                    </button>
                  </div>
                  <div class="grid grid-cols-3 gap-3">
                    <div class="form-group">
                      <label class="label">Timestamp</label>
                      <input
                        type="datetime-local"
                        class="input-sm"
                        value={entry.timestamp}
                        onInput={(e) => updateEntry(entry.id, "timestamp", e.currentTarget.value)}
                      />
                    </div>
                    <div class="form-group">
                      <label class="label">Category</label>
                      <select class="input-sm" value={entry.category} onChange={(e) => updateEntry(entry.id, "category", e.currentTarget.value)}>
                        <For each={USER_ACTIVITY_CATEGORIES}>
                          {(cat) => <option value={cat.value}>{cat.label}</option>}
                        </For>
                      </select>
                    </div>
                    <div class="form-group">
                      <label class="label">Significance</label>
                      <select class="input-sm" value={entry.significance} onChange={(e) => updateEntry(entry.id, "significance", e.currentTarget.value)}>
                        <For each={SEVERITY_OPTIONS}>
                          {(opt) => <option value={opt.value}>{opt.label}</option>}
                        </For>
                      </select>
                    </div>
                  </div>
                  <div class="form-group">
                    <label class="label">Description</label>
                    <input class="input-sm" value={entry.description} placeholder="What did the user do?" onInput={(e) => updateEntry(entry.id, "description", e.currentTarget.value)} />
                  </div>
                  <div class="grid grid-cols-2 gap-3">
                    <div class="form-group">
                      <label class="label">Source Artifact</label>
                      <input class="input-sm" value={entry.source_artifact} placeholder="e.g., Chrome History, Registry, LNK file" onInput={(e) => updateEntry(entry.id, "source_artifact", e.currentTarget.value)} />
                    </div>
                    <div class="form-group">
                      <label class="label">Application</label>
                      <input class="input-sm" value={entry.application || ""} placeholder="e.g., Chrome, Firefox, Outlook" onInput={(e) => updateEntry(entry.id, "application", e.currentTarget.value)} />
                    </div>
                  </div>
                  <div class="grid grid-cols-2 gap-3">
                    <div class="form-group">
                      <label class="label">Evidence Reference</label>
                      <input class="input-sm" value={entry.evidence_ref || ""} placeholder="File or evidence container reference" onInput={(e) => updateEntry(entry.id, "evidence_ref", e.currentTarget.value)} />
                    </div>
                    <div class="form-group">
                      <label class="label">User Account</label>
                      <input class="input-sm" value={entry.user_account || ""} placeholder="Account used for this activity" onInput={(e) => updateEntry(entry.id, "user_account", e.currentTarget.value)} />
                    </div>
                  </div>
                  <div class="form-group">
                    <label class="label">Notes</label>
                    <textarea class="textarea text-sm" rows={2} value={entry.notes || ""} onInput={(e) => updateEntry(entry.id, "notes", e.currentTarget.value)} />
                  </div>
                </div>
              )}
            </For>
          </div>
        </Show>
      </div>
    </div>
  );
}
