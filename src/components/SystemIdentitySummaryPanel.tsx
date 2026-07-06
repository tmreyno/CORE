// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import {
  For,
  Show,
  createEffect,
  createMemo,
  createSignal,
  on,
  type Accessor,
} from "solid-js";
import { commands, type DbNormalizedArtifact } from "../api/commands";
import type { DiscoveredFile } from "../types";
import {
  artifactMatchesEvidence,
  buildSystemIdentitySummary,
  formatSystemIdentitySummaryForClipboard,
} from "../utils/systemIdentitySummary";
import { isTauri } from "../utils/platform";
import { logger } from "../utils/logger";
import { useToast } from "./Toast";
import {
  HiOutlineClipboardDocument,
  HiOutlineComputerDesktop,
} from "./icons";

const log = logger.scope("SystemIdentitySummary");

export interface SystemIdentitySummaryPanelProps {
  activeFile: Accessor<DiscoveredFile | null>;
  hasProject: Accessor<boolean>;
}

export function SystemIdentitySummaryPanel(props: SystemIdentitySummaryPanelProps) {
  const toast = useToast();
  const [records, setRecords] = createSignal<DbNormalizedArtifact[]>([]);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  const loadData = async () => {
    const activePath = props.activeFile()?.path ?? "";
    if (!props.hasProject() || !activePath || !isTauri) {
      setRecords([]);
      setError(null);
      setLoading(false);
      return;
    }

    setLoading(true);
    setError(null);
    try {
      const systemRecords = await commands.artifact.listByCategory("systeminfo", 1000);
      setRecords(systemRecords.filter((record) => artifactMatchesEvidence(record, activePath)));
    } catch (err) {
      log.warn("Failed to load system identity artifacts:", err);
      setRecords([]);
      setError("System identity data is unavailable.");
    } finally {
      setLoading(false);
    }
  };

  createEffect(on(() => props.hasProject(), loadData));
  createEffect(on(() => props.activeFile()?.path, loadData));

  const summary = createMemo(() => buildSystemIdentitySummary(records()));
  const hasData = createMemo(() => summary().recordCount > 0 && summary().groups.length > 0);

  const copySummary = async () => {
    if (!hasData()) return;
    try {
      await navigator.clipboard.writeText(formatSystemIdentitySummaryForClipboard(summary()));
      toast.success("System Identity Copied", "Extracted device and user information copied.");
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      toast.error("Copy Failed", message);
    }
  };

  return (
    <section class="border-b border-border bg-bg">
      <div class="flex items-center gap-1 border-b border-border bg-bg-secondary px-3 py-2">
        <HiOutlineComputerDesktop class="w-4 h-4 text-accent shrink-0" />
        <span class="text-xs font-medium text-txt flex-1">System Identity</span>
        <Show when={hasData()}>
          <button class="icon-btn-sm" title="Copy system identity" onClick={copySummary}>
            <HiOutlineClipboardDocument class="w-3.5 h-3.5" />
          </button>
        </Show>
      </div>

      <div class="max-h-96 overflow-auto p-3">
        <Show when={loading()}>
          <div class="text-xs text-txt-muted">Loading system identity...</div>
        </Show>

        <Show when={!loading() && error()}>
          <div class="text-xs text-error">{error()}</div>
        </Show>

        <Show when={!loading() && !error() && !hasData()}>
          <div class="text-xs text-txt-muted">No system identity artifacts found for this evidence.</div>
        </Show>

        <Show when={!loading() && hasData()}>
          <div class="mb-2 text-[11px] text-txt-muted">
            {summary().recordCount} record{summary().recordCount === 1 ? "" : "s"} from {summary().sourceCount} source{summary().sourceCount === 1 ? "" : "s"}
          </div>
          <div class="space-y-3">
            <For each={summary().groups}>
              {(group) => (
                <div>
                  <h3 class="mb-1 text-[11px] font-semibold uppercase tracking-wide text-txt-secondary">
                    {group.title}
                  </h3>
                  <div class="space-y-1.5">
                    <For each={group.fields}>
                      {(field) => (
                        <div class="grid grid-cols-[6.5rem_minmax(0,1fr)] gap-2 text-xs">
                          <div class="text-txt-muted">{field.label}</div>
                          <div class="break-words font-mono text-[11px] leading-relaxed text-txt">
                            {field.value}
                          </div>
                        </div>
                      )}
                    </For>
                  </div>
                </div>
              )}
            </For>
          </div>
        </Show>
      </div>
    </section>
  );
}
