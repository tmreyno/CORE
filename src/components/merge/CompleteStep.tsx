// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * CompleteStep — Step 4 of MergeProjectsWizard: show merge results.
 */

import { Component, For, Show } from "solid-js";
import { HiOutlineCheckCircle } from "../icons";
import type { MergeResult } from "./types";

export interface CompleteStepProps {
  mergeResult: MergeResult | null;
}

export const CompleteStep: Component<CompleteStepProps> = (props) => {
  return (
    <div class="col gap-4">
      <div class="flex flex-col items-center py-6 gap-3">
        <HiOutlineCheckCircle class="w-12 h-12 text-success" />
        <h3 class="text-lg font-semibold text-txt">Merge Complete!</h3>
      </div>

      <Show when={props.mergeResult}>
        {(result) => (
          <div class="col gap-3">
            <div class="p-3 rounded-lg bg-bg-secondary border border-border">
              <div class="text-sm text-txt-secondary mb-1">Merged project:</div>
              <div class="text-sm font-medium text-txt truncate">{result().cffxPath}</div>
            </div>

            <Show when={result().stats}>
              {(stats) => (
                <div class="p-3 rounded-lg bg-bg-panel border border-border">
                  <h4 class="text-sm font-semibold text-txt mb-2">Merge Statistics</h4>
                  <div class="grid grid-cols-3 gap-2 text-xs text-txt-secondary">
                    <span>Users: {stats().usersMerged}</span>
                    <span>Sessions: {stats().sessionsMerged}</span>
                    <span>Activity: {stats().activityEntriesMerged}</span>
                    <span>Evidence: {stats().evidenceFilesMerged}</span>
                    <span>Hashes: {stats().hashesMerged}</span>
                    <span>Bookmarks: {stats().bookmarksMerged}</span>
                    <span>Notes: {stats().notesMerged}</span>
                    <span>Reports: {stats().reportsMerged}</span>
                    <span>Tags: {stats().tagsMerged}</span>
                    <span>Searches: {stats().searchesMerged}</span>
                    <span>DB Tables: {stats().ffxdbTablesMerged}</span>
                  </div>
                </div>
              )}
            </Show>

            {/* Source provenance */}
            <Show when={result().sources && result().sources!.length > 0}>
              <div class="p-3 rounded-lg bg-bg-secondary border border-border">
                <h4 class="text-sm font-semibold text-txt mb-2">Source Projects</h4>
                <div class="col gap-2">
                  <For each={result().sources!}>
                    {(source) => (
                      <div class="text-xs text-txt-secondary flex items-center justify-between">
                        <div>
                          <span class="font-medium text-txt">{source.sourceProjectName}</span>
                          <Show when={source.ownerName}>
                            <span class="text-txt-muted"> — {source.ownerName}</span>
                          </Show>
                        </div>
                        <span class="text-txt-muted">{source.evidenceFileCount} evidence</span>
                      </div>
                    )}
                  </For>
                </div>
              </div>
            </Show>
          </div>
        )}
      </Show>
    </div>
  );
};
