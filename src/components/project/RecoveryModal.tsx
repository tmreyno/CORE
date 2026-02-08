// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show, For, createSignal, createEffect } from "solid-js";
import { Dynamic } from "solid-js/web";
import { 
  useProjectRecovery, 
  type BackupFile, 
  type ProjectHealthStatus 
} from "../../hooks/useProjectRecovery";
import { HiOutlineShieldCheck, HiOutlineExclamationTriangle, HiOutlineCheckCircle, HiOutlineXCircle, HiOutlineClock, HiOutlineArrowPath } from "../icons";

interface RecoveryModalProps {
  isOpen: boolean;
  onClose: () => void;
  projectPath: string;
}

export const RecoveryModal: Component<RecoveryModalProps> = (props) => {
  const recovery = useProjectRecovery();
  const [activeTab, setActiveTab] = createSignal<"backups" | "health" | "recovery">("backups");
  const [selectedBackup, setSelectedBackup] = createSignal<BackupFile | null>(null);

  // Load data when modal opens
  const loadData = async () => {
    if (props.isOpen && props.projectPath) {
      await Promise.all([
        recovery.listVersions(props.projectPath),
        recovery.checkHealth(props.projectPath),
        recovery.checkRecovery(props.projectPath),
      ]);
    }
  };

  // Auto-load when modal opens
  createEffect(() => {
    if (props.isOpen) {
      loadData();
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
      "Recover from autosave?\n\nThis will replace the current project with the autosaved version."
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

  const getHealthIcon = (status: ProjectHealthStatus) => {
    switch (status) {
      case "Healthy":
        return HiOutlineCheckCircle;
      case "Warning":
        return HiOutlineExclamationTriangle;
      case "Critical":
        return HiOutlineXCircle;
      default:
        return HiOutlineShieldCheck;
    }
  };

  const getHealthColor = (status: ProjectHealthStatus) => {
    switch (status) {
      case "Healthy":
        return "text-success";
      case "Warning":
        return "text-warning";
      case "Critical":
        return "text-error";
      default:
        return "text-txt-muted";
    }
  };

  const formatFileSize = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
  };

  const formatAutosaveAge = (seconds: number | null): string => {
    if (seconds === null) return "Unknown";
    if (seconds < 60) return `${seconds} seconds ago`;
    if (seconds < 3600) return `${Math.floor(seconds / 60)} minutes ago`;
    if (seconds < 86400) return `${Math.floor(seconds / 3600)} hours ago`;
    return `${Math.floor(seconds / 86400)} days ago`;
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
            <button
              onClick={props.onClose}
              class="icon-btn-sm"
            >
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
            {/* Backups Tab */}
            <Show when={activeTab() === "backups"}>
              <div class="space-y-4">
                {/* Actions */}
                <div class="flex gap-2">
                  <button
                    onClick={handleCreateBackup}
                    disabled={recovery.loading()}
                    class="btn-sm-primary"
                  >
                    Create Backup
                  </button>
                  <button
                    onClick={handleCreateVersion}
                    disabled={recovery.loading()}
                    class="btn-sm"
                  >
                    Create Version Snapshot
                  </button>
                </div>

                {/* Backup List */}
                <Show when={recovery.versions().length === 0}>
                  <div class="text-center py-12 text-txt-muted">
                    <HiOutlineClock class="w-12 h-12 mx-auto mb-4 opacity-50" />
                    <p>No version backups found</p>
                  </div>
                </Show>

                <div class="space-y-2">
                  <For each={recovery.versions()}>
                    {(backup) => (
                      <div
                        class={`p-4 rounded-md border cursor-pointer transition-colors ${
                          selectedBackup()?.path === backup.path
                            ? "border-accent bg-bg-active"
                            : "border-border hover:border-accent/50 hover:bg-bg-hover"
                        }`}
                        onClick={() => setSelectedBackup(backup)}
                      >
                        <div class="flex items-start justify-between">
                          <div class="flex-1">
                            <div class="flex items-center gap-2 mb-1">
                              <span class="font-medium text-txt">
                                {backup.path.split('/').pop() || "Backup"}
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
                              {formatFileSize(backup.metadata.file_size)} • v{backup.metadata.app_version}
                            </p>
                          </div>
                        </div>
                      </div>
                    )}
                  </For>
                </div>
              </div>
            </Show>

            {/* Health Tab */}
            <Show when={activeTab() === "health"}>
              <Show when={recovery.health()} fallback={<div class="text-txt-muted">Loading health check...</div>}>
                {(health) => (
                  <div class="space-y-6">
                    {/* Health Status */}
                    <div class="flex items-center gap-4 p-4 rounded-md bg-bg-secondary">
                      <div class={`text-4xl ${getHealthColor(health().status)}`}>
                        <Dynamic component={getHealthIcon(health().status)} class="w-8 h-8" />
                      </div>
                      <div class="flex-1">
                        <h3 class="text-lg font-semibold text-txt capitalize">
                          {health().status}
                        </h3>
                        <p class="text-txt-secondary">
                          {health().version_count} version backups • {health().tab_count} tabs • {health().activity_log_size} activity entries
                        </p>
                      </div>
                    </div>

                    {/* Health Issues */}
                    <Show when={health().issues.length > 0}>
                      <div class="space-y-2">
                        <h4 class="font-medium text-txt mb-3">Issues Detected</h4>
                        <For each={health().issues}>
                          {(issue) => (
                            <div class="flex items-start gap-3 p-3 rounded-md bg-bg-panel">
                              <div class={`mt-0.5 ${recovery.getSeverityColor(issue.severity)}`}>
                                {issue.severity === "Info" || issue.severity === "Warning" ? 
                                  <HiOutlineExclamationTriangle class="w-icon-sm h-icon-sm" /> :
                                  <HiOutlineXCircle class="w-icon-sm h-icon-sm" />}
                              </div>
                              <div class="flex-1">
                                <p class="font-medium text-txt">{issue.category}</p>
                                <p class="text-sm text-txt-secondary">{issue.message}</p>
                                <Show when={issue.recommendation}>
                                  <p class="text-xs text-txt-muted mt-1">
                                    💡 {issue.recommendation}
                                  </p>
                                </Show>
                              </div>
                            </div>
                          )}
                        </For>
                      </div>
                    </Show>

                    <Show when={health().issues.length === 0}>
                      <div class="text-center py-8 text-success">
                        <HiOutlineCheckCircle class="w-12 h-12 mx-auto mb-4" />
                        <p class="font-medium">No issues detected</p>
                        <p class="text-sm text-txt-muted">Your project is healthy</p>
                      </div>
                    </Show>

                    {/* File Info */}
                    <div class="p-4 rounded-md bg-bg-panel">
                      <h4 class="font-medium text-txt mb-3">Project Statistics</h4>
                      <div class="grid grid-cols-2 gap-4 text-sm">
                        <div>
                          <span class="text-txt-muted">File Size:</span>
                          <span class="ml-2 text-txt">{formatFileSize(health().file_size)}</span>
                        </div>
                        <div>
                          <span class="text-txt-muted">Activity Log:</span>
                          <span class="ml-2 text-txt">{health().activity_log_size} entries</span>
                        </div>
                        <div>
                          <span class="text-txt-muted">Tabs:</span>
                          <span class="ml-2 text-txt">{health().tab_count}</span>
                        </div>
                        <div>
                          <span class="text-txt-muted">Sessions:</span>
                          <span class="ml-2 text-txt">{health().session_count}</span>
                        </div>
                        <div>
                          <span class="text-txt-muted">Has Backup:</span>
                          <span class="ml-2 text-txt">{health().has_backup ? "Yes" : "No"}</span>
                        </div>
                        <div>
                          <span class="text-txt-muted">Versions:</span>
                          <span class="ml-2 text-txt">{health().version_count}</span>
                        </div>
                      </div>
                    </div>
                  </div>
                )}
              </Show>
            </Show>

            {/* Recovery Tab */}
            <Show when={activeTab() === "recovery"}>
              <div class="space-y-6">
                <Show when={recovery.recoveryInfo()} fallback={<div class="text-txt-muted">Checking recovery options...</div>}>
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
                          fallback={
                            <p class="text-txt-muted">No autosave file found</p>
                          }
                        >
                          <div class="space-y-3">
                            <p class="text-txt-secondary">
                              Autosave available from {formatAutosaveAge(info().autosave_age_seconds)}
                            </p>
                            <Show when={info().autosave_is_newer}>
                              <div class="p-2 rounded bg-warning/20 text-warning text-sm">
                                ⚠️ Autosave is newer than the saved project
                              </div>
                            </Show>
                            <div class="flex gap-2">
                              <button
                                onClick={handleRecoverAutosave}
                                disabled={recovery.loading()}
                                class="px-4 py-2 bg-success hover:bg-success/90 text-white rounded-md transition-colors disabled:opacity-50"
                              >
                                Recover from Autosave
                              </button>
                              <button
                                onClick={handleClearAutosave}
                                disabled={recovery.loading()}
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
                          fallback={
                            <p class="text-txt-muted">No backup file found</p>
                          }
                        >
                          <p class="text-txt-secondary">
                            Backup available at: <code class="text-xs bg-bg px-1 rounded">{info().backup_path}</code>
                          </p>
                        </Show>
                      </div>
                    </>
                  )}
                </Show>
              </div>
            </Show>
          </div>

          {/* Footer */}
          <div class="modal-footer justify-end">
            <button
              onClick={props.onClose}
              class="btn-sm"
            >
              Close
            </button>
          </div>
        </div>
      </div>
    </Show>
  );
};
