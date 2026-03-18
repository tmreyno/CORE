// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * AcquireDashboard — Primary UI for CORE Acquire edition.
 *
 * Replaces the full CORE-FFX three-panel layout with a streamlined
 * acquisition-focused interface inspired by FTK Imager and Magnet Acquire.
 *
 * Action cards:
 *   1. Create Physical Image (E01)
 *   2. Create Logical Image (L01)
 *   3. Export Files / Archive (7z)
 *   4. Browse Evidence
 *   5. Verify Hash
 *   6. Evidence Collection
 *   7. Memory Capture
 *   8. Forensic Triage (artifact collection + credential/secret scanning)
 */

import {
  Component,
  Show,
  For,
  createSignal,
  createMemo,
  onMount,
  Accessor,
} from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import {
  HiOutlineCircleStack,
  HiOutlineFolder,
  HiOutlineArchiveBox,
  HiOutlineFingerPrint,
  HiOutlineArrowUpTray,
  HiOutlineFolderOpen,
  HiOutlineFolderPlus,
  HiOutlineDocumentCheck,
  HiOutlineCog6Tooth,
  HiOutlineQuestionMarkCircle,
  HiOutlineCommandLine,
  HiOutlineServer,
  HiOutlineExclamationTriangle,
  HiOutlineCpuChip,
  HiOutlineShieldExclamation,
  HiOutlineClock,
  HiOutlineCheckCircle,
  HiOutlineXCircle,
  HiOutlineComputerDesktop,
  HiOutlineArrowPath,
  HiOutlineChevronDown,
  HiOutlineChevronUp,
  HiOutlineGlobeAlt,
} from "../icons";
import { APP_NAME } from "../../utils/edition";
import type { PortableConfig } from "../../api/portable";
import type { DbExportRecord } from "../../types/projectDb";
import { getExportHistory } from "../../api/exportHistory";
import { listDrives, formatDriveSize, type DriveInfo } from "../../api/drives";
import type { SystemStats } from "../../hooks";
import AcquireCollectionSummary from "./AcquireCollectionSummary";

// =============================================================================
// Types
// =============================================================================

export type AcquireAction =
  | "physical"
  | "logical"
  | "export"
  | "browse"
  | "verify"
  | "collection"
  | "memory"
  | "triage";

interface ActionCard {
  id: AcquireAction;
  title: string;
  description: string;
  icon: Component<{ class?: string }>;
  accent: string;
}

export interface AcquireDashboardProps {
  /** Handler when an action card is clicked */
  onAction: (action: AcquireAction) => void;
  /** Open settings */
  onSettings: () => void;
  /** Open help */
  onHelp: () => void;
  /** Open command palette */
  onCommandPalette: () => void;
  /** Open a project */
  onOpenProject: () => void;
  /** Create a new project */
  onNewProject: () => void;
  /** Project name if one is loaded */
  projectName: Accessor<string | undefined>;
  /** Whether a project is loaded */
  hasProject: Accessor<boolean>;
  /** Number of evidence files discovered */
  evidenceCount: Accessor<number>;
  /** Whether running in portable mode */
  isPortable: () => boolean;
  /** Portable mode configuration */
  portableConfig: () => PortableConfig | null;
  /** Quick verify — opens file picker and navigates to verify with selected files */
  onQuickVerify?: () => void;
  /** Navigate to the evidence collection view */
  onViewCollection?: (collectionId: string) => void;
  /** Called when drives are loaded/refreshed — allows parent to access drive data */
  onDrivesLoaded?: (drives: DriveInfo[]) => void;
  /** Called when system stats are loaded — allows parent to access system info */
  onSystemStatsLoaded?: (stats: SystemStats) => void;
  /** Previously collected system stats (restored on remount) */
  initialSystemStats?: SystemStats | null;
  /** Previously collected drives (restored on remount) */
  initialDrives?: DriveInfo[];
}

// =============================================================================
// Constants
// =============================================================================

