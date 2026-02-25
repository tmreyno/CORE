// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * TimelineFormSection - Build a chronological timeline report
 * from project activity data and manual event entries.
 */

import { createMemo, For, Show } from "solid-js";
import { HiOutlinePlus, HiOutlineTrash, HiOutlineStar } from "../../../../icons";
import { useWizard } from "../../WizardContext";
import type { TimelineReportData, TimelineEvent } from "../../../types";

/** Default categories for timeline events */
const TIMELINE_CATEGORIES = [
  { value: "acquisition", label: "Evidence Acquisition" },
  { value: "processing", label: "Processing / Imaging" },
  { value: "analysis", label: "Analysis" },
  { value: "communication", label: "Communication" },
  { value: "file_activity", label: "File Activity" },
  { value: "web_activity", label: "Web / Internet Activity" },
  { value: "login_logout", label: "Login / Logout" },
  { value: "application", label: "Application Usage" },
  { value: "email", label: "Email Activity" },
  { value: "usb_device", label: "USB / Device Connection" },
  { value: "system_event", label: "System Event" },
  { value: "user_action", label: "User Action" },
  { value: "other", label: "Other" },
];

export function TimelineFormSection() {
  const ctx = useWizard();

  const data = (): TimelineReportData => ctx.timelineReportData();
  const update = (patch: Partial<TimelineReportData>) => {
    ctx.setTimelineReportData({ ...data(), ...patch });
  };

  const keyEventSet = createMemo(() => new Set(data().key_events));

  const addEvent = () => {
    const event: TimelineEvent = {
      timestamp: new Date().toISOString().slice(0, 16),
      event_type: "analysis",
      description: "",
      source: "",
    };
    update({ events: [...data().events, event] });
  };

  const updateEvent = (index: number, field: keyof TimelineEvent, value: string) => {
    const updated = [...data().events];
    updated[index] = { ...updated[index], [field]: value };
    update({ events: updated });
  };

  const removeEvent = (index: number) => {
    const updated = [...data().events];
    updated.splice(index, 1);
    update({ events: updated });
  };

  /** Use timestamp+description as a pseudo-ID for key event marking */
  const eventKey = (ev: TimelineEvent) => `${ev.timestamp}|${ev.description}`;

  const toggleKeyEvent = (ev: TimelineEvent) => {
    const key = eventKey(ev);
    const current = new Set(data().key_events);
    if (current.has(key)) {
      current.delete(key);
    } else {
      current.add(key);
    }
    update({ key_events: [...current] });
  };

  const toggleCategory = (cat: string) => {
    const current = new Set(data().included_categories);
    if (current.has(cat)) {
      current.delete(cat);
    } else {
      current.add(cat);
    }
    update({ included_categories: [...current] });
  };

  const sortedEvents = createMemo(() =>
    [...data().events].sort((a, b) => a.timestamp.localeCompare(b.timestamp))
  );

  return (
    <div class="space-y-6">
      {/* Header */}
      <div class="flex items-center gap-3">
        <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-accent/20 to-accent/5 flex items-center justify-center">
          <span class="text-xl">📅</span>
        </div>
        <div>
          <h3 class="text-base font-semibold">Timeline Report</h3>
          <p class="text-sm text-txt/60">
            Build a chronological timeline of events from the investigation
          </p>
        </div>
      </div>

      {/* Time range & narrative */}
      <div class="border border-border/50 rounded-xl p-4 space-y-4">
        <h4 class="font-medium text-sm">Timeline Parameters</h4>
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
          <label class="label">Narrative</label>
          <textarea
            class="textarea text-sm"
            rows={3}
            value={data().narrative || ""}
            placeholder="Narrative text connecting and contextualizing the timeline events..."
            onInput={(e) => update({ narrative: e.currentTarget.value })}
          />
        </div>
      </div>

      {/* Category filter */}
      <div class="border border-border/50 rounded-xl p-4 space-y-3">
        <h4 class="font-medium text-sm">Included Categories</h4>
        <p class="text-xs text-txt-muted">Select which categories of events to include in the report</p>
        <div class="flex flex-wrap gap-2">
          <For each={TIMELINE_CATEGORIES}>
            {(cat) => {
              const active = () => data().included_categories.includes(cat.value);
              return (
                <button
                  type="button"
                  class="chip"
                  classList={{
                    "chip-cyan": active(),
                    "chip-neutral": !active(),
                  }}
                  onClick={() => toggleCategory(cat.value)}
                >
                  {cat.label}
                </button>
              );
            }}
          </For>
        </div>
      </div>

      {/* Timeline events */}
      <div class="space-y-3">
        <div class="flex items-center justify-between">
          <h4 class="font-medium text-sm">
            Events
            <Show when={data().events.length > 0}>
              <span class="text-txt-muted font-normal ml-2">({data().events.length})</span>
            </Show>
          </h4>
          <button type="button" class="btn-action-primary" onClick={addEvent}>
            <HiOutlinePlus class="w-4 h-4" />
            Add Event
          </button>
        </div>

        <Show
          when={data().events.length > 0}
          fallback={
            <div class="text-center py-10 text-txt/50 border border-dashed border-border rounded-xl">
              <p class="text-lg mb-1">No timeline events</p>
              <p class="text-sm">
                Add events manually or they will be auto-populated from project activity data.
              </p>
            </div>
          }
        >
          <div class="space-y-3">
            <For each={sortedEvents()}>
              {(event) => {
                const idx = () => data().events.indexOf(event);
                const isKey = () => keyEventSet().has(eventKey(event));

                return (
                  <div class="border rounded-xl p-4 bg-surface/50 space-y-3" classList={{ "border-accent/50": isKey(), "border-border/50": !isKey() }}>
                    <div class="flex items-center justify-between">
                      <div class="flex items-center gap-2">
                        <button
                          type="button"
                          class="icon-btn-sm"
                          classList={{ "text-accent": isKey(), "text-txt-muted": !isKey() }}
                          title={isKey() ? "Remove from key events" : "Mark as key event"}
                          onClick={() => toggleKeyEvent(event)}
                        >
                          <HiOutlineStar class="w-4 h-4" />
                        </button>
                        <Show when={isKey()}>
                          <span class="badge badge-success text-[10px]">Key Event</span>
                        </Show>
                      </div>
                      <button type="button" class="icon-btn-sm text-txt-muted hover:text-error" onClick={() => removeEvent(idx())}>
                        <HiOutlineTrash class="w-4 h-4" />
                      </button>
                    </div>
                    <div class="grid grid-cols-3 gap-3">
                      <div class="form-group">
                        <label class="label">Timestamp</label>
                        <input
                          type="datetime-local"
                          class="input-sm"
                          value={event.timestamp}
                          onInput={(e) => updateEvent(idx(), "timestamp", e.currentTarget.value)}
                        />
                      </div>
                      <div class="form-group">
                        <label class="label">Event Type</label>
                        <select class="input-sm" value={event.event_type} onChange={(e) => updateEvent(idx(), "event_type", e.currentTarget.value)}>
                          <For each={TIMELINE_CATEGORIES}>{(cat) => <option value={cat.value}>{cat.label}</option>}</For>
                        </select>
                      </div>
                      <div class="form-group">
                        <label class="label">Source</label>
                        <input class="input-sm" value={event.source} placeholder="e.g., Chrome History, NTFS MFT" onInput={(e) => updateEvent(idx(), "source", e.currentTarget.value)} />
                      </div>
                    </div>
                    <div class="form-group">
                      <label class="label">Description</label>
                      <input class="input-sm" value={event.description} placeholder="What happened at this time" onInput={(e) => updateEvent(idx(), "description", e.currentTarget.value)} />
                    </div>
                    <div class="grid grid-cols-2 gap-3">
                      <div class="form-group">
                        <label class="label">Evidence Reference</label>
                        <input class="input-sm" value={event.evidence_ref || ""} placeholder="Evidence container or item" onInput={(e) => updateEvent(idx(), "evidence_ref", e.currentTarget.value)} />
                      </div>
                      <div class="form-group">
                        <label class="label">Artifact Path</label>
                        <input class="input-sm" value={event.artifact_path || ""} placeholder="Path within evidence" onInput={(e) => updateEvent(idx(), "artifact_path", e.currentTarget.value)} />
                      </div>
                    </div>
                  </div>
                );
              }}
            </For>
          </div>
        </Show>
      </div>
    </div>
  );
}
