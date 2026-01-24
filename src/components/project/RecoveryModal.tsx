// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show, For, createSignal, createEffect } from "solid-js";
import { useProjectRecovery, type BackupFile, type ProjectHealth } from "../../hooks/useProjectRecovery";
import { HiOutlineShieldCheck, HiOutlineExclamationTriangle, HiOutlineCheckCircle, HiOutlineXCircle, HiOutlineClock, HiOutlineArrowPath } from "../icons";

interface RecoveryModalProps {
  isOpen: boolean;
  onClose: () => void;
  projectPath: string;
}

export const RecoveryModal: Component<RecoveryModalProps> = (props) => {
  const recovery = useProjectRecovery();
  const [activeTab, setActiveTab] = createSignal<"backups" | "health">("backups");
  const [selectedBackup, setSelectedBackup] = createSignal<BackupFile | null>(null);

  // Load data when modal opens
  const loadData = async () => {
    if (props.isOpen && props.projectPath) {
      await Promise.all([
        recovery.listBackups(props.projectPath),
        recovery.checkHealth(props.projectPath),
      ]);
    }
  };

  // Auto-load when modal opens
  createEffect(() => {
    if (props.isOpen) {
      loadData();
    }
  });

  const handleRestore = async () => {
    const backup = selectedBackup();
    if (!backup) return;

    const confirmed = confirm(
      `Restore project from backup created at ${new Date(backup.metadata.created_at).toLocaleString()}?\n\nThis will replace the current project.`
    );
    if (!confirmed) return;

    const success = await recovery.restoreBackup(backup.path, props.projectPath);
    if (success) {
      alert("Successfully restored from backup");
      props.onClose();
    } else {
      alert(`Restore failed: ${recovery.error() || "Unknown error"}`);
    }
  };

  const handleCreateBackup = async () => {
    const success = await recovery.createBackup(props.projectPath);
    if (success) {
      alert("Manual backup created successfully");
    } else {
      alert(`Backup failed: ${recovery.error() || "Unknown error"}`);
    }
  };

  const handleCleanBackups = async () => {
    const confirmed = confirm("Remove old backups? This will keep the 10 most recent backups.");
    if (!confirmed) return;

    const removed = await recovery.cleanBackups(props.projectPath, 10);
    if (removed > 0) {
      alert(`Removed ${removed} old backup(s)`);
    }
  };

  const getHealthIcon = (health: ProjectHealth | null) => {
    if (!health) return HiOutlineShieldCheck;
    switch (health.status) {
      case "healthy":
        return HiOutlineCheckCircle;
      case "warning":
        return HiOutlineExclamationTriangle;
      case "critical":
        return HiOutlineXCircle;
    }
  };

  const getHealthColor = (health: ProjectHealth | null) => {
    if (!health) return "text-txt-muted";
    switch (health.status) {
      case "healthy":
        return "text-success";
      case "warning":
        return "text-warning";
      case "critical":
        return "text-error";
    }
  };

  const formatFileSize = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
  };

  return (
    <Show when={props.isOpen}>
      <div className="fixed inset-0 z-modal-backdrop bg-black/50 flex items-center justify-center p-4">
        <div className="bg-bg rounded-lg shadow-xl max-w-4xl w-full max-h-[90vh] flex flex-col">
          {/* Header */}
          <div className="flex items-center justify-between p-6 border-b border-border">
            <div className="flex items-center gap-3">
              <HiOutlineShieldCheck className="w-icon-lg h-icon-lg text-accent" />
              <h2 className="text-xl font-semibold text-txt">Project Recovery</h2>
            </div>
            <button
              onClick={props.onClose}
              className="text-txt-muted hover:text-txt transition-colors"
            >
              <HiOutlineXCircle className="w-icon-base h-icon-base" />
            </button>
          </div>

          {/* Tabs */}
          <div className="flex border-b border-border px-6">
            <button
              className={`py-3 px-4 font-medium border-b-2 transition-colors ${
                activeTab() === "backups"
                  ? "border-accent text-accent"
                  : "border-transparent text-txt-secondary hover:text-txt"
              }`}
              onClick={() => setActiveTab("backups")}
            >
              Backups ({recovery.backups().length})
            </button>
            <button
              className={`py-3 px-4 font-medium border-b-2 transition-colors ${
                activeTab() === "health"
                  ? "border-accent text-accent"
                  : "border-transparent text-txt-secondary hover:text-txt"
              }`}
              onClick={() => setActiveTab("health")}
            >
              Health Check
            </button>
          </div>

          {/* Content */}
          <div className="flex-1 overflow-y-auto p-6">
            <Show when={activeTab() === "backups"}>
              <div className="space-y-4">
                {/* Actions */}
                <div className="flex gap-2">
                  <button
                    onClick={handleCreateBackup}
                    disabled={recovery.loading()}
                    className="px-4 py-2 bg-accent hover:bg-accent-hover text-white rounded-md transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    Create Backup
                  </button>
                  <button
                    onClick={handleCleanBackups}
                    disabled={recovery.loading()}
                    className="px-4 py-2 bg-bg-secondary hover:bg-bg-hover text-txt rounded-md transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    Clean Old Backups
                  </button>
                  <button
                    onClick={handleRestore}
                    disabled={!selectedBackup() || recovery.loading()}
                    className="px-4 py-2 bg-success hover:bg-success/90 text-white rounded-md transition-colors disabled:opacity-50 disabled:cursor-not-allowed ml-auto"
                  >
                    <div className="flex items-center gap-2">
                      <HiOutlineArrowPath className="w-icon-sm h-icon-sm" />
                      Restore Selected
                    </div>
                  </button>
                </div>

                {/* Backup List */}
                <Show when={recovery.backups().length === 0}>
                  <div className="text-center py-12 text-txt-muted">
                    <HiOutlineClock className="w-12 h-12 mx-auto mb-4 opacity-50" />
                    <p>No backups found</p>
                  </div>
                </Show>

                <div className="space-y-2">
                  <For each={recovery.backups()}>
                    {(backup) => (
                      <div
                        className={`p-4 rounded-md border cursor-pointer transition-colors ${
                          selectedBackup()?.path === backup.path
                            ? "border-accent bg-bg-active"
                            : "border-border hover:border-accent/50 hover:bg-bg-hover"
                        }`}
                        onClick={() => setSelectedBackup(backup)}
                      >
                        <div className="flex items-start justify-between">
                          <div className="flex-1">
                            <div className="flex items-center gap-2 mb-1">
                              <span className="font-medium text-txt">
                                {backup.metadata.project_name}
                              </span>
                              <span
                                className={`text-xs px-2 py-0.5 rounded ${
                                  backup.metadata.backup_type === "manual"
                                    ? "bg-accent/20 text-accent"
                                    : "bg-bg-secondary text-txt-secondary"
                                }`}
                              >
                                {backup.metadata.backup_type}
                              </span>
                            </div>
                            <p className="text-sm text-txt-secondary">
                              {new Date(backup.metadata.created_at).toLocaleString()}
                            </p>
                            <p className="text-xs text-txt-muted mt-1">
                              {formatFileSize(backup.metadata.file_size)} • Version {backup.metadata.project_version}
                            </p>
                          </div>
                        </div>
                      </div>
                    )}
                  </For>
                </div>
              </div>
            </Show>

            <Show when={activeTab() === "health"}>
              <Show when={recovery.health()} fallback={<div className="text-txt-muted">Loading health check...</div>}>
                {(health) => (
                  <div className="space-y-6">
                    {/* Health Score */}
                    <div className="flex items-center gap-4 p-4 rounded-md bg-bg-secondary">
                      <div className={`text-4xl ${getHealthColor(health())}`}>
                        {getHealthIcon(health())}
                      </div>
                      <div className="flex-1">
                        <h3 className="text-lg font-semibold text-txt capitalize">
                          {health().status}
                        </h3>
                        <p className="text-txt-secondary">
                          Health Score: {Math.round(health().score * 100)}%
                        </p>
                      </div>
                    </div>

                    {/* Health Checks */}
                    <div className="space-y-2">
                      <h4 className="font-medium text-txt mb-3">Diagnostic Checks</h4>
                      <For each={health().checks}>
                        {(check) => (
                          <div className="flex items-start gap-3 p-3 rounded-md bg-bg-panel">
                            <div className={`mt-0.5 ${
                              check.status === "pass" ? "text-success" :
                              check.status === "warning" ? "text-warning" : "text-error"
                            }`}>
                              {check.status === "pass" ? <HiOutlineCheckCircle className="w-icon-sm h-icon-sm" /> :
                               check.status === "warning" ? <HiOutlineExclamationTriangle className="w-icon-sm h-icon-sm" /> :
                               <HiOutlineXCircle className="w-icon-sm h-icon-sm" />}
                            </div>
                            <div className="flex-1">
                              <p className="font-medium text-txt">{check.name}</p>
                              <p className="text-sm text-txt-secondary">{check.message}</p>
                            </div>
                          </div>
                        )}
                      </For>
                    </div>

                    {/* Recommendations */}
                    <Show when={health().recommendations.length > 0}>
                      <div className="space-y-2">
                        <h4 className="font-medium text-txt mb-3">Recommendations</h4>
                        <For each={health().recommendations}>
                          {(rec) => (
                            <div className="p-3 rounded-md bg-warning/10 border border-warning/30 text-txt-secondary">
                              {rec}
                            </div>
                          )}
                        </For>
                      </div>
                    </Show>
                  </div>
                )}
              </Show>
            </Show>
          </div>

          {/* Footer */}
          <div className="flex justify-end gap-3 p-6 border-t border-border">
            <button
              onClick={props.onClose}
              className="px-4 py-2 bg-bg-secondary hover:bg-bg-hover text-txt rounded-md transition-colors"
            >
              Close
            </button>
          </div>
        </div>
      </div>
    </Show>
  );
};
