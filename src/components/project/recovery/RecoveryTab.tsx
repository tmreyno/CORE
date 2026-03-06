// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * RecoveryTab — autosave and backup recovery options.
 */

import { Show } from "solid-js";
import type { Accessor } from "solid-js";
import { HiOutlineArrowPath, HiOutlineClock } from "../../icons";
import { formatAutosaveAge } from "./types";
import type { RecoveryInfo } from "../../../hooks/useProjectRecovery";

export interface RecoveryTabProps {
  loading: Accessor<boolean>;
  recoveryInfo: Accessor<RecoveryInfo | null>;
  onRecoverAutosave: () => void;
  onClearAutosave: () => void;
}

export function RecoveryTab(props: RecoveryTabProps) {
  return (
    <div class="space-y-6">
      <Show
        when={props.recoveryInfo()}
        fallback={<div class="text-txt-muted">Checking recovery options...</div>}
      >
        {(info) => (
          <>
            {/* Autosave Recovery */}
            <div class="p-4 rounded-md bg-bg-secondary">
              <h4 class="font-medium text-txt mb-3 flex items-center gap-2">
                <HiOutlineArrowPath class="w-icon-sm h-icon-sm" />
                Autosave Recovery
              </h4>
              <Show
                when={info().has_autosave}
                fallback={<p class="text-txt-muted">No autosave file found</p>}
              >
                <div class="space-y-3">
                  <p class="text-txt-secondary">
                    Autosave available from{" "}
                    {formatAutosaveAge(info().autosave_age_seconds)}
                  </p>
                  <Show when={info().autosave_is_newer}>
                    <div class="p-2 rounded bg-warning/20 text-warning text-sm">
                      ⚠️ Autosave is newer than the saved project
                    </div>
                  </Show>
                  <div class="flex gap-2">
                    <button
                      onClick={props.onRecoverAutosave}
                      disabled={props.loading()}
                      class="px-4 py-2 bg-success hover:bg-success/90 text-white rounded-md transition-colors disabled:opacity-50"
                    >
                      Recover from Autosave
                    </button>
                    <button
                      onClick={props.onClearAutosave}
                      disabled={props.loading()}
                      class="px-4 py-2 bg-bg-hover hover:bg-bg-active text-txt rounded-md transition-colors disabled:opacity-50"
                    >
                      Clear Autosave
                    </button>
                  </div>
                </div>
              </Show>
            </div>

            {/* Backup Recovery */}
            <div class="p-4 rounded-md bg-bg-secondary">
              <h4 class="font-medium text-txt mb-3 flex items-center gap-2">
                <HiOutlineClock class="w-icon-sm h-icon-sm" />
                Backup Recovery
              </h4>
              <Show
                when={info().has_backup}
                fallback={<p class="text-txt-muted">No backup file found</p>}
              >
                <p class="text-txt-secondary">
                  Backup available at:{" "}
                  <code class="text-xs bg-bg px-1 rounded">{info().backup_path}</code>
                </p>
              </Show>
            </div>
          </>
        )}
      </Show>
    </div>
  );
}
