// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * BackupsTab — displays version backups list and creation controls.
 */

import { Show, For } from "solid-js";
import type { Accessor, Setter } from "solid-js";
import { HiOutlineClock } from "../../icons";
import { getBasename } from "../../../utils/pathUtils";
import { formatFileSize } from "./types";
import type { BackupFile } from "../../../hooks/useProjectRecovery";

export interface BackupsTabProps {
  loading: Accessor<boolean>;
  versions: Accessor<BackupFile[]>;
  selectedBackup: Accessor<BackupFile | null>;
  setSelectedBackup: Setter<BackupFile | null>;
  onCreateBackup: () => void;
  onCreateVersion: () => void;
}

export function BackupsTab(props: BackupsTabProps) {
  return (
    <div class="space-y-4">
      {/* Actions */}
      <div class="flex gap-2">
        <button
          onClick={props.onCreateBackup}
          disabled={props.loading()}
          class="btn-sm-primary"
        >
          Create Backup
        </button>
        <button
          onClick={props.onCreateVersion}
          disabled={props.loading()}
          class="btn-sm"
        >
          Create Version Snapshot
        </button>
      </div>

      {/* Empty state */}
      <Show when={props.versions().length === 0}>
        <div class="text-center py-12 text-txt-muted">
          <HiOutlineClock class="w-12 h-12 mx-auto mb-4 opacity-50" />
          <p>No version backups found</p>
        </div>
      </Show>

      {/* Backup List */}
      <div class="space-y-2">
        <For each={props.versions()}>
          {(backup) => (
            <div
              class={`p-4 rounded-md border cursor-pointer transition-colors ${
                props.selectedBackup()?.path === backup.path
                  ? "border-accent bg-bg-active"
                  : "border-border hover:border-accent/50 hover:bg-bg-hover"
              }`}
              onClick={() => props.setSelectedBackup(backup)}
            >
              <div class="flex items-start justify-between">
                <div class="flex-1">
                  <div class="flex items-center gap-2 mb-1">
                    <span class="font-medium text-txt">
                      {getBasename(backup.path) || "Backup"}
                    </span>
                    <span
                      class={`text-xs px-2 py-0.5 rounded ${
                        backup.metadata.backup_type === "ManualBackup"
                          ? "bg-accent/20 text-accent"
                          : "bg-bg-secondary text-txt-secondary"
                      }`}
                    >
                      {backup.metadata.backup_type}
                    </span>
                  </div>
                  <p class="text-sm text-txt-secondary">
                    {new Date(backup.metadata.created_at).toLocaleString()}
                  </p>
                  <p class="text-xs text-txt-muted mt-1">
                    {formatFileSize(backup.metadata.file_size)} • v
                    {backup.metadata.app_version}
                  </p>
                </div>
              </div>
            </div>
          )}
        </For>
      </div>
    </div>
  );
}