/** All action cards keyed by ID */
const ALL_CARDS: Record<AcquireAction, ActionCard> = {
  browse: {
    id: "browse",
    title: "Browse Evidence",
    description: "Open and explore E01, AD1, L01, and archive containers",
    icon: HiOutlineArchiveBox,
    accent: "text-purple-400",
  },
  triage: {
    id: "triage",
    title: "Quick Triage",
    description: "Scan for key artifacts, browser data, and credentials",
    icon: HiOutlineShieldExclamation,
    accent: "text-red-400",
  },
  memory: {
    id: "memory",
    title: "Memory Capture",
    description: "Capture live RAM from a running system for volatile data analysis",
    icon: HiOutlineCpuChip,
    accent: "text-orange-400",
  },
  physical: {
    id: "physical",
    title: "Create Disk Image",
    description: "Create an E01 forensic image from a drive with built-in hash verification",
    icon: HiOutlineCircleStack,
    accent: "text-blue-400",
  },
  logical: {
    id: "logical",
    title: "Create Logical Image",
    description: "Package files and folders into an L01 logical evidence container",
    icon: HiOutlineFolder,
    accent: "text-emerald-400",
  },
  export: {
    id: "export",
    title: "Export Files",
    description: "Archive files to 7z or copy to a folder with hash manifests",
    icon: HiOutlineArrowUpTray,
    accent: "text-amber-400",
  },
  verify: {
    id: "verify",
    title: "Verify Hashes",
    description: "Compute and verify hashes of files, folders, and forensic containers (E01, L01, AD1, AFF4)",
    icon: HiOutlineFingerPrint,
    accent: "text-rose-400",
  },
  collection: {
    id: "collection",
    title: "Evidence Collection",
    description: "Record collection details and maintain chain of custody",
    icon: HiOutlineFolderOpen,
    accent: "text-cyan-400",
  },
};

/** Workflow phases that guide the user through an evidence collection workflow */
interface WorkflowPhase {
  step: number;
  title: string;
  subtitle: string;
  cardIds: AcquireAction[];
  /** When true, renders project setup UI instead of action cards */
  projectPhase?: boolean;
  /** When true, renders system/drive identification UI above the cards */
  identifyPhase?: boolean;
}

const WORKFLOW_PHASES: WorkflowPhase[] = [
  {
    step: 1,
    title: "Project",
    subtitle: "Create or open a project to organize and document your work",
    cardIds: [],
    projectPhase: true,
  },
  {
    step: 2,
    title: "Identify",
    subtitle: "Survey the system, identify drives and volatile data sources",
    cardIds: ["browse", "triage", "memory"],
    identifyPhase: true,
  },
  {
    step: 3,
    title: "Acquire & Package",
    subtitle: "Create forensic images, export files, and archive with integrity manifests",
    cardIds: ["physical", "logical", "export"],
  },
  {
    step: 4,
    title: "Verify & Document",
    subtitle: "Validate hash integrity and record chain of custody",
    cardIds: ["verify", "collection"],
  },
];

// =============================================================================
// Helpers
// =============================================================================

/** Format bytes for system-level display (e.g. 16 GB RAM) */
const formatSystemBytes = (bytes: number): string => {
  if (bytes <= 0) return "0 B";
  const gb = bytes / (1024 * 1024 * 1024);
  if (gb >= 1) return `${gb.toFixed(gb >= 10 ? 0 : 1)} GB`;
  const mb = bytes / (1024 * 1024);
  return `${mb.toFixed(0)} MB`;
};

const formatUptime = (secs: number): string => {
  const days = Math.floor(secs / 86400);
  const hours = Math.floor((secs % 86400) / 3600);
  const mins = Math.floor((secs % 3600) / 60);
  const parts: string[] = [];
  if (days > 0) parts.push(`${days}d`);
  if (hours > 0) parts.push(`${hours}h`);
  parts.push(`${mins}m`);
  return parts.join(" ");
};

