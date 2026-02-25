// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * IARFormSection - Investigative Activity Report data entry.
 * Two sections: Summary (investigation overview) and Activity Entries.
 */

import { For, Show, createSignal } from "solid-js";
import {
  HiOutlinePlus,
  HiOutlineTrash,
  HiOutlineChevronDown,
  HiOutlineChevronUp,
} from "../../../../icons";
import { useWizard } from "../../WizardContext";
import { IAR_EVENT_CATEGORIES } from "../../../constants";
import type { IAREntry, IAREventCategory } from "../../../types";

export function IARFormSection() {
  const ctx = useWizard();
  const [showSummary, setShowSummary] = createSignal(true);
  const [expandedEntries, setExpandedEntries] = createSignal<Set<string>>(new Set());

  const toggleEntry = (id: string) => {
    setExpandedEntries((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const updateSummary = (field: string, value: unknown) => {
    ctx.setIarSummary({ ...ctx.iarSummary(), [field]: value });
  };

  const addEntry = () => {
    const entry: IAREntry = {
      id: crypto.randomUUID(),
      date: new Date().toISOString().slice(0, 10),
      category: "analysis",
      personnel: ctx.examiner().name || "",
      description: "",
      evidence_refs: [],
    };
    ctx.setIarEntries([...ctx.iarEntries(), entry]);
    setExpandedEntries((prev) => new Set(prev).add(entry.id));
  };

  const updateEntry = (id: string, field: keyof IAREntry, value: unknown) => {
    ctx.setIarEntries(
      ctx.iarEntries().map((e) => (e.id === id ? { ...e, [field]: value } : e))
    );
  };

  const removeEntry = (id: string) => {
    ctx.setIarEntries(ctx.iarEntries().filter((e) => e.id !== id));
  };

  const categoryLabel = (cat: string) =>
    IAR_EVENT_CATEGORIES.find((c) => c.value === cat)?.label ?? cat;

  const categoryIcon = (cat: string) =>
    IAR_EVENT_CATEGORIES.find((c) => c.value === cat)?.icon ?? "📝";

  return (
    <div class="space-y-6">
      {/* Header */}
      <div class="flex items-center gap-3">
        <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-accent/20 to-accent/5 flex items-center justify-center">
          <span class="text-xl">📋</span>
        </div>
        <div>
          <h3 class="text-base font-semibold">Investigative Activity Report</h3>
          <p class="text-sm text-txt/60">
            Document investigation activities, personnel, and milestones
          </p>
        </div>
      </div>

      {/* ── Summary Section ── */}
      <div class="border border-border/50 rounded-xl overflow-hidden">
        <button
          type="button"
          class="w-full flex items-center justify-between px-4 py-3 hover:bg-bg-hover/50 transition-colors"
          onClick={() => setShowSummary(!showSummary())}
        >
          <span class="font-medium text-sm">Investigation Summary</span>
          {showSummary() ? (
            <HiOutlineChevronUp class="w-4 h-4 text-txt-muted" />
          ) : (
            <HiOutlineChevronDown class="w-4 h-4 text-txt-muted" />
          )}
        </button>
        <Show when={showSummary()}>
          <div class="px-4 pb-4 border-t border-border/30 space-y-4 pt-3">
            <div class="grid grid-cols-2 gap-3">
              <div class="form-group">
                <label class="label">Investigation Start Date</label>
                <input
                  type="date"
                  class="input-sm"
                  value={ctx.iarSummary().investigation_start}
                  onInput={(e) => updateSummary("investigation_start", e.currentTarget.value)}
                />
              </div>
              <div class="form-group">
                <label class="label">Investigation End Date</label>
                <input
                  type="date"
                  class="input-sm"
                  value={ctx.iarSummary().investigation_end || ""}
                  onInput={(e) => updateSummary("investigation_end", e.currentTarget.value)}
                />
              </div>
            </div>
            <div class="grid grid-cols-2 gap-3">
              <div class="form-group">
                <label class="label">Lead Examiner</label>
                <input
                  class="input-sm"
                  value={ctx.iarSummary().lead_examiner}
                  onInput={(e) => updateSummary("lead_examiner", e.currentTarget.value)}
                />
              </div>
              <div class="form-group">
                <label class="label">Authorization (SW #, Court Order, etc.)</label>
                <input
                  class="input-sm"
                  value={ctx.iarSummary().authorization}
                  placeholder="e.g., SW-2024-12345"
                  onInput={(e) => updateSummary("authorization", e.currentTarget.value)}
                />
              </div>
            </div>
            <div class="form-group">
              <label class="label">Investigation Synopsis</label>
              <textarea
                class="textarea text-sm"
                rows={3}
                value={ctx.iarSummary().synopsis}
                placeholder="Brief synopsis of the investigation scope and objectives..."
                onInput={(e) => updateSummary("synopsis", e.currentTarget.value)}
              />
            </div>
          </div>
        </Show>
      </div>

      {/* ── Activity Entries ── */}
      <div class="space-y-3">
        <div class="flex items-center justify-between">
          <h4 class="font-medium text-sm">Activity Entries</h4>
          <button type="button" class="btn-action-primary" onClick={addEntry}>
            <HiOutlinePlus class="w-4 h-4" />
            Add Activity
          </button>
        </div>

        <Show
          when={ctx.iarEntries().length > 0}
          fallback={
            <div class="text-center py-10 text-txt/50 border border-dashed border-border rounded-xl">
              <p class="text-lg mb-1">No activities recorded</p>
              <p class="text-sm">Click "Add Activity" to begin documenting investigation events.</p>
            </div>
          }
        >
          <div class="space-y-2">
            <For each={ctx.iarEntries()}>
              {(entry) => {
                const isExpanded = () => expandedEntries().has(entry.id);
                return (
                  <div class="border border-border/50 rounded-xl bg-surface/50 overflow-hidden">
                    <div
                      class="flex items-center justify-between px-4 py-2.5 cursor-pointer hover:bg-bg-hover/50 transition-colors"
                      onClick={() => toggleEntry(entry.id)}
                    >
                      <div class="flex items-center gap-3">
                        {isExpanded() ? (
                          <HiOutlineChevronUp class="w-4 h-4 text-txt-muted" />
                        ) : (
                          <HiOutlineChevronDown class="w-4 h-4 text-txt-muted" />
                        )}
                        <span class="text-base">{categoryIcon(entry.category)}</span>
                        <span class="text-sm font-medium">{entry.date}</span>
                        <span class="text-sm text-txt/60">{categoryLabel(entry.category)}</span>
                        <Show when={(entry.hours_spent ?? 0) > 0}>
                          <span class="text-xs text-txt/40">({entry.hours_spent}h)</span>
                        </Show>
                      </div>
                      <button
                        type="button"
                        class="icon-btn-sm text-txt-muted hover:text-error"
                        onClick={(e) => { e.stopPropagation(); removeEntry(entry.id); }}
                      >
                        <HiOutlineTrash class="w-4 h-4" />
                      </button>
                    </div>

                    <Show when={isExpanded()}>
                      <div class="px-4 pb-4 pt-1 border-t border-border/30 space-y-3">
                        <div class="grid grid-cols-3 gap-3">
                          <div class="form-group">
                            <label class="label">Date</label>
                            <input type="date" class="input-sm" value={entry.date} onInput={(e) => updateEntry(entry.id, "date", e.currentTarget.value)} />
                          </div>
                          <div class="form-group">
                            <label class="label">Category</label>
                            <select class="input-sm" value={entry.category} onChange={(e) => updateEntry(entry.id, "category", e.currentTarget.value as IAREventCategory)}>
                              <For each={IAR_EVENT_CATEGORIES}>{(cat) => <option value={cat.value}>{cat.icon} {cat.label}</option>}</For>
                            </select>
                          </div>
                          <div class="form-group">
                            <label class="label">Hours Spent</label>
                            <input type="number" class="input-sm" min="0" step="0.5" value={entry.hours_spent ?? 0} onInput={(e) => updateEntry(entry.id, "hours_spent", parseFloat(e.currentTarget.value) || 0)} />
                          </div>
                        </div>
                        <div class="form-group">
                          <label class="label">Personnel</label>
                          <input class="input-sm" value={entry.personnel} onInput={(e) => updateEntry(entry.id, "personnel", e.currentTarget.value)} />
                        </div>
                        <div class="form-group">
                          <label class="label">Description</label>
                          <textarea class="textarea text-sm" rows={3} value={entry.description} placeholder="Describe the activity performed..." onInput={(e) => updateEntry(entry.id, "description", e.currentTarget.value)} />
                        </div>
                        <div class="grid grid-cols-2 gap-3">
                          <div class="form-group">
                            <label class="label">Keywords (comma-separated)</label>
                            <input class="input-sm" value={(entry.keywords || []).join(", ")} placeholder="e.g., CSAM, fraud" onInput={(e) => updateEntry(entry.id, "keywords", e.currentTarget.value.split(",").map((k) => k.trim()).filter(Boolean))} />
                          </div>
                          <div class="form-group">
                            <label class="label">Tools Used (comma-separated)</label>
                            <input class="input-sm" value={(entry.tools_used || []).join(", ")} placeholder="e.g., EnCase, FTK" onInput={(e) => updateEntry(entry.id, "tools_used", e.currentTarget.value.split(",").map((k) => k.trim()).filter(Boolean))} />
                          </div>
                        </div>
                        <div class="form-group">
                          <label class="label">Notes</label>
                          <textarea class="textarea text-sm" rows={2} value={entry.notes || ""} onInput={(e) => updateEntry(entry.id, "notes", e.currentTarget.value)} />
                        </div>
                      </div>
                    </Show>
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
