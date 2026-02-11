// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show, For, Component, Accessor } from "solid-js";
import { HiOutlineDocumentDuplicate } from "../icons";
import type { HashHistoryEntry } from "../../types";

interface HashHistoryProps {
  hashHistory: HashHistoryEntry[];
  reversedHashHistory: Accessor<HashHistoryEntry[]>;
}

export const HashHistory: Component<HashHistoryProps> = (props) => {
  return (
    <Show when={props.hashHistory.length > 0}>
      <div class="info-card">
        <div class="flex items-center justify-between mb-2">
          <span class="info-card-title">🕒 Hash History ({props.hashHistory.length})</span>
        </div>
        <div class="flex flex-col gap-1 max-h-40 overflow-y-auto">
          <For each={props.reversedHashHistory()}>
            {(entry) => {
              const isStored = entry.source === "stored";
              const isVerified = entry.source === "verified";
              const entryDate = entry.timestamp instanceof Date ? entry.timestamp : new Date(entry.timestamp);
              // Check if this is the epoch fallback (no real date available)
              const hasValidDate = entryDate.getTime() > 0;
              
              // Format short date (e.g., "Jan 16, 2026")
              const shortDateStr = hasValidDate 
                ? entryDate.toLocaleDateString(undefined, { month: "short", day: "numeric", year: "numeric" })
                : "Container";
              
              // Build detailed tooltip
              const tooltipParts = [
                `Algorithm: ${entry.algorithm}`,
                `Hash: ${entry.hash}`,
                `Source: ${isStored ? "Stored in container" : isVerified ? "Verified copy" : "Computed"}`,
              ];
              if (hasValidDate) {
                tooltipParts.push(`Date: ${entryDate.toLocaleString()}`);
              }
              if (entry.verified === true) {
                tooltipParts.push("Status: Verified ✓");
              } else if (entry.verified === false) {
                tooltipParts.push("Status: MISMATCH ✗");
              }
              if (entry.verified_against) {
                tooltipParts.push(`Compared against: ${entry.verified_against.substring(0, 16)}...`);
              }
              const tooltipText = tooltipParts.join("\n");
              
              return (
                <div 
                  class={`hash-row ${
                    entry.verified === true 
                      ? 'hash-row-verified' 
                      : entry.verified === false 
                        ? 'hash-row-failed' 
                        : isStored 
                          ? 'hash-row-stored' 
                          : 'hash-row-neutral'
                  }`}
                  title={tooltipText}
                >
                  <span class="text-txt-muted w-24 shrink-0 text-xs cursor-help">
                    {isStored ? '◆ ' : isVerified ? '⟳ ' : ''}{shortDateStr}
                  </span>
                  <span class="text-accent w-12 shrink-0 uppercase">{entry.algorithm}</span>
                  <span class={`w-16 shrink-0 text-xs ${
                    isStored 
                      ? 'text-amber-400' 
                      : isVerified 
                        ? 'text-green-400' 
                        : 'text-txt-muted'
                  }`}>
                    {isStored ? 'stored' : isVerified ? 'verified' : entry.source}
                  </span>
                  <code class="text-txt-secondary font-mono truncate flex-1">{entry.hash}</code>
                  <Show when={entry.verified === true}>
                    <span class="relative inline-flex text-green-400" title="Verified match">
                      <span>✓</span>
                      <span class="absolute left-[3px]">✓</span>
                    </span>
                  </Show>
                  <Show when={entry.verified === false}>
                    <span class="text-red-400 font-bold" title="Hash mismatch!">✗</span>
                  </Show>
                  <button 
                    class="text-txt-muted hover:text-txt-tertiary p-0.5 flex items-center" 
                    onClick={() => navigator.clipboard.writeText(entry.hash)} 
                    title="Copy hash"
                  >
                    <HiOutlineDocumentDuplicate class="w-3 h-3" />
                  </button>
                </div>
              );
            }}
          </For>
        </div>
      </div>
    </Show>
  );
};