/** Compact inline drive card for the identify phase */
const DriveCard: Component<{ drive: DriveInfo; onAction: (action: AcquireAction) => void }> = (props) => {
  const d = () => props.drive;
  const usedPct = () => {
    const total = d().totalBytes;
    if (!total) return 0;
    return Math.min(100, ((d().usedBytes || 0) / total) * 100);
  };
  const kindLabel = () => d().kind === "SSD" ? "SSD" : d().kind === "HDD" ? "HDD" : "";
  const locationLabel = () => d().isRemovable ? "Portable" : d().isSystemDisk ? "Internal" : "External";
  const locationClass = () => d().isRemovable ? "badge-portable" : d().isSystemDisk ? "badge-internal" : "badge-external";
  const DriveIcon = d().isRemovable
    ? HiOutlineCircleStack
    : d().isSystemDisk
      ? HiOutlineComputerDesktop
      : HiOutlineServer;

  return (
    <button
      class="acquire-drive-tile"
      onClick={() => props.onAction("physical")}
      title={`Image ${d().mountPoint || d().devicePath}`}
    >
      <div class="acquire-drive-tile-header">
        <DriveIcon class="w-4 h-4 text-txt-muted shrink-0" />
        <span class="acquire-drive-name truncate">{d().name || d().devicePath}</span>
        <Show when={d().isReadOnly}>
          <span class="acquire-drive-badge badge-ro">RO</span>
        </Show>
        <span class={`acquire-drive-badge ${locationClass()}`}>{locationLabel()}</span>
        <Show when={d().connectionType}>
          <span class="acquire-drive-badge badge-connection">{d().connectionType}</span>
        </Show>
        <Show when={kindLabel()}>
          <span class="acquire-drive-badge badge-kind">{kindLabel()}</span>
        </Show>
      </div>
      {/* Hardware identification — model / vendor / serial */}
      <Show when={d().model || d().vendor}>
        <div class="acquire-drive-hw">
          <Show when={d().vendor}>
            <span class="acquire-drive-vendor">{d().vendor}</span>
          </Show>
          <Show when={d().model}>
            <span class="acquire-drive-model truncate">{d().model}</span>
          </Show>
        </div>
      </Show>
      <Show when={d().serial}>
        <div class="acquire-drive-serial" title={`S/N: ${d().serial}`}>
          S/N: <span class="font-mono">{d().serial}</span>
        </div>
      </Show>
      <div class="acquire-drive-tile-meta">
        <span class="text-txt-muted truncate">{d().mountPoint || "—"}</span>
        <Show when={d().fileSystem}>
          <span class="acquire-drive-fs">{d().fileSystem}</span>
        </Show>
      </div>
      <div class="acquire-drive-capacity">
        <div class="acquire-drive-capacity-track">
          <div
            class="acquire-drive-capacity-fill"
            classList={{
              "bg-success": usedPct() < 75,
              "bg-warning": usedPct() >= 75 && usedPct() < 90,
              "bg-error": usedPct() >= 90,
            }}
            style={{ width: `${usedPct()}%` }}
          />
        </div>
        <span class="acquire-drive-size">
          {formatDriveSize(d().availableBytes || 0)} free / {formatDriveSize(d().totalBytes || 0)}
        </span>
      </div>
    </button>
  );
};

// =============================================================================
// Component
// =============================================================================

