// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * EvidenceStep - Third wizard step for evidence selection
 */

import { For, Show } from "solid-js";
import { formatBytes } from "../../../../utils";
import {
  HiOutlineCircleStack,
  HiOutlineXMark,
  HiOutlineServer,
  HiOutlineCalendarDays,
  HiOutlineCheckCircle,
} from "../../../icons";
import { useWizard } from "../WizardContext";
import { getDisplayName, getDisplaySize, getAcquisitionDate } from "../utils/evidenceUtils";

export function EvidenceStep() {
  const ctx = useWizard();

  return (
    <div class="space-y-5">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <HiOutlineCircleStack class="w-5 h-5 text-accent" />
          <h3 class="text-base font-semibold">Evidence Items</h3>
        </div>
        <div class="flex items-center gap-3">
          <span class="text-sm text-text/60">
            <span class="font-medium text-accent">{ctx.selectedEvidence().size}</span> of {ctx.groupedEvidence().length} selected
          </span>
          <Show when={ctx.groupedEvidence().length > 0}>
            <button
              class="text-xs text-accent hover:underline"
              onClick={() => {
                if (ctx.selectedEvidence().size === ctx.groupedEvidence().length) {
                  ctx.setSelectedEvidence(new Set<string>());
                } else {
                  ctx.setSelectedEvidence(new Set<string>(ctx.groupedEvidence().map(g => g.primaryFile.path)));
                }
              }}
            >
              {ctx.selectedEvidence().size === ctx.groupedEvidence().length ? 'Deselect All' : 'Select All'}
            </button>
          </Show>
        </div>
      </div>

      <Show when={ctx.groupedEvidence().length === 0}>
        <div class="text-center py-12 bg-surface/30 rounded-xl border border-border/30">
          <div class="w-16 h-16 mx-auto mb-4 rounded-2xl bg-accent/10 flex items-center justify-center">
            <span class="text-3xl">📂</span>
          </div>
          <p class="font-medium text-text/80">No evidence files discovered</p>
          <p class="text-sm text-text/50 mt-1">Scan a directory first to discover forensic images</p>
        </div>
      </Show>

      <div class="space-y-2 max-h-[300px] overflow-y-auto pr-1">
        <For each={ctx.groupedEvidence()}>
          {(group) => {
            const file = group.primaryFile;
            const info = () => ctx.props.fileInfoMap.get(file.path);
            const hashInfo = () => ctx.props.fileHashMap.get(file.path);
            const isSelected = () => ctx.selectedEvidence().has(file.path);

            // Extract display info from container - use group's total size for segments
            const displayInfo = () => {
              const i = info();
              const totalSize = getDisplaySize(group, i);
              const acqDate = getAcquisitionDate(i);
              return { totalSize, acqDate };
            };

            // Get base name without segment extension for multi-segment containers
            const displayName = () => getDisplayName(group);

            return (
              <div
                class={`flex items-start gap-3 p-3.5 rounded-xl border-2 cursor-pointer transition-all duration-200 ${
                  isSelected()
                    ? 'border-accent bg-accent/5 shadow-sm shadow-accent/10'
                    : 'border-border/30 bg-surface/30 hover:border-accent/30 hover:bg-surface/50'
                }`}
                onClick={() => ctx.toggleEvidence(file.path)}
              >
                <div class={`w-5 h-5 rounded-md border-2 flex items-center justify-center mt-0.5 transition-colors ${
                  isSelected() ? 'bg-accent border-accent' : 'border-border/50'
                }`}>
                  <Show when={isSelected()}>
                    <span class="text-white text-xs font-bold">✓</span>
                  </Show>
                </div>
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2 flex-wrap">
                    <span class="font-medium text-sm truncate">{displayName()}</span>
                    <span class="text-xs px-2 py-0.5 bg-accent/10 text-accent rounded-full font-medium">
                      {file.container_type}
                    </span>
                    <Show when={group.segmentCount > 1}>
                      <span class="text-xs px-2 py-0.5 bg-blue-500/10 text-blue-400 rounded-full font-medium">
                        {group.segmentCount} segments
                      </span>
                    </Show>
                  </div>
                  <div class="text-xs text-text/50 truncate mt-0.5">
                    {file.path.substring(0, file.path.lastIndexOf('/'))}
                  </div>
                  <div class="flex items-center gap-3 mt-2 flex-wrap">
                    <Show when={displayInfo()?.totalSize}>
                      <span class="text-xs text-text/60 flex items-center gap-1">
                        <HiOutlineServer class="w-3 h-3" /> {formatBytes(displayInfo()!.totalSize!)}
                        <Show when={group.segmentCount > 1}>
                          <span class="text-text/40">(total)</span>
                        </Show>
                      </span>
                    </Show>
                    <Show when={displayInfo()?.acqDate}>
                      <span class="text-xs text-text/60 flex items-center gap-1">
                        <HiOutlineCalendarDays class="w-3 h-3" /> {displayInfo()!.acqDate}
                      </span>
                    </Show>
                    <Show when={hashInfo()}>
                      <span class={`text-xs font-mono flex items-center gap-1 ${
                        hashInfo()!.verified === true ? 'text-success' :
                        hashInfo()!.verified === false ? 'text-error' : 'text-text/60'
                      }`}>
                        <HiOutlineCheckCircle class="w-3 h-3" /> {hashInfo()!.algorithm}
                        {hashInfo()!.verified === true && " ✓"}
                        {hashInfo()!.verified === false && " ✗"}
                      </span>
                    </Show>
                  </div>
                </div>
              </div>
            );
          }}
        </For>
      </div>

      {/* Chain of Custody Section */}
      <Show when={ctx.enabledSections().chainOfCustody}>
        <ChainOfCustodySection />
      </Show>
    </div>
  );
}

/**
 * Chain of Custody sub-section
 */
function ChainOfCustodySection() {
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
          <p class="text-sm font-medium text-text/70">No chain of custody records</p>
          <p class="text-xs text-text/50 mt-1">Add records to document evidence handling</p>
        </div>
      </Show>

      <div class="space-y-3">
        <For each={ctx.chainOfCustody()}>
          {(record, index) => (
            <div class="p-4 bg-surface/50 border border-border/30 rounded-xl">
              <div class="grid grid-cols-4 gap-3">
                <div>
                  <label class="block text-xs text-text/50 mb-1.5">Date/Time</label>
                  <input
                    type="datetime-local"
                    class="w-full px-2.5 py-2 bg-bg-panel border border-border/50 rounded-lg text-sm focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/20 transition-all"
                    value={record.timestamp.slice(0, 16)}
                    onInput={(e) => ctx.updateCustodyRecord(index(), { timestamp: new Date(e.currentTarget.value).toISOString() })}
                  />
                </div>
                <div>
                  <label class="block text-xs text-text/50 mb-1.5">Action</label>
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
                  <label class="block text-xs text-text/50 mb-1.5">Handler</label>
                  <input
                    type="text"
                    class="w-full px-2.5 py-2 bg-bg-panel border border-border/50 rounded-lg text-sm focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/20 transition-all"
                    value={record.handler}
                    onInput={(e) => ctx.updateCustodyRecord(index(), { handler: e.currentTarget.value })}
                    placeholder="Name of handler"
                  />
                </div>
                <div>
                  <label class="block text-xs text-text/50 mb-1.5">Location</label>
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
