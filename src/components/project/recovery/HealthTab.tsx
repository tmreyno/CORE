// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * HealthTab — displays project health status, issues, and statistics.
 */

import { Show, For } from "solid-js";
import type { Accessor } from "solid-js";
import { Dynamic } from "solid-js/web";
import {
  HiOutlineExclamationTriangle,
  HiOutlineCheckCircle,
  HiOutlineXCircle,
} from "../../icons";
import { getHealthIcon, getHealthColor, formatFileSize } from "./types";
import type { ProjectHealth, IssueSeverity } from "../../../hooks/useProjectRecovery";

export interface HealthTabProps {
  health: Accessor<ProjectHealth | null>;
  getSeverityColor: (severity: IssueSeverity) => string;
}

export function HealthTab(props: HealthTabProps) {
  return (
    <Show
      when={props.health()}
      fallback={<div class="text-txt-muted">Loading health check...</div>}
    >
      {(health) => (
        <div class="space-y-6">
          {/* Health Status */}
          <div class="flex items-center gap-4 p-4 rounded-md bg-bg-secondary">
            <div class={`text-4xl ${getHealthColor(health().status)}`}>
              <Dynamic component={getHealthIcon(health().status)} class="w-8 h-8" />
            </div>
            <div class="flex-1">
              <h3 class="text-lg font-semibold text-txt capitalize">{health().status}</h3>
              <p class="text-txt-secondary">
                {health().version_count} version backups • {health().tab_count} tabs •{" "}
                {health().activity_log_size} activity entries
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
                    <div class={`mt-0.5 ${props.getSeverityColor(issue.severity)}`}>
                      {issue.severity === "Info" || issue.severity === "Warning" ? (
                        <HiOutlineExclamationTriangle class="w-icon-sm h-icon-sm" />
                      ) : (
                        <HiOutlineXCircle class="w-icon-sm h-icon-sm" />
                      )}
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
  );
}
