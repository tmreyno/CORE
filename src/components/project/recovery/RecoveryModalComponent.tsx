// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * RecoveryModalComponent — slim shell that wires tabs to sub-components.
 */

import { Component, Show, createSignal, createEffect } from "solid-js";
import { useProjectRecovery, type BackupFile } from "../../../hooks/useProjectRecovery";
import { HiOutlineShieldCheck, HiOutlineXCircle } from "../../icons";
import { BackupsTab } from "./BackupsTab";
import { HealthTab } from "./HealthTab";
import { RecoveryTab } from "./RecoveryTab";
import type { RecoveryModalProps } from "./types";

export const RecoveryModal: Component<RecoveryModalProps> = (props) => {
  const recovery = useProjectRecovery();
  const [activeTab, setActiveTab] = createSignal<"backups" | "health" | "recovery">("backups");
  const [selectedBackup, setSelectedBackup] = createSignal<BackupFile | null>(null);

  // Load data when modal opens
  createEffect(() => {
    if (props.isOpen && props.projectPath) {
      Promise.all([
        recovery.listVersions(props.projectPath),
        recovery.checkHealth(props.projectPath),
        recovery.checkRecovery(props.projectPath),
      ]);
    }
  });

  const handleCreateBackup = async () => {
    const result = await recovery.createBackup(props.projectPath, "ManualBackup");
    if (result) {
      alert("Manual backup created successfully");
    } else {
      alert(`Backup failed: ${recovery.error() || "Unknown error"}`);
    }
  };

  const handleCreateVersion = async () => {
    const result = await recovery.createVersionBackup(props.projectPath);
    if (result) {
      alert("Version backup created successfully");
    } else {
      alert(`Version backup failed: ${recovery.error() || "Unknown error"}`);
    }
  };

  const handleRecoverAutosave = async () => {
    const recoveryInfo = recovery.recoveryInfo();
    if (!recoveryInfo?.has_autosave) return;

    const confirmed = confirm(
      "Recover from autosave?\n\nThis will replace the current project with the autosaved version.",
    );
    if (!confirmed) return;

    const result = await recovery.recoverFromAutosave(props.projectPath);
    if (result) {
      alert("Successfully recovered from autosave");
      props.onClose();
    } else {
      alert(`Recovery failed: ${recovery.error() || "Unknown error"}`);
    }
  };

  const handleClearAutosave = async () => {
    const confirmed = confirm("Clear autosave file? This cannot be undone.");
    if (!confirmed) return;

    const success = await recovery.clearAutosave(props.projectPath);
    if (success) {
      alert("Autosave cleared");
      await recovery.checkRecovery(props.projectPath);
    }
  };

  return (
    <Show when={props.isOpen}>
      <div class="modal-overlay">
        <div class="modal-content max-w-4xl w-full max-h-[90vh] flex flex-col">
          {/* Header */}
          <div class="modal-header">
            <div class="flex items-center gap-3">
              <HiOutlineShieldCheck class="w-icon-lg h-icon-lg text-accent" />
              <h2 class="text-xl font-semibold text-txt">Project Recovery</h2>
            </div>
            <button onClick={props.onClose} class="icon-btn-sm">
              <HiOutlineXCircle class="w-icon-base h-icon-base" />
            </button>
          </div>

          {/* Tabs */}
          <div class="flex border-b border-border px-5">
            <button
              class={`py-3 px-4 font-medium border-b-2 transition-colors ${
                activeTab() === "backups"
                  ? "border-accent text-accent"
                  : "border-transparent text-txt-secondary hover:text-txt"
              }`}
              onClick={() => setActiveTab("backups")}
            >
              Version Backups ({recovery.versions().length})
            </button>
            <button
              class={`py-3 px-4 font-medium border-b-2 transition-colors ${
                activeTab() === "health"
                  ? "border-accent text-accent"
                  : "border-transparent text-txt-secondary hover:text-txt"
              }`}
              onClick={() => setActiveTab("health")}
            >
              Health Check
            </button>
            <button
              class={`py-3 px-4 font-medium border-b-2 transition-colors ${
                activeTab() === "recovery"
                  ? "border-accent text-accent"
                  : "border-transparent text-txt-secondary hover:text-txt"
              }`}
              onClick={() => setActiveTab("recovery")}
            >
              Recovery Options
            </button>
          </div>

          {/* Content */}
          <div class="modal-body">
            <Show when={activeTab() === "backups"}>
              <BackupsTab
                loading={recovery.loading}
                versions={recovery.versions}
                selectedBackup={selectedBackup}
                setSelectedBackup={setSelectedBackup}
                onCreateBackup={handleCreateBackup}
                onCreateVersion={handleCreateVersion}
              />
            </Show>

            <Show when={activeTab() === "health"}>
              <HealthTab
                health={recovery.health}
                getSeverityColor={recovery.getSeverityColor}
              />
            </Show>

            <Show when={activeTab() === "recovery"}>
              <RecoveryTab
                loading={recovery.loading}
                recoveryInfo={recovery.recoveryInfo}
                onRecoverAutosave={handleRecoverAutosave}
                onClearAutosave={handleClearAutosave}
              />
            </Show>
          </div>

          {/* Footer */}
          <div class="modal-footer justify-end">
            <button onClick={props.onClose} class="btn-sm">
              Close
            </button>
          </div>
        </div>
      </div>
    </Show>
  );
};
