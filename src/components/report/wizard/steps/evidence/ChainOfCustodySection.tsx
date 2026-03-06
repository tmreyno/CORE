// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ChainOfCustodySection — COC sub-section within the Evidence wizard step.
 */

import { Show, For } from "solid-js";
import { HiOutlineXMark } from "../../../../icons";
import { useWizard } from "../../WizardContext";

export function ChainOfCustodySection() {
  const ctx = useWizard();

  return (
    <div class="mt-6 pt-5 border-t border-border/30">
      <div class="flex items-center justify-between mb-4">
        <div class="flex items-center gap-2">
          <span class="text-lg">🔗</span>
          <h4 class="text-sm font-semibold">Chain of Custody</h4>
          <span class="text-xs px-2 py-0.5 bg-accent/10 text-accent rounded-full font-medium">
            {ctx.chainOfCustody().length} records
          </span>
        </div>
        <button
          type="button"
          class="px-3 py-1.5 rounded-lg text-sm font-medium bg-accent/10 text-accent hover:bg-accent/20 transition-colors"
          onClick={ctx.addCustodyRecord}
        >
          + Add Record
        </button>
      </div>

      <Show when={ctx.chainOfCustody().length === 0}>
        <div class="text-center py-8 bg-surface/30 rounded-xl border-2 border-dashed border-border/30">
          <div class="w-12 h-12 mx-auto mb-3 rounded-xl bg-accent/10 flex items-center justify-center">
            <span class="text-2xl">📋</span>
          </div>
          <p class="text-sm font-medium text-txt/70">No chain of custody records</p>
          <p class="text-xs text-txt/50 mt-1">Add records to document evidence handling</p>
        </div>
      </Show>

      <div class="space-y-3">
        <For each={ctx.chainOfCustody()}>
          {(record, index) => (
            <div class="p-4 bg-surface/50 border border-border/30 rounded-xl">
              <div class="grid grid-cols-4 gap-3">
                <div>
                  <label class="block text-xs text-txt/50 mb-1.5">Date/Time</label>
                  <input
                    type="datetime-local"
                    class="w-full px-2.5 py-2 bg-bg-panel border border-border/50 rounded-lg text-sm focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/20 transition-all"
                    value={record.timestamp.slice(0, 16)}
                    onInput={(e) => ctx.updateCustodyRecord(index(), { timestamp: new Date(e.currentTarget.value).toISOString() })}
                  />
                </div>
                <div>
                  <label class="block text-xs text-txt/50 mb-1.5">Action</label>
                  <select
                    class="w-full px-2.5 py-2 bg-bg-panel border border-border/50 rounded-lg text-sm focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/20 transition-all"
                    value={record.action}
                    onChange={(e) => ctx.updateCustodyRecord(index(), { action: e.currentTarget.value })}
                  >
                    <option value="Received">Received</option>
                    <option value="Transferred">Transferred</option>
                    <option value="Imaged">Imaged</option>
                    <option value="Analyzed">Analyzed</option>
                    <option value="Stored">Stored</option>
                    <option value="Released">Released</option>
                    <option value="Returned">Returned</option>
                    <option value="Destroyed">Destroyed</option>
                  </select>
                </div>
                <div>
                  <label class="block text-xs text-txt/50 mb-1.5">Handler</label>
                  <input
                    type="text"
                    class="w-full px-2.5 py-2 bg-bg-panel border border-border/50 rounded-lg text-sm focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/20 transition-all"
                    value={record.handler}
                    onInput={(e) => ctx.updateCustodyRecord(index(), { handler: e.currentTarget.value })}
                    placeholder="Name of handler"
                  />
                </div>
                <div>
                  <label class="block text-xs text-txt/50 mb-1.5">Location</label>
                  <input
                    type="text"
                    class="w-full px-2.5 py-2 bg-bg-panel border border-border/50 rounded-lg text-sm focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/20 transition-all"
                    value={record.location || ""}
                    onInput={(e) => ctx.updateCustodyRecord(index(), { location: e.currentTarget.value || undefined })}
                    placeholder="Storage location"
                  />
                </div>
              </div>
              <div class="mt-3 flex gap-2">
                <input
                  type="text"
                  class="flex-1 px-2.5 py-2 bg-bg-panel border border-border/50 rounded-lg text-sm focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/20 transition-all"
                  value={record.notes || ""}
                  onInput={(e) => ctx.updateCustodyRecord(index(), { notes: e.currentTarget.value || undefined })}
                  placeholder="Additional notes..."
                />
                <button
                  type="button"
                  class="p-2 text-error/70 hover:text-error hover:bg-error/10 rounded-lg transition-colors"
                  onClick={() => ctx.removeCustodyRecord(index())}
                  title="Remove custody record"
                >
                  <HiOutlineXMark class="w-4 h-4" />
                </button>
              </div>
            </div>
          )}
        </For>
      </div>
    </div>
  );
}
