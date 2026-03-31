// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show, createMemo, createSignal, type Accessor, type Setter } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { CoreSpinner } from "@core-suite/icons";
import {
  HiOutlineArrowPath,
  HiOutlineCircleStack,
  HiOutlineComputerDesktop,
  HiOutlineFolder,
  HiOutlineCheckCircle,
} from "../icons";
import { listDrives, type DriveInfo } from "../../api/drives";
import type { SystemStats } from "../../hooks";
import { generateEvidenceFolderName } from "../../utils/evidenceNaming";
import { DriveTreeBrowser } from "../export-panel/DriveTreeBrowser";
import { useToast } from "../Toast";
import SystemInfoPanel from "./SystemInfoPanel";
import AcquireProcessShell from "./AcquireProcessShell";

const formatSystemBytes = (bytes: number): string => {
  if (bytes <= 0) return "0 B";
  const gb = bytes / (1024 * 1024 * 1024);
  if (gb >= 1) return `${gb.toFixed(gb >= 10 ? 0 : 1)} GB`;
  const mb = bytes / (1024 * 1024);
  return `${mb.toFixed(0)} MB`;
};

export interface AcquireIdentifyViewProps {
  onBack: () => void;
  hasProject: Accessor<boolean>;
  projectName: Accessor<string | undefined>;
  currentUsername?: string;
  evidenceBasePath?: string;
  systemStatsData: Accessor<SystemStats | null>;
  setSystemStatsData: Setter<SystemStats | null>;
  systemDrivesData: Accessor<DriveInfo[]>;
  setSystemDrivesData: Setter<DriveInfo[]>;
  evidenceItemFolder: Accessor<string>;
  setEvidenceItemFolder: Setter<string>;
  onOpenCollection: () => void;
  onOpenBrowse: () => void;
}