const AcquireDashboard: Component<AcquireDashboardProps> = (props) => {
  // Seed local state from previously-collected data (persisted in parent)
  const hasInitialData = !!(props.initialSystemStats && props.initialDrives?.length);
  const [drives, setDrives] = createSignal<DriveInfo[]>(props.initialDrives ?? []);
  const [identifyState, setIdentifyState] = createSignal<"idle" | "loading" | "done">(hasInitialData ? "done" : "idle");
  const [systemStats, setSystemStats] = createSignal<SystemStats | null>(props.initialSystemStats ?? null);
  const [showSystemDetail, setShowSystemDetail] = createSignal(hasInitialData);

  // Run system identification — collects drives + system stats once
  const runIdentify = async () => {
    setIdentifyState("loading");
    try {
      const [list, stats] = await Promise.all([
        listDrives(),
        invoke<SystemStats>("get_system_stats"),
      ]);
      setDrives(list);
      setSystemStats(stats);
      setIdentifyState("done");
      setShowSystemDetail(true);
      props.onDrivesLoaded?.(list);
      props.onSystemStatsLoaded?.(stats);
    } catch {
      setIdentifyState("idle");
    }
  };

  // Separate system drives from external/removable
  const externalDrives = createMemo(() => drives().filter(d => !d.isSystemDisk));
  const systemDrives = createMemo(() => drives().filter(d => d.isSystemDisk));

  const freeSpaceGb = () => {
    const cfg = props.portableConfig();
    if (!cfg) return null;
    return (cfg.freeSpaceBytes / (1024 * 1024 * 1024)).toFixed(1);
  };

  return (
    <div class="acquire-dashboard">
      {/* Top bar — branding + project info + utility buttons */}
      <header class="acquire-topbar" role="banner">
        <div class="flex items-center gap-3">
          <span class="text-sm font-semibold text-txt tracking-tight">{APP_NAME}</span>
          <Show when={props.isPortable()}>
            <div class="flex items-center gap-1 px-2 py-0.5 bg-success/10 border border-success/20 rounded" title={`Portable mode — data stored on removable media\n${props.portableConfig()?.dataDir || ""}`}>
              <HiOutlineServer class="w-icon-sm h-icon-sm text-success" />
              <span class="text-2xs font-medium text-success">Portable</span>
              <Show when={freeSpaceGb() !== null}>
                <span class="text-2xs text-success/60">{freeSpaceGb()} GB</span>
              </Show>
            </div>
          </Show>
          <Show when={props.hasProject()}>
            <div class="flex items-center gap-1.5 px-2 py-0.5 bg-accent/10 border border-accent/20 rounded">
              <span class="text-xs font-medium text-accent truncate max-w-[200px]">
                {props.projectName()}
              </span>
              <Show when={props.evidenceCount() > 0}>
                <span class="text-2xs text-accent/70">{props.evidenceCount()} files</span>
              </Show>
            </div>
          </Show>
        </div>

        <div class="flex items-center gap-0.5">
          <Show when={!props.hasProject()}>
            <button class="btn-sm gap-1 mr-2" onClick={props.onNewProject}>New Project</button>
            <button class="btn-sm btn-ghost gap-1 mr-2" onClick={props.onOpenProject}>Open</button>
          </Show>
          <button class="icon-btn-sm" onClick={props.onCommandPalette} title="Command Palette (⌘K)" aria-label="Command Palette">
            <HiOutlineCommandLine class="w-4 h-4" />
          </button>
          <button class="icon-btn-sm" onClick={props.onSettings} title="Settings" aria-label="Settings">
            <HiOutlineCog6Tooth class="w-4 h-4" />
          </button>
          <button class="icon-btn-sm" onClick={props.onHelp} title="Help" aria-label="Help">
            <HiOutlineQuestionMarkCircle class="w-4 h-4" />
          </button>
        </div>
      </header>

      {/* Low space warning */}
      <Show when={props.isPortable() && props.portableConfig()?.hasSufficientSpace === false}>
        <div class="flex items-center gap-2 mx-4 mt-1 px-3 py-1.5 bg-warning/10 border border-warning/20 rounded text-warning text-xs">
          <HiOutlineExclamationTriangle class="w-4 h-4 shrink-0" />
          <span>Low disk space — {freeSpaceGb()} GB remaining</span>
        </div>
      </Show>

      {/* Two-column layout: workflow + collection summary */}
      <div class="acquire-dashboard-layout">
      <div class="acquire-dashboard-main">

      {/* Workflow Phases */}
      <div class="acquire-workflow">
        <For each={WORKFLOW_PHASES}>
          {(phase, index) => (
            <div class="acquire-phase">
              <Show when={index() > 0}>
                <div class="acquire-phase-connector" aria-hidden="true" />
              </Show>
              <div class="acquire-phase-header">
                <div class="acquire-phase-number">{phase.step}</div>
                <div class="acquire-phase-info">
                  <span class="acquire-phase-title">{phase.title}</span>
                  <p class="acquire-phase-subtitle">{phase.subtitle}</p>
                </div>
              </div>

              {/* Phase 1: Project — new/open or loaded state */}
              <Show when={phase.projectPhase}>
                <div class="acquire-phase-cards">
                  <Show when={props.hasProject()} fallback={
                    <>
                      <button class="acquire-card" onClick={props.onNewProject}>
                        <div class="acquire-card-icon text-emerald-400">
                          <HiOutlineFolderPlus class="w-5 h-5" />
                        </div>
                        <div class="acquire-card-content">
                          <h3 class="acquire-card-title">New Project</h3>
                          <p class="acquire-card-desc">Create a forensic project to track acquisitions</p>
                        </div>
                      </button>
                      <button class="acquire-card" onClick={props.onOpenProject}>
                        <div class="acquire-card-icon text-blue-400">
                          <HiOutlineDocumentCheck class="w-5 h-5" />
                        </div>
                        <div class="acquire-card-content">
                          <h3 class="acquire-card-title">Open Project</h3>
                          <p class="acquire-card-desc">Resume an existing forensic project</p>
                        </div>
                      </button>
                    </>
                  }>
                    <div class="acquire-card-loaded">
                      <div class="acquire-card-icon text-success">
                        <HiOutlineCheckCircle class="w-5 h-5" />
                      </div>
                      <div class="acquire-card-content">
                        <h3 class="acquire-card-title truncate">{props.projectName()}</h3>
                        <p class="acquire-card-desc">
                          Project loaded
                          <Show when={props.evidenceCount() > 0}>
                            {" · "}{props.evidenceCount()} evidence files
                          </Show>
                        </p>
                      </div>
                    </div>
                  </Show>
                </div>
              </Show>

              {/* Phase 2: Identify — system info + drives + action cards */}
              <Show when={phase.identifyPhase}>
                <div class="acquire-identify-section">
                  {/* Identify button / completed status */}
                  <Show when={identifyState() === "done"} fallback={
                    <button
                      class="acquire-identify-btn"
                      onClick={runIdentify}
                      disabled={identifyState() === "loading"}
                    >
                      <Show when={identifyState() === "loading"} fallback={
                        <>
                          <HiOutlineComputerDesktop class="w-4 h-4" />
                          Identify System
                        </>
                      }>
                        <HiOutlineArrowPath class="w-4 h-4 animate-spin" />
                        Scanning…
                      </Show>
                    </button>
                  }>
                    <div class="acquire-system-bar" onClick={() => setShowSystemDetail(v => !v)} role="button" tabIndex={0}>
                      <div class="acquire-system-stat">
                        <HiOutlineCheckCircle class="w-3.5 h-3.5 text-success" />
                        <span>System identified</span>
                        <span class="text-txt-muted/60">&middot;</span>
                        <Show when={systemStats()!.osName}>
                          <span>{systemStats()!.osName} {systemStats()!.osVersion}</span>
                          <span class="text-txt-muted/60">&middot;</span>
                        </Show>
                        <HiOutlineCpuChip class="w-3.5 h-3.5 text-txt-muted" />
                        <span>{systemStats()!.cpuCores} cores</span>
                        <span class="text-txt-muted/60">&middot;</span>
                        <span>{formatSystemBytes(systemStats()!.memoryTotal)} RAM</span>
                        <span class="text-txt-muted/60">&middot;</span>
                        <HiOutlineCircleStack class="w-3.5 h-3.5 text-txt-muted" />
                        <span>{drives().length} volume{drives().length !== 1 ? "s" : ""}</span>
                        <Show when={showSystemDetail()} fallback={
                          <HiOutlineChevronDown class="w-3.5 h-3.5 text-txt-muted ml-auto" />
                        }>
                          <HiOutlineChevronUp class="w-3.5 h-3.5 text-txt-muted ml-auto" />
                        </Show>
                        <button
                          class="acquire-identify-rerun"
                          onClick={(e) => { e.stopPropagation(); runIdentify(); }}
                          title="Re-scan system"
                        >
                          <HiOutlineArrowPath class="w-3 h-3" />
                        </button>
                      </div>
                    </div>

                    {/* Expandable system detail panel */}
                    <Show when={showSystemDetail()}>
                      <div class="acquire-system-detail">
                        <div class="acquire-system-detail-grid">
                          <Show when={systemStats()!.hostname}>
                            <div class="acquire-detail-item">
                              <span class="acquire-detail-label">Hostname</span>
                              <span class="acquire-detail-value font-mono">{systemStats()!.hostname}</span>
                            </div>
                          </Show>
                          <Show when={systemStats()!.osName}>
                            <div class="acquire-detail-item">
                              <span class="acquire-detail-label">Operating System</span>
                              <span class="acquire-detail-value">
                                {systemStats()!.longOsVersion || `${systemStats()!.osName} ${systemStats()!.osVersion}`}
                              </span>
                            </div>
                          </Show>
                          <Show when={systemStats()!.kernelVersion}>
                            <div class="acquire-detail-item">
                              <span class="acquire-detail-label">Kernel Version</span>
                              <span class="acquire-detail-value font-mono">{systemStats()!.kernelVersion}</span>
                            </div>
                          </Show>
                          <Show when={systemStats()!.timezone}>
                            <div class="acquire-detail-item">
                              <span class="acquire-detail-label">Timezone</span>
                              <span class="acquire-detail-value font-mono">{systemStats()!.timezone}</span>
                            </div>
                          </Show>
                          <Show when={systemStats()!.cpuBrand}>
                            <div class="acquire-detail-item">
                              <span class="acquire-detail-label">Processor</span>
                              <span class="acquire-detail-value">{systemStats()!.cpuBrand}</span>
                            </div>
                          </Show>
                          <Show when={systemStats()!.cpuVendor}>
                            <div class="acquire-detail-item">
                              <span class="acquire-detail-label">CPU Vendor</span>
                              <span class="acquire-detail-value">{systemStats()!.cpuVendor}</span>
                            </div>
                          </Show>
                          <div class="acquire-detail-item">
                            <span class="acquire-detail-label">CPU Cores</span>
                            <span class="acquire-detail-value">
                              {systemStats()!.cpuCores} logical
                              <Show when={systemStats()!.physicalCores > 0}>
                                {" "}/ {systemStats()!.physicalCores} physical
                              </Show>
                            </span>
                          </div>
                          <Show when={systemStats()!.cpuFrequencyMhz > 0}>
                            <div class="acquire-detail-item">
                              <span class="acquire-detail-label">CPU Frequency</span>
                              <span class="acquire-detail-value">
                                {systemStats()!.cpuFrequencyMhz >= 1000
                                  ? `${(systemStats()!.cpuFrequencyMhz / 1000).toFixed(2)} GHz`
                                  : `${systemStats()!.cpuFrequencyMhz} MHz`}
                              </span>
                            </div>
                          </Show>
                          <Show when={systemStats()!.cpuArch}>
                            <div class="acquire-detail-item">
                              <span class="acquire-detail-label">Architecture</span>
                              <span class="acquire-detail-value font-mono">{systemStats()!.cpuArch}</span>
                            </div>
                          </Show>
                          <div class="acquire-detail-item">
                            <span class="acquire-detail-label">Total Memory</span>
                            <span class="acquire-detail-value">{formatSystemBytes(systemStats()!.memoryTotal)}</span>
                          </div>
                          <div class="acquire-detail-item">
                            <span class="acquire-detail-label">Memory Used</span>
                            <span class="acquire-detail-value">{formatSystemBytes(systemStats()!.memoryUsed)} ({systemStats()!.memoryPercent.toFixed(1)}%)</span>
                          </div>
                          <Show when={systemStats()!.totalSwap > 0}>
                            <div class="acquire-detail-item">
                              <span class="acquire-detail-label">Swap Memory</span>
                              <span class="acquire-detail-value">{formatSystemBytes(systemStats()!.usedSwap)} / {formatSystemBytes(systemStats()!.totalSwap)}</span>
                            </div>
                          </Show>
                          <Show when={systemStats()!.uptimeSecs > 0}>
                            <div class="acquire-detail-item">
                              <span class="acquire-detail-label">System Uptime</span>
                              <span class="acquire-detail-value">{formatUptime(systemStats()!.uptimeSecs)}</span>
                            </div>
                          </Show>
                          <Show when={systemStats()!.bootTimeEpoch > 0}>
                            <div class="acquire-detail-item">
                              <span class="acquire-detail-label">Last Boot</span>
                              <span class="acquire-detail-value font-mono">{new Date(systemStats()!.bootTimeEpoch * 1000).toLocaleString()}</span>
                            </div>
                          </Show>
                          <div class="acquire-detail-item">
                            <span class="acquire-detail-label">Volumes Detected</span>
                            <span class="acquire-detail-value">{drives().length}</span>
                          </div>
                        </div>

                        {/* Network Interfaces section */}
                        <Show when={systemStats()!.networkInterfaces.length > 0}>
                          <div class="acquire-network-section">
                            <div class="acquire-network-header">
                              <HiOutlineGlobeAlt class="w-3.5 h-3.5 text-txt-muted" />
                              <span>Network Interfaces</span>
                            </div>
                            <div class="acquire-network-list">
                              <For each={systemStats()!.networkInterfaces}>
                                {(iface) => (
                                  <div class="acquire-network-item">
                                    <div class="acquire-network-name">{iface.name}</div>
                                    <div class="acquire-network-detail">
                                      <span class="acquire-detail-label">MAC</span>
                                      <span class="acquire-detail-value font-mono">{iface.macAddress}</span>
                                    </div>
                                    <Show when={iface.ipAddresses.length > 0}>
                                      <div class="acquire-network-detail">
                                        <span class="acquire-detail-label">IP</span>
                                        <span class="acquire-detail-value font-mono">{iface.ipAddresses.join(", ")}</span>
                                      </div>
                                    </Show>
                                  </div>
                                )}
                              </For>
                            </div>
                          </div>
                        </Show>
                      </div>
                    </Show>

                    {/* Drive groups */}
                    <Show when={drives().length > 0} fallback={
                      <div class="acquire-drives-empty">
                        <HiOutlineCircleStack class="w-6 h-6 opacity-30" />
                        <span>No drives detected</span>
                      </div>
                    }>
                      <Show when={externalDrives().length > 0}>
                        <div class="acquire-drives-group">
                          <span class="acquire-drives-label">External & Removable</span>
                          <div class="acquire-drives-grid">
                            <For each={externalDrives()}>
                              {(drive) => <DriveCard drive={drive} onAction={props.onAction} />}
                            </For>
                          </div>
                        </div>
                      </Show>
                      <Show when={systemDrives().length > 0}>
                        <div class="acquire-drives-group">
                          <span class="acquire-drives-label">System Volumes</span>
                          <div class="acquire-drives-grid">
                            <For each={systemDrives()}>
                              {(drive) => <DriveCard drive={drive} onAction={props.onAction} />}
                            </For>
                          </div>
                        </div>
                      </Show>
                    </Show>
                  </Show>

                  {/* Action cards: Browse Evidence, Triage, Memory */}
                  <div class="acquire-phase-cards">
                    <For each={phase.cardIds}>
                      {(cardId) => {
                        const card = ALL_CARDS[cardId];
                        const Icon = card.icon;
                        return (
                          <button
                            class="acquire-card"
                            onClick={() => props.onAction(card.id)}
                          >
                            <div class={`acquire-card-icon ${card.accent}`}>
                              <Icon class="w-5 h-5" />
                            </div>
                            <div class="acquire-card-content">
                              <h3 class="acquire-card-title">{card.title}</h3>
                              <p class="acquire-card-desc">{card.description}</p>
                            </div>
                          </button>
                        );
                      }}
                    </For>
                  </div>
                </div>
              </Show>

              {/* Default card phases (Acquire & Package, Verify & Document) */}
              <Show when={!phase.projectPhase && !phase.identifyPhase}>
                <div class="acquire-phase-cards">
                  <For each={phase.cardIds}>
                    {(cardId) => {
                      const card = ALL_CARDS[cardId];
                      const Icon = card.icon;
                      return (
                        <button
                          class="acquire-card"
                          onClick={() => props.onAction(card.id)}
                        >
                          <div class={`acquire-card-icon ${card.accent}`}>
                            <Icon class="w-5 h-5" />
                          </div>
                          <div class="acquire-card-content">
                            <h3 class="acquire-card-title">{card.title}</h3>
                            <p class="acquire-card-desc">{card.description}</p>
                          </div>
                        </button>
                      );
                    }}
                  </For>
                </div>
              </Show>
            </div>
          )}
        </For>
        <Show when={props.onQuickVerify}>
          <button
            class="acquire-quick-action"
            onClick={() => props.onQuickVerify?.()}
          >
            <div class="acquire-card-icon text-accent">
              <HiOutlineFingerPrint class="w-5 h-5" />
            </div>
            <div class="acquire-card-content">
              <h3 class="acquire-card-title">Quick Hash File</h3>
              <p class="acquire-card-desc">Select files to immediately compute and verify hashes</p>
            </div>
          </button>
        </Show>
      </div>

      {/* Recent Acquisitions */}
      <Show when={props.hasProject()}>
        <RecentAcquisitions />
      </Show>

      </div>{/* end acquire-dashboard-main */}

      {/* Right panel — Evidence Collection Summary */}
      <Show when={props.hasProject()}>
        <AcquireCollectionSummary
          hasProject={props.hasProject}
          onViewCollection={props.onViewCollection}
          onNewCollection={() => props.onAction("collection")}
        />
      </Show>

      </div>{/* end acquire-dashboard-layout */}
    </div>
  );
};

