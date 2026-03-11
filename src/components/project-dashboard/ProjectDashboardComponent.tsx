// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ProjectDashboard - Overview panel showing project statistics,
 * recent activity, evidence summary, and quick actions.
 */

import { Component, Show, For, createSignal, createMemo, onMount } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import {
  HiOutlineArchiveBox,
  HiOutlineBookmark,
  HiOutlineDocumentText,
  HiOutlineClock,
  HiOutlineFingerPrint,
  HiOutlineTag,
  HiOutlineUser,
  HiOutlineShieldCheck,
  HiOutlineArrowUpTray,
  HiOutlineClipboardDocumentList,
} from "../icons";
import type { ProjectDbStats } from "../../types/projectDb";
import { formatBytes } from "../../utils";
import { logger } from "../../utils/logger";
import type { ProjectDashboardProps } from "./types";
import { formatTimeAgo } from "./helpers";
import { StatCard } from "./StatCard";

const log = logger.scope("ProjectDashboard");

export const ProjectDashboard: Component<ProjectDashboardProps> = (props) => {
  const [dbStats, setDbStats] = createSignal<ProjectDbStats | null>(null);

  // Try to load db stats (non-blocking)
  onMount(async () => {
    try {
      const stats = await invoke<ProjectDbStats>("project_db_get_stats");
      setDbStats(stats);
    } catch {
      log.debug("Could not load DB stats (db may not be open)");
    }
  });

  // ---------- Derived project data ----------

  const projectName = createMemo(() => props.project()?.name ?? "No Project");
  const projectDesc = createMemo(() => props.project()?.description ?? "");
  const createdAt = createMemo(() => {
    const d = props.project()?.created_at;
    return d ? new Date(d).toLocaleDateString() : "—";
  });
  const savedAt = createMemo(() => {
    const d = props.project()?.saved_at;
    return d ? new Date(d).toLocaleString() : "—";
  });

  const evidenceCount = createMemo(() => props.discoveredFiles().length);
  const totalEvidenceSize = createMemo(() =>
    props.discoveredFiles().reduce((sum, f) => sum + (f.size || 0), 0),
  );
  const hashedCount = createMemo(() => {
    let count = 0;
    for (const [, info] of props.fileHashMap()) {
      if (info.hash) count++;
    }
    return count;
  });
  const verifiedCount = createMemo(() => {
    let count = 0;
    for (const [, info] of props.fileHashMap()) {
      if (info.verified === true) count++;
    }
    return count;
  });

  const sessionCount = createMemo(() => props.project()?.sessions?.length ?? 0);
  const tagCount = createMemo(() => props.project()?.tags?.length ?? 0);
  const reportCount = createMemo(() => props.project()?.reports?.length ?? 0);
  const activityCount = createMemo(() => props.project()?.activity_log?.length ?? 0);

  const currentUser = createMemo(() => props.project()?.current_user ?? "—");
  const userCount = createMemo(() => props.project()?.users?.length ?? 0);

  // Recent activity (last 8 entries)
  const recentActivity = createMemo(() => {
    const activityLog = props.project()?.activity_log;
    if (!activityLog || activityLog.length === 0) return [];
    return activityLog.slice(0, 8);
  });

  // Evidence type breakdown
  const evidenceByType = createMemo(() => {
    const counts: Record<string, number> = {};
    for (const f of props.discoveredFiles()) {
      const dotIdx = f.filename.lastIndexOf(".");
      const ext = dotIdx >= 0 ? f.filename.substring(dotIdx + 1).toUpperCase() : "OTHER";
      counts[ext] = (counts[ext] || 0) + 1;
    }
    return Object.entries(counts).sort((a, b) => b[1] - a[1]);
  });

  return (
    <div class="flex-1 flex flex-col overflow-y-auto p-3 gap-4">
      {/* === Project Header === */}
      <Show
        when={props.project()}
        fallback={
          <div class="flex-1 flex items-center justify-center text-txt-muted text-sm">
            <div class="text-center space-y-2">
              <span class="text-3xl">📂</span>
              <p>No project loaded</p>
              <p class="text-xs">Open or create a project to see the dashboard</p>
            </div>
          </div>
        }
      >
        <div class="space-y-1">
          <h2 class="text-base font-semibold text-txt truncate" title={projectName()}>
            {projectName()}
          </h2>
          <Show when={projectDesc()}>
            <p class="text-xs text-txt-muted line-clamp-2">{projectDesc()}</p>
          </Show>
          <div class="flex items-center gap-3 text-2xs text-txt-muted">
            <span>Created {createdAt()}</span>
            <span>·</span>
            <span>Saved {savedAt()}</span>
          </div>
        </div>

        {/* === Stats Grid === */}
        <div class="grid grid-cols-2 gap-2">
          <StatCard icon={HiOutlineArchiveBox} label="Evidence" value={evidenceCount()} accent onClick={() => props.onNavigateTab?.("evidence")} />
          <StatCard icon={HiOutlineFingerPrint} label="Hashed" value={`${hashedCount()} / ${evidenceCount()}`} />
          <StatCard icon={HiOutlineBookmark} label="Bookmarks" value={props.bookmarkCount()} onClick={() => props.onNavigateTab?.("bookmarks")} />
          <StatCard icon={HiOutlineDocumentText} label="Notes" value={props.noteCount()} />
          <StatCard icon={HiOutlineTag} label="Tags" value={tagCount()} />
          <StatCard icon={HiOutlineShieldCheck} label="Verified" value={verifiedCount()} />
          <StatCard icon={HiOutlineClipboardDocumentList} label="Reports" value={reportCount()} onClick={props.onReport} />
          <StatCard icon={HiOutlineArrowUpTray} label="Export" value="" onClick={props.onExport} />
          <StatCard icon={HiOutlineUser} label="Sessions" value={sessionCount()} />
        </div>

        {/* === Evidence Summary === */}
        <Show when={evidenceByType().length > 0}>
          <div class="space-y-2">
            <h3 class="text-xs font-semibold text-txt-secondary uppercase tracking-wider">Evidence Breakdown</h3>
            <div class="space-y-1">
              <For each={evidenceByType()}>
                {([type, count]) => (
                  <div class="flex items-center justify-between px-2 py-1 bg-bg-secondary rounded text-xs">
                    <span class="font-medium text-txt">{type}</span>
                    <span class="text-txt-muted">{count} file{count !== 1 ? "s" : ""}</span>
                  </div>
                )}
              </For>
              <div class="flex items-center justify-between px-2 py-1 text-xs text-txt-muted">
                <span>Total Size</span>
                <span class="font-medium text-txt">{formatBytes(totalEvidenceSize())}</span>
              </div>
            </div>
          </div>
        </Show>

        {/* === Current User === */}
        <div class="flex items-center gap-2 px-2 py-1.5 bg-bg-secondary rounded text-xs">
          <HiOutlineUser class="w-3.5 h-3.5 text-accent" />
          <span class="text-txt-muted">User:</span>
          <span class="font-medium text-txt">{currentUser()}</span>
          <Show when={userCount() > 1}>
            <span class="text-txt-muted ml-auto">({userCount()} total)</span>
          </Show>
        </div>

        {/* === Recent Activity === */}
        <Show when={recentActivity().length > 0}>
          <div class="space-y-2">
            <div class="flex items-center justify-between">
              <h3 class="text-xs font-semibold text-txt-secondary uppercase tracking-wider">Recent Activity</h3>
              <button class="text-2xs text-accent hover:underline" onClick={() => props.onNavigateTab?.("activity")}>
                View all ({activityCount()})
              </button>
            </div>
            <div class="space-y-0.5">
              <For each={recentActivity()}>
                {(entry) => {
                  const time = new Date(entry.timestamp);
                  const ago = formatTimeAgo(time);
                  return (
                    <div class="flex items-start gap-2 px-2 py-1.5 rounded hover:bg-bg-hover text-xs group">
                      <HiOutlineClock class="w-3 h-3 text-txt-muted mt-0.5 flex-shrink-0" />
                      <div class="flex-1 min-w-0">
                        <p class="text-txt truncate">{entry.description}</p>
                        <p class="text-2xs text-txt-muted">
                          {entry.action} · {ago}
                        </p>
                      </div>
                    </div>
                  );
                }}
              </For>
            </div>
          </div>
        </Show>

        {/* === DB Stats (if available) === */}
        <Show when={dbStats()}>
          {(stats) => (
            <div class="space-y-2">
              <h3 class="text-xs font-semibold text-txt-secondary uppercase tracking-wider">Database</h3>
              <div class="grid grid-cols-2 gap-1 text-2xs">
                <div class="flex justify-between px-2 py-1 bg-bg-secondary rounded">
                  <span class="text-txt-muted">Schema</span>
                  <span class="text-txt">v{stats().schemaVersion}</span>
                </div>
                <div class="flex justify-between px-2 py-1 bg-bg-secondary rounded">
                  <span class="text-txt-muted">Size</span>
                  <span class="text-txt">{formatBytes(stats().dbSizeBytes)}</span>
                </div>
                <div class="flex justify-between px-2 py-1 bg-bg-secondary rounded">
                  <span class="text-txt-muted">Activities</span>
                  <span class="text-txt">{stats().totalActivities}</span>
                </div>
                <div class="flex justify-between px-2 py-1 bg-bg-secondary rounded">
                  <span class="text-txt-muted">Custody</span>
                  <span class="text-txt">{stats().totalCustodyRecords}</span>
                </div>
              </div>
            </div>
          )}
        </Show>
      </Show>
    </div>
  );
};