const AcquireIdentifyView: Component<AcquireIdentifyViewProps> = (props) => {
  const toast = useToast();
  const [showSystemPanel, setShowSystemPanel] = createSignal(true);
  const [isLoading, setIsLoading] = createSignal(false);
  const [selectedSources, setSelectedSources] = createSignal<Set<string>>(new Set());

  const hasSystemData = createMemo(() => props.systemStatsData() != null);
  const systemStats = createMemo(() => props.systemStatsData());
  const drives = createMemo(() => props.systemDrivesData());

  const systemSummary = createMemo(() => {
    const stats = systemStats();
    if (!stats) return null;
    const parts: string[] = [];
    if (stats.hostname) parts.push(stats.hostname);
    if (stats.systemModel) parts.push(stats.systemModel);
    if (stats.osName) parts.push(stats.osVersion ? `${stats.osName} ${stats.osVersion}` : stats.osName);
    if (drives().length > 0) parts.push(`${drives().length} volume${drives().length !== 1 ? "s" : ""}`);
    return parts.join(" · ");
  });

  const handleSelectSource = (path: string) => {
    setSelectedSources((prev) => {
      const next = new Set(prev);
      if (next.has(path)) next.delete(path);
      else next.add(path);
      return next;
    });
  };

  const runIdentify = async () => {
    if (!props.hasProject()) return;
    setIsLoading(true);
    try {
      const [list, stats] = await Promise.all([
        listDrives(),
        invoke<SystemStats>("get_system_stats"),
      ]);
      props.setSystemDrivesData(list);
      props.setSystemStatsData(stats);
      setShowSystemPanel(true);

      if (props.evidenceBasePath) {
        try {
          const folderName = generateEvidenceFolderName(
            props.projectName(),
            stats,
            props.currentUsername,
          );
          const folderPath = `${props.evidenceBasePath}/${folderName}`;
          await invoke("create_directory", { path: folderPath });
          props.setEvidenceItemFolder(folderPath);
          toast.success("Evidence Folder Created", folderPath.split("/").pop() || folderPath);
        } catch (error) {
          console.warn("Failed to create evidence item folder:", error);
        }
      }
    } finally {
      setIsLoading(false);
    }
  };

  // System info and disk info are collected ONLY when the user explicitly
  // clicks "Identify System" — no auto-scan on project creation.

  return (
    <AcquireProcessShell
      title="Identify System"
      onBack={props.onBack}
      headerActions={
        <>
          <Show when={hasSystemData() && !isLoading()}>
            <button class="btn btn-ghost gap-1 text-xs py-1 px-2" onClick={() => void runIdentify()}>
              <HiOutlineArrowPath class="w-icon-sm h-icon-sm" />
              Re-Scan
            </button>
          </Show>
          <button
            class="icon-btn-sm"
            classList={{ "text-accent": showSystemPanel(), "text-txt-muted": !showSystemPanel() }}
            onClick={() => setShowSystemPanel((value) => !value)}
            title={showSystemPanel() ? "Hide System Info" : "Show System Info"}
          >
            <HiOutlineComputerDesktop class="w-icon-sm h-icon-sm" />
          </button>
        </>
      }
    >

      {/* ── Content area ── */}
      <div class="flex flex-1 min-h-0 overflow-hidden">
        <div class="flex-1 min-h-0 overflow-auto">
          <div class="flex-1 min-h-0 overflow-y-auto px-3 py-2">
            <div class="w-full max-w-[640px] mx-auto flex flex-col gap-2.5">
              {/* ── System Survey section ── */}
              <div class="flex flex-col gap-2 pb-3 border-b border-border last:border-b-0 last:pb-0">
                <div class="flex items-center gap-2">
                  <HiOutlineComputerDesktop class="w-icon-sm h-icon-sm text-accent" />
                  <span class="text-xs font-semibold text-txt">System Survey</span>
                </div>

                <Show when={!props.hasProject()}>
                  <div class="callout callout-warning">
                    <span class="text-sm text-txt">Open or create a project before capturing system identity and evidence location data.</span>
                  </div>
                </Show>

                <Show when={props.hasProject() && isLoading()}>
                  <div class="callout">
                    <div class="flex items-center gap-2 text-sm text-txt">
                      <CoreSpinner size={16} />
                      Collecting system, volume, and destination metadata...
                    </div>
                  </div>
                </Show>

                <Show when={props.hasProject() && !isLoading() && !hasSystemData()}>
                  <div class="callout">
                    <span class="text-sm text-txt">Capture host identity, detected volumes, and the evidence item folder before moving into collection or review.</span>
                  </div>
                </Show>

                <Show when={hasSystemData() && !isLoading()}>
                  <div class="callout callout-success">
                    <div class="flex items-center gap-2 text-sm text-txt">
                      <HiOutlineCheckCircle class="w-icon-sm h-icon-sm text-success" />
                      <span>{systemSummary()}</span>
                    </div>
                  </div>
                </Show>

                <div class="flex flex-wrap gap-2">
                  <button class="btn btn-primary gap-1" onClick={() => void runIdentify()} disabled={!props.hasProject() || isLoading()}>
                    <Show when={isLoading()} fallback={<HiOutlineComputerDesktop class="w-icon-sm h-icon-sm" />}>
                      <CoreSpinner size={16} />
                    </Show>
                    {hasSystemData() ? "Re-Run Identify" : "Identify System"}
                  </button>
                </div>

                <Show when={props.evidenceItemFolder()}>
                  <div class="px-2 py-1.5 rounded bg-bg-secondary border border-border text-xs text-txt-muted">
                    <span class="font-medium">Evidence Folder: </span>
                    <span class="font-mono text-compact break-all">{props.evidenceItemFolder()}</span>
                  </div>
                </Show>

                <Show when={hasSystemData()}>
                  <div class="grid grid-cols-2 sm:grid-cols-4 gap-2">
                    <div class="stat-box">
                      <div class="text-txt-muted text-xs">Model</div>
                      <div class="text-sm font-semibold text-txt truncate" title={systemStats()?.systemModel}>{systemStats()?.systemModel || "Unknown"}</div>
                    </div>
                    <div class="stat-box">
                      <div class="text-txt-muted text-xs">Hostname</div>
                      <div class="text-sm font-semibold text-txt truncate" title={systemStats()?.hostname}>{systemStats()?.hostname || "Unknown"}</div>
                    </div>
                    <div class="stat-box">
                      <div class="text-txt-muted text-xs">Memory</div>
                      <div class="text-sm font-semibold text-txt">{formatSystemBytes(systemStats()?.memoryTotal || 0)}</div>
                    </div>
                    <div class="stat-box">
                      <div class="text-txt-muted text-xs">Volumes</div>
                      <div class="text-sm font-semibold text-txt">{drives().length}</div>
                    </div>
                  </div>
                </Show>
              </div>

              {/* ── Drive & Source Survey section ── */}
              <div class="flex flex-col gap-2 pb-3 border-b border-border last:border-b-0 last:pb-0">
                <div class="flex items-center gap-2">
                  <HiOutlineCircleStack class="w-icon-sm h-icon-sm text-accent" />
                  <span class="text-xs font-semibold text-txt">Drive And Source Survey</span>
                </div>
                <div class="text-xs text-txt-muted">
                  Review detected mount points and mark likely evidence sources.
                </div>
                <div class="border border-border rounded-lg bg-bg-secondary/40 max-h-[340px] overflow-hidden">
                  <DriveTreeBrowser
                    onSelectSource={handleSelectSource}
                    selectedPaths={selectedSources}
                  />
                </div>
                <Show when={selectedSources().size > 0}>
                  <div class="flex items-center gap-2 px-2 py-1.5 rounded bg-bg-secondary border border-border">
                    <span class="text-xs text-txt">{selectedSources().size} source{selectedSources().size !== 1 ? "s" : ""} marked for follow-on acquisition or review.</span>
                  </div>
                </Show>
              </div>
            </div>
          </div>
        </div>

        <Show when={showSystemPanel()}>
          <div class="w-72 shrink-0 border-l border-border overflow-hidden">
            <SystemInfoPanel systemStats={props.systemStatsData()} drives={props.systemDrivesData()} />
          </div>
        </Show>
      </div>

      {/* ── Sticky bottom action bar — primary workflow navigation ── */}
      <div class="flex items-center justify-between gap-2 px-3 py-1.5 border-t border-border bg-bg-secondary shrink-0">
        <div class="flex items-center gap-2 text-xs text-txt-muted">
          <Show when={hasSystemData() && !isLoading()}>
            <button class="btn btn-ghost gap-1" onClick={props.onOpenBrowse}>
              <HiOutlineCircleStack class="w-icon-sm h-icon-sm" />
              Browse Evidence
            </button>
          </Show>
        </div>
        <button
          class="btn btn-primary gap-1"
          onClick={props.onOpenCollection}
          disabled={!props.hasProject() || !hasSystemData() || isLoading()}
        >
          <HiOutlineFolder class="w-icon-sm h-icon-sm" />
          Continue To Collection
        </button>
      </div>
    </AcquireProcessShell>
  );
};

export default AcquireIdentifyView;