// =============================================================================
// Recent Acquisitions sub-component
// =============================================================================

const EXPORT_TYPE_LABELS: Record<string, string> = {
  e01: "E01 Image",
  l01: "L01 Image",
  archive: "7z Archive",
  "file-export": "File Export",
  memory: "Memory Dump",
  triage: "Triage",
};

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
  const val = bytes / Math.pow(1024, i);
  return `${val < 10 ? val.toFixed(1) : Math.round(val)} ${units[i]}`;
}

function formatRelativeTime(iso: string): string {
  const diff = Date.now() - new Date(iso).getTime();
  const mins = Math.floor(diff / 60000);
  if (mins < 1) return "just now";
  if (mins < 60) return `${mins}m ago`;
  const hrs = Math.floor(mins / 60);
  if (hrs < 24) return `${hrs}h ago`;
  const days = Math.floor(hrs / 24);
  return `${days}d ago`;
}

const RecentAcquisitions: Component = () => {
  const [records, setRecords] = createSignal<DbExportRecord[]>([]);

  onMount(async () => {
    try {
      const data = await getExportHistory(10);
      setRecords(data);
    } catch {
      // DB may not be open yet — silently ignore
    }
  });

  return (
    <Show when={records().length > 0}>
      <div class="px-6 pb-6">
        <div class="flex items-center gap-2 mb-2">
          <HiOutlineClock class="w-3.5 h-3.5 text-txt-muted" />
          <span class="text-2xs font-medium text-txt-muted uppercase tracking-wider">Recent Activity</span>
        </div>
        <div class="space-y-1">
          <For each={records()}>
            {(rec) => {
              const isOk = rec.status === "completed";
              const isFail = rec.status === "failed";
              const label = EXPORT_TYPE_LABELS[rec.exportType] ?? rec.exportType;
              const dest = rec.destination?.split("/").pop() || rec.destination;

              return (
                <div class="flex items-center gap-3 px-3 py-2 rounded-lg bg-bg-secondary hover:bg-bg-hover transition-colors text-xs">
                  <Show when={isOk} fallback={
                    <Show when={isFail} fallback={
                      <div class="w-1.5 h-1.5 rounded-full bg-accent animate-pulse-slow shrink-0" />
                    }>
                      <HiOutlineXCircle class="w-4 h-4 text-error shrink-0" />
                    </Show>
                  }>
                    <HiOutlineCheckCircle class="w-4 h-4 text-success shrink-0" />
                  </Show>

                  <span class="font-medium text-txt shrink-0">{label}</span>
                  <span class="text-txt-muted truncate flex-1" title={rec.destination}>{dest}</span>

                  <Show when={rec.totalBytes > 0}>
                    <span class="text-txt-muted shrink-0">{formatBytes(rec.totalBytes)}</span>
                  </Show>
                  <Show when={rec.totalFiles > 0}>
                    <span class="text-txt-muted shrink-0">{rec.totalFiles} files</span>
                  </Show>

                  <span class="text-txt-muted/60 shrink-0">{formatRelativeTime(rec.startedAt)}</span>

                  <Show when={isFail && rec.error}>
                    <span class="text-error truncate max-w-[120px]" title={rec.error}>{rec.error}</span>
                  </Show>
                </div>
              );
            }}
          </For>
        </div>
      </div>
    </Show>
  );
};

export default AcquireDashboard;
