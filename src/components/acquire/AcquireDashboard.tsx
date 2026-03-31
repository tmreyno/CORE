// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * AcquireDashboard — Unified forensic acquisition workflow.
 *
 * Three phases, all in one surface:
 *   1. SELECT  — Check drives, toggle memory/triage, configure per-source format
 *   2. PROCESS — Tasks execute sequentially (memory → triage → physical → logical → export)
 *   3. REVIEW  — Completed tasks expand inline evidence collection forms
 */

import {
  Component,
  Show,
  For,
  createSignal,
  createMemo,
  createEffect,
  onCleanup,
  lazy,
  Suspense,
  type Accessor,
} from "solid-js";
import { CoreProgressBar } from "@core-suite/icons";
import {
  HiOutlineCircleStack,
  HiOutlineFolder,
  HiOutlineFolderOpen,
  HiOutlineFingerPrint,
  HiOutlineFolderPlus,
  HiOutlineDocumentCheck,
  HiOutlineCog6Tooth,
  HiOutlineQuestionMarkCircle,
  HiOutlineCommandLine,
  HiOutlineServer,
  HiOutlineExclamationTriangle,
  HiOutlineCpuChip,
  HiOutlineShieldExclamation,
  HiOutlineCheckCircle,
  HiOutlineXCircle,
  HiOutlinePlayCircle,
  HiOutlineStopCircle,
  HiOutlineTrash,
  HiOutlineComputerDesktop,
  HiOutlineArchiveBox,
  ChevronDownIcon,
  ChevronRightIcon,
} from "../icons";
import { APP_NAME } from "../../utils/edition";
import type { PortableConfig } from "../../api/portable";
import { checkFullDiskAccess, openFullDiskAccessSettings } from "../../api/fda";
import type { FullDiskAccessStatus } from "../../api/fda";
import type { DriveInfo } from "../../api/drives";
import type { SystemStats } from "../../hooks";
import type { DiscoveredFile, ContainerInfo } from "../../types";
import { getTriageProfiles } from "../../api/triage";
import { DriveTreeBrowser } from "../export-panel/DriveTreeBrowser";
import { RecentProjectsList } from "../RecentProjectsList";
import {
  useAcquisitionRunner,
  type AcquisitionSessionWriter,
} from "../../hooks/acquire/useAcquisitionRunner";
import type {
  AcquisitionTask,
  AcquisitionTaskType,
  AcquisitionTaskConfig,
  AcquisitionPhase,
} from "../../hooks/acquire/types";
import { ACQUISITION_PRIORITY, defaultConfig } from "../../hooks/acquire/types";
import { getPreference } from "../preferences";

// Lazy-load the evidence collection form for inline review
const EvidenceCollectionPanel = lazy(() =>
  import("../EvidenceCollectionPanel").then((m) => ({
    default: m.EvidenceCollectionPanel,
  })),
);

// =============================================================================
// Types
// =============================================================================

/** Actions that navigate away from the dashboard (used by AcquireLayout) */
export type AcquireAction =
  | "identify"
  | "physical"
  | "logical"
  | "export"
  | "browse"
  | "verify"
  | "collection"
  | "memory"
  | "triage";

/** A source selected for acquisition */
interface SelectedSource {
  path: string;
  label: string;
  type: AcquisitionTaskType;
  config: AcquisitionTaskConfig;
}

export interface AcquireDashboardProps {
  onAction: (action: AcquireAction) => void;
  onSettings: () => void;
  onHelp: () => void;
  onCommandPalette: () => void;
  onOpenProject: () => void;
  onOpenRecentProject?: (path: string) => void;
  onNewProject: () => void;
  projectName: Accessor<string | undefined>;
  hasProject: Accessor<boolean>;
  evidenceCount: Accessor<number>;
  isPortable: () => boolean;
  portableConfig: () => PortableConfig | null;
  onQuickVerify?: () => void;
  onViewCollection?: (collectionId: string) => void;
  /** Evidence item folder for output destination */
  evidenceItemFolder?: Accessor<string>;
  /** Fallback destination from project exports_path */
  initialDestination?: string;
  /** System context for evidence records */
  initialSystemStats?: SystemStats | null;
  initialDrives?: DriveInfo[];
  /** Case metadata */
  caseNumber?: Accessor<string | undefined>;
  examinerName?: Accessor<string | undefined>;
  /** Evidence data for inline collection panels */
  discoveredFiles?: Accessor<DiscoveredFile[]>;
  fileInfoMap?: Accessor<Map<string, ContainerInfo>>;
  /** Called when acquisition complete - register output */
  onExportComplete?: (destination: string) => void;
  /** Session writer for Acquire edition (replaces DB sync) */
  sessionWriter?: AcquisitionSessionWriter;
}

// =============================================================================
// Formatters
// =============================================================================

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(i > 1 ? 1 : 0)} ${sizes[i]}`;
}

function formatDuration(ms: number): string {
  const s = Math.floor(ms / 1000);
  if (s < 60) return `${s}s`;
  const m = Math.floor(s / 60);
  const rem = s % 60;
  if (m < 60) return `${m}m ${rem}s`;
  const h = Math.floor(m / 60);
  return `${h}h ${m % 60}m`;
}

function basename(path: string): string {
  const parts = path.replace(/\\/g, "/").split("/");
  return parts[parts.length - 1] || path;
}

// =============================================================================
// Component
// =============================================================================

const AcquireDashboard: Component<AcquireDashboardProps> = (props) => {
  // ── Selection state ──
  const [selectedSources, setSelectedSources] = createSignal<SelectedSource[]>([]);
  const [includeMemory, setIncludeMemory] = createSignal(false);
  const [includeTriage, setIncludeTriage] = createSignal(false);
  const [triageSecrets, setTriageSecrets] = createSignal(true);
  const [triageCategories, setTriageCategories] = createSignal<string[]>([]);

  // ── Full Disk Access detection (macOS) ──
  // Only checks after a project is loaded (system/drive checks happen during
  // project setup). Re-checks periodically + on window focus so the banner
  // updates after the user grants FDA in System Settings.
  // IMPORTANT: The backend TCC probe can block for several seconds per path
  // when access is denied, so we defer the initial check (2s) and use a
  // longer poll interval (30s) to avoid saturating the thread pool.
  const [fdaStatus, setFdaStatus] = createSignal<FullDiskAccessStatus | null>(null);
  const [fdaJustGranted, setFdaJustGranted] = createSignal(false);
  let fdaInterval: ReturnType<typeof setInterval> | undefined;
  let fdaCheckInFlight = false;

  function recheckFda() {
    if (fdaCheckInFlight) return; // prevent concurrent probes
    fdaCheckInFlight = true;
    checkFullDiskAccess()
      .then((status) => {
        const prev = fdaStatus();
        // Detect transition: was denied → now granted
        if (prev && !prev.hasFullDiskAccess && status.hasFullDiskAccess) {
          setFdaJustGranted(true);
          setTimeout(() => setFdaJustGranted(false), 4000);
        }
        setFdaStatus(status);
      })
      .catch(() => {}) // non-macOS or backend unavailable
      .finally(() => { fdaCheckInFlight = false; });
  }

  function handleFdaFocus() { recheckFda(); }

  // Start FDA checks + listeners when a project becomes active
  createEffect(() => {
    if (!props.hasProject()) return;
    // Defer initial check 2s so it doesn't compete with session creation
    // or project setup operations on the Tauri thread pool
    setTimeout(() => recheckFda(), 2000);
    // Poll every 30s while FDA is not granted (each probe can block for
    // seconds when denied, so aggressive polling wastes thread pool time)
    if (!fdaInterval) {
      fdaInterval = setInterval(() => {
        const s = fdaStatus();
        if (!s || !s.hasFullDiskAccess) recheckFda();
      }, 30_000);
    }
    window.addEventListener("focus", handleFdaFocus);
  });

  onCleanup(() => {
    if (fdaInterval) clearInterval(fdaInterval);
    window.removeEventListener("focus", handleFdaFocus);
  });

  // ── Manual destination override (browse button) ──
  const [manualDestination, setManualDestination] = createSignal("");

  // ── Acquisition runner ──
  const effectiveDestination = () =>
    manualDestination() || props.evidenceItemFolder?.() || props.initialDestination || "";

  const runner = useAcquisitionRunner({
    destination: effectiveDestination,
    caseNumber: props.caseNumber,
    examiner: props.examinerName,
    hostname: props.initialSystemStats?.hostname,
    systemModel: props.initialSystemStats?.systemModel,
    systemSerialNumber: props.initialSystemStats?.systemSerialNumber,
    systemManufacturer: props.initialSystemStats?.systemManufacturer,
    osName: props.initialSystemStats?.osName,
    osVersion: props.initialSystemStats?.osVersion,
    sessionWriter: props.sessionWriter,
    onTaskComplete: (task) => {
      if (task.result?.outputPath) {
        props.onExportComplete?.(task.result.outputPath);
      }
    },
  });

  // Load triage profiles when triage is first enabled so we have categories
  let triageProfilesLoaded = false;
  createEffect(() => {
    if (includeTriage() && !triageProfilesLoaded) {
      triageProfilesLoaded = true;
      getTriageProfiles()
        .then(([profiles]) => {
          const full = profiles.find((p) => p.id === "full_triage");
          if (full && full.categories.length > 0) {
            setTriageCategories(full.categories);
          } else if (profiles.length > 0 && profiles[0].categories.length > 0) {
            setTriageCategories(profiles[0].categories);
          }
        })
        .catch(() => {
          // Profiles unavailable — categories stay empty, backend will log warning
        });
    }
  });

  const isIdle = () => runner.phase() === "idle";
  const isRunning = () => runner.phase() === "running";
  const isComplete = () => runner.phase() === "complete";
  const canStart = createMemo(() => {
    if (!props.hasProject()) return false;
    if (isRunning()) return false;
    if (!effectiveDestination()) return false;
    return (
      selectedSources().length > 0 || includeMemory() || includeTriage()
    );
  });

  // ── Drive selection ──
  const selectedPaths = createMemo(() => {
    return new Set(selectedSources().map((s) => s.path));
  });

  /** Build a config from the user's acquisition preferences */
  function prefConfig(): { type: AcquisitionTaskType; config: AcquisitionTaskConfig } {
    const fmt = getPreference("defaultAcquisitionFormat") as AcquisitionTaskType;
    const base = defaultConfig(fmt);
    return {
      type: fmt,
      config: {
        ...base,
        compression: getPreference("defaultAcquisitionCompression") as AcquisitionTaskConfig["compression"],
        segmentSize: getPreference("defaultAcquisitionSegmentMb"),
        hashMd5: getPreference("defaultAcquisitionHashMd5"),
        hashSha1: getPreference("defaultAcquisitionHashSha1"),
        hashSha256: getPreference("defaultAcquisitionHashSha256"),
      },
    };
  }

  function handleDriveSelect(path: string) {
    setSelectedSources((prev) => {
      const exists = prev.find((s) => s.path === path);
      if (exists) return prev.filter((s) => s.path !== path);
      const pref = prefConfig();
      return [...prev, {
        path,
        label: basename(path),
        type: pref.type,
        config: pref.config,
      }];
    });
  }

  function updateSourceType(path: string, type: AcquisitionTaskType) {
    setSelectedSources((prev) =>
      prev.map((s) =>
        s.path === path
          ? { ...s, type, config: { ...defaultConfig(type), ...s.config } }
          : s,
      ),
    );
  }

  function updateSourceConfig(path: string, updates: Partial<AcquisitionTaskConfig>) {
    setSelectedSources((prev) =>
      prev.map((s) =>
        s.path === path ? { ...s, config: { ...s.config, ...updates } } : s,
      ),
    );
  }

  function removeSource(path: string) {
    setSelectedSources((prev) => prev.filter((s) => s.path !== path));
  }

  // ── Start acquisition ──
  function handleStart() {
    if (!canStart()) return;
    runner.clearTasks();

    // Add memory capture if checked
    if (includeMemory()) {
      runner.addTask("memory", "system", "Live Memory", {
        caseNumber: props.caseNumber?.(),
        examiner: props.examinerName?.(),
      });
    }

    // Add triage if checked
    if (includeTriage()) {
      runner.addTask("triage", "/", "System Triage", {
        triageCategories: triageCategories(),
        scanSecrets: triageSecrets(),
        caseNumber: props.caseNumber?.(),
        examiner: props.examinerName?.(),
      });
    }

    // Add each selected source
    for (const src of selectedSources()) {
      runner.addTask(src.type, src.path, src.label, {
        ...src.config,
        caseNumber: props.caseNumber?.(),
        examiner: props.examinerName?.(),
      });
    }

    runner.start();
  }

  function handleReset() {
    runner.clearTasks();
    setSelectedSources([]);
    setIncludeMemory(false);
    setIncludeTriage(false);
  }

  async function handleBrowseDestination() {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({ directory: true, title: "Select output folder" });
      if (selected && typeof selected === "string") {
        setManualDestination(selected);
      }
    } catch { /* user cancelled */ }
  }

  // Portable mode helpers
  const freeSpaceGb = () => {
    const cfg = props.portableConfig();
    if (!cfg) return null;
    return (cfg.freeSpaceBytes / (1024 * 1024 * 1024)).toFixed(1);
  };

  // ── Task counts ──
  const completedCount = createMemo(() =>
    runner.tasks().filter((t) => t.status === "completed").length,
  );
  const failedCount = createMemo(() =>
    runner.tasks().filter((t) => t.status === "failed").length,
  );
  const totalTasks = createMemo(() => runner.tasks().length);

  return (
    <div class="flex flex-col flex-1 min-h-0 overflow-hidden">
      {/* ── Top bar ─────────────────────────────────────────────────── */}
      <header class="flex items-center justify-between px-2.5 py-1 border-b border-border bg-bg-secondary shrink-0" role="banner">
        <div class="flex items-center gap-2">
          <span class="text-xs font-semibold text-txt tracking-tight">{APP_NAME}</span>
          <Show when={props.isPortable()}>
            <div
              class="flex items-center gap-1 px-2 py-0.5 bg-success/10 border border-success/20 rounded"
              title={`Portable mode — data on removable media\n${props.portableConfig()?.dataDir || ""}`}
            >
              <HiOutlineServer class="w-icon-sm h-icon-sm text-success" />
              <span class="text-2xs font-medium text-success">Portable</span>
              <Show when={freeSpaceGb() !== null}>
                <span class="text-2xs text-success/60">{freeSpaceGb()} GB</span>
              </Show>
            </div>
          </Show>
          <Show when={props.hasProject()}>
            <div class="flex items-center gap-1 px-1.5 py-0.5 bg-accent/10 border border-accent/20 rounded">
              <span class="text-2xs font-medium text-accent truncate max-w-[160px]">
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
            <button class="btn-sm gap-1 mr-1" onClick={props.onNewProject}>New Project</button>
            <button class="btn-sm btn-ghost gap-1 mr-1" onClick={props.onOpenProject}>Open</button>
          </Show>
          <button class="icon-btn-sm" onClick={props.onCommandPalette} title="Command Palette" aria-label="Command Palette">
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
        <div class="flex items-center gap-1.5 mx-3 mt-0.5 px-2 py-1 bg-warning/10 border border-warning/20 rounded text-warning text-2xs">
          <HiOutlineExclamationTriangle class="w-3.5 h-3.5 shrink-0" />
          <span>Low disk space — {freeSpaceGb()} GB remaining</span>
        </div>
      </Show>

      {/* Full Disk Access status (macOS) */}
      <Show when={fdaJustGranted()}>
        <div class="flex items-center gap-2 mx-3 mt-1 px-3 py-1.5 bg-success/10 border border-success/20 rounded text-success text-xs">
          <HiOutlineCheckCircle class="w-4 h-4 shrink-0" />
          <span>Full Disk Access granted — all protected directories are accessible.</span>
        </div>
      </Show>
      <Show when={fdaStatus() && !fdaStatus()!.hasFullDiskAccess && !fdaJustGranted()}>
        <div class="flex items-center gap-2 mx-3 mt-1 px-3 py-1.5 bg-warning/10 border border-warning/20 rounded text-warning text-xs">
          <HiOutlineShieldExclamation class="w-4 h-4 shrink-0" />
          <span class="flex-1">
            Full Disk Access not granted — triage and acquisition will skip protected directories
            ({fdaStatus()!.blockedPaths.length} blocked).
          </span>
          <button
            class="text-xs text-warning hover:text-warning/80 underline underline-offset-2 px-1 py-0 shrink-0"
            onClick={() => recheckFda()}
          >
            Re-check
          </button>
          <button
            class="text-xs text-warning hover:text-warning/80 underline underline-offset-2 px-1 py-0 shrink-0"
            onClick={() => openFullDiskAccessSettings().catch(() => {})}
          >
            Open Settings
          </button>
        </div>
      </Show>

      {/* ── No project state ────────────────────────────────────────── */}
      <Show when={!props.hasProject()}>
        <div class="flex flex-col items-center justify-center flex-1 gap-3 p-6">
          <div class="flex gap-3">
            <button class="card-interactive flex items-start gap-3 p-3 w-56" onClick={props.onNewProject}>
              <div class="w-8 h-8 rounded-lg flex items-center justify-center bg-accent-soft shrink-0 text-emerald-400">
                <HiOutlineFolderPlus class="w-5 h-5" />
              </div>
              <div class="flex flex-col gap-0.5">
                <h3 class="text-sm font-semibold text-txt">New Project</h3>
                <p class="text-xs text-txt-muted">Create a forensic project to track acquisitions</p>
              </div>
            </button>
            <button class="card-interactive flex items-start gap-3 p-3 w-56" onClick={props.onOpenProject}>
              <div class="w-8 h-8 rounded-lg flex items-center justify-center bg-accent-soft shrink-0 text-blue-400">
                <HiOutlineDocumentCheck class="w-5 h-5" />
              </div>
              <div class="flex flex-col gap-0.5">
                <h3 class="text-sm font-semibold text-txt">Open Project</h3>
                <p class="text-xs text-txt-muted">Resume an existing forensic project</p>
              </div>
            </button>
          </div>
          <Show when={props.onOpenRecentProject}>
            <div class="mt-2 max-w-md w-full">
              <RecentProjectsList
                onOpenProject={props.onOpenRecentProject!}
                maxItems={5}
                compact
              />
            </div>
          </Show>
        </div>
      </Show>

      {/* ── Main content (project loaded) ───────────────────────────── */}
      <Show when={props.hasProject()}>
        <div class="flex flex-col flex-1 min-h-0 overflow-hidden">
          {/* Compact system summary bar */}
          <Show when={props.initialSystemStats}>
            <div class="flex items-center gap-1.5 px-2.5 py-1 border-b border-border bg-bg-secondary shrink-0 text-2xs text-txt-muted">
              <HiOutlineComputerDesktop class="w-3.5 h-3.5 text-accent shrink-0" />
              <Show when={props.initialSystemStats!.systemModel}>
                <span class="text-2xs text-txt-muted">{props.initialSystemStats!.systemModel}</span>
              </Show>
              <Show when={props.initialSystemStats!.hostname}>
                <span class="text-2xs text-txt-muted">{props.initialSystemStats!.hostname}</span>
              </Show>
              <Show when={props.initialSystemStats!.osName}>
                <span class="text-2xs text-txt-muted">{props.initialSystemStats!.osName}</span>
              </Show>
              <span class="text-2xs text-txt-muted">{props.initialSystemStats!.cpuCores} cores</span>
              <Show when={props.initialSystemStats!.memoryTotal > 0}>
                <span class="text-2xs text-txt-muted">
                  {(props.initialSystemStats!.memoryTotal / (1024 * 1024 * 1024)).toFixed(
                    props.initialSystemStats!.memoryTotal / (1024 * 1024 * 1024) >= 10 ? 0 : 1,
                  )} GB RAM
                </span>
              </Show>
              <span class="text-2xs text-txt-muted">{props.initialDrives?.length ?? 0} volumes</span>
              <div class="flex-1" />
              {/* Inline quick actions */}
              <button class="icon-btn-sm" onClick={() => props.onAction("identify")} title="Identify System">
                <HiOutlineCircleStack class="w-3.5 h-3.5" />
              </button>
              <button class="icon-btn-sm" onClick={() => props.onAction("browse")} title="Browse Evidence">
                <HiOutlineArchiveBox class="w-3.5 h-3.5" />
              </button>
              <Show when={props.onQuickVerify}>
                <button class="icon-btn-sm" onClick={() => props.onQuickVerify?.()} title="Quick Hash File">
                  <HiOutlineFingerPrint class="w-3.5 h-3.5" />
                </button>
              </Show>
              <button class="icon-btn-sm" onClick={() => props.onAction("collection")} title="Evidence Collection">
                <HiOutlineFolder class="w-3.5 h-3.5" />
              </button>
              <button class="icon-btn-sm" onClick={() => props.onAction("verify")} title="Verify Hashes">
                <HiOutlineCheckCircle class="w-3.5 h-3.5" />
              </button>
            </div>
          </Show>

          <div class="flex-1 min-h-0 overflow-y-auto flex flex-col">
            {/* ── Selection Phase ──────────────────────────────────── */}
            <Show when={isIdle()}>
              <SelectionPhase
                selectedSources={selectedSources}
                includeMemory={includeMemory}
                setIncludeMemory={setIncludeMemory}
                includeTriage={includeTriage}
                setIncludeTriage={setIncludeTriage}
                triageSecrets={triageSecrets}
                setTriageSecrets={setTriageSecrets}
                onRemoveSource={removeSource}
                onUpdateType={updateSourceType}
                onUpdateConfig={updateSourceConfig}
                canStart={canStart}
                onStart={handleStart}
                onBrowse={() => props.onAction("browse")}
                onIdentify={() => props.onAction("identify")}
                onQuickVerify={props.onQuickVerify}
                destination={effectiveDestination}
                onBrowseDestination={handleBrowseDestination}
                onSelectSource={handleDriveSelect}
                selectedPaths={selectedPaths}
                initialDrives={props.initialDrives}
              />
            </Show>

            {/* ── Process Phase (running or complete) ──────────────── */}
            <Show when={isRunning() || isComplete()}>
              <ProcessPhase
                tasks={runner.tasks}
                phase={runner.phase}
                currentTaskId={runner.currentTaskId}
                onCancel={runner.cancel}
                onToggleCollection={runner.toggleCollectionExpanded}
                onReset={handleReset}
                completedCount={completedCount}
                failedCount={failedCount}
                totalTasks={totalTasks}
                caseNumber={props.caseNumber}
                projectName={props.projectName}
                examinerName={props.examinerName}
                discoveredFiles={props.discoveredFiles}
                fileInfoMap={props.fileInfoMap}
              />
            </Show>
          </div>
        </div>
      </Show>
    </div>
  );
};

// =============================================================================
// Selection Phase
// =============================================================================

interface SelectionPhaseProps {
  selectedSources: Accessor<SelectedSource[]>;
  includeMemory: Accessor<boolean>;
  setIncludeMemory: (v: boolean) => void;
  includeTriage: Accessor<boolean>;
  setIncludeTriage: (v: boolean) => void;
  triageSecrets: Accessor<boolean>;
  setTriageSecrets: (v: boolean) => void;
  onRemoveSource: (path: string) => void;
  onUpdateType: (path: string, type: AcquisitionTaskType) => void;
  onUpdateConfig: (path: string, updates: Partial<AcquisitionTaskConfig>) => void;
  canStart: Accessor<boolean>;
  onStart: () => void;
  onBrowse: () => void;
  onIdentify: () => void;
  onQuickVerify?: () => void;
  destination: Accessor<string>;
  onBrowseDestination: () => void;
  onSelectSource: (path: string) => void;
  selectedPaths: Accessor<Set<string>>;
  initialDrives?: DriveInfo[];
}

const SelectionPhase: Component<SelectionPhaseProps> = (p) => {
  const [showDriveBrowser, setShowDriveBrowser] = createSignal(true);
  const [expandedConfig, setExpandedConfig] = createSignal<string | null>(null);
  const itemCount = createMemo(() => {
    let count = p.selectedSources().length;
    if (p.includeMemory()) count++;
    if (p.includeTriage()) count++;
    return count;
  });

  return (
    <div class="flex flex-col gap-2 p-2.5">
      <div class="flex items-center justify-between gap-2 pb-1">
        <h2 class="text-xs font-semibold text-txt uppercase tracking-wider">Select Evidence to Capture</h2>
        <div class="flex items-center gap-1.5">
          <Show
            when={p.destination()}
            fallback={
              <span class="text-2xs text-warning">No output folder set</span>
            }
          >
            <span class="text-2xs text-txt-muted font-mono truncate max-w-[200px]" title={p.destination()}>
              {basename(p.destination())}
            </span>
          </Show>
          <button
            class="icon-btn-sm"
            onClick={p.onBrowseDestination}
            title="Choose output folder"
          >
            <HiOutlineFolderOpen class="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      {/* Volatile data toggles */}
      <div class="flex flex-col gap-1.5">
        <span class="text-2xs uppercase tracking-wider font-medium text-txt-muted">Volatile Data</span>
        <div class="flex flex-col gap-1">
          <label class="flex items-center gap-2 px-2 py-1.5 rounded hover:bg-bg-hover cursor-pointer">
            <input
              type="checkbox"
              checked={p.includeMemory()}
              onChange={(e) => p.setIncludeMemory(e.currentTarget.checked)}
              class="w-3.5 h-3.5 rounded border border-border accent-accent shrink-0"
            />
            <HiOutlineCpuChip class="w-4 h-4 text-orange-400" />
            <div class="flex flex-col">
              <span class="text-xs font-medium text-txt">Memory Capture</span>
              <span class="text-2xs text-txt-muted">Capture live RAM — highest priority</span>
            </div>
          </label>
        </div>
        <div class="flex flex-col gap-1">
          <label class="flex items-center gap-2 px-2 py-1.5 rounded hover:bg-bg-hover cursor-pointer">
            <input
              type="checkbox"
              checked={p.includeTriage()}
              onChange={(e) => p.setIncludeTriage(e.currentTarget.checked)}
              class="w-3.5 h-3.5 rounded border border-border accent-accent shrink-0"
            />
            <HiOutlineShieldExclamation class="w-4 h-4 text-red-400" />
            <div class="flex flex-col flex-1">
              <span class="text-xs font-medium text-txt">Quick Triage</span>
              <span class="text-2xs text-txt-muted">Scan key artifacts, browser data, credentials</span>
            </div>
          </label>
          <Show when={p.includeTriage()}>
            <label class="flex items-center gap-2 pl-8 py-0.5 cursor-pointer">
              <input
                type="checkbox"
                checked={p.triageSecrets()}
                onChange={(e) => p.setTriageSecrets(e.currentTarget.checked)}
                class="w-3.5 h-3.5 rounded border border-border accent-accent shrink-0"
              />
              <span class="text-2xs text-txt-muted">Scan for secrets</span>
            </label>
          </Show>
        </div>
      </div>

      {/* Inline Drive Browser */}
      <div class="flex flex-col border border-border rounded-lg overflow-hidden">
        <button
          class="flex items-center gap-2 px-2.5 py-1.5 bg-bg-secondary cursor-pointer hover:bg-bg-hover"
          onClick={() => setShowDriveBrowser((v) => !v)}
        >
          <HiOutlineCircleStack class="w-3.5 h-3.5 text-accent shrink-0" />
          <span class="text-xs font-medium text-txt">Select Sources</span>
          <Show when={p.selectedSources().length > 0}>
            <span class="text-2xs text-accent">({p.selectedSources().length})</span>
          </Show>
          <div class="flex-1" />
          <Show when={showDriveBrowser()} fallback={<ChevronRightIcon class="w-3 h-3 text-txt-muted" />}>
            <ChevronDownIcon class="w-3 h-3 text-txt-muted" />
          </Show>
        </button>
        <Show when={showDriveBrowser()}>
          <div class="max-h-[280px] overflow-y-auto">
            <DriveTreeBrowser
              onSelectSource={p.onSelectSource}
              selectedPaths={() => p.selectedPaths()}
              fillHeight={false}
              initialDrives={p.initialDrives}
            />
          </div>
        </Show>
      </div>

      {/* Selected sources */}
      <div class="flex flex-col gap-1.5">
        <Show when={p.selectedSources().length > 0}>
          <span class="text-2xs uppercase tracking-wider font-medium text-txt-muted">
            Queued Sources
            <span class="text-accent ml-1">({p.selectedSources().length})</span>
          </span>
        </Show>
        <Show
          when={p.selectedSources().length > 0}
          fallback={null}
        >
          <div class="flex flex-col gap-1">
            <For each={p.selectedSources()}>
              {(src) => {
                const isExpanded = () => expandedConfig() === src.path;
                const fmt = () => src.type === "physical" ? (src.config.format || "e01") : src.type === "logical" ? "l01" : src.type === "aff4" ? "aff4" : "7z";
                const showCompression = () => fmt() === "e01" || fmt() === "l01" || fmt() === "aff4";
                const showSegment = () => fmt() === "e01" || fmt() === "raw" || fmt() === "l01" || fmt() === "7z";
                const showHashToggles = () => true;
                return (
                <div class="flex flex-col border border-border rounded-lg overflow-hidden">
                  <div class="flex items-center gap-2 px-2 py-1.5 bg-bg-secondary">
                    <button
                      class="shrink-0 p-0"
                      onClick={() => setExpandedConfig(isExpanded() ? null : src.path)}
                      title="Configure options"
                    >
                      <Show when={isExpanded()} fallback={<ChevronRightIcon class="w-3 h-3 text-txt-muted" />}>
                        <ChevronDownIcon class="w-3 h-3 text-txt-muted" />
                      </Show>
                    </button>
                    <HiOutlineCircleStack class="w-3.5 h-3.5 text-accent shrink-0" />
                    <span class="text-xs font-medium text-txt truncate flex-1" title={src.path}>
                      {src.label}
                    </span>
                    <select
                      class="text-2xs px-1.5 py-0.5 bg-bg border border-border rounded cursor-pointer"
                      value={
                        src.type === "physical"
                          ? (src.config.format || "e01")
                          : src.type === "logical"
                            ? "l01"
                            : src.type === "aff4"
                              ? "aff4"
                              : "7z"
                      }
                      onChange={(e) => {
                        const val = e.currentTarget.value;
                        if (val === "e01" || val === "raw") {
                          p.onUpdateType(src.path, "physical");
                          p.onUpdateConfig(src.path, { format: val });
                        } else if (val === "l01") {
                          p.onUpdateType(src.path, "logical");
                          p.onUpdateConfig(src.path, { format: "l01" });
                        } else if (val === "aff4") {
                          p.onUpdateType(src.path, "aff4");
                          p.onUpdateConfig(src.path, { format: "aff4" });
                        } else if (val === "7z") {
                          p.onUpdateType(src.path, "export");
                          p.onUpdateConfig(src.path, { format: "7z" });
                        }
                      }}
                    >
                      <option value="e01">E01</option>
                      <option value="raw">Raw (.dd)</option>
                      <option value="l01">L01</option>
                      <option value="aff4">AFF4</option>
                      <option value="7z">7z</option>
                    </select>
                    <button
                      class="icon-btn-sm text-txt-muted hover:text-error"
                      onClick={() => p.onRemoveSource(src.path)}
                      title="Remove"
                    >
                      <HiOutlineTrash class="w-3.5 h-3.5" />
                    </button>
                  </div>
                  <div class="px-2 py-1 border-t border-border bg-bg">
                    <span class="font-mono text-compact text-txt-muted truncate">{src.path}</span>
                  </div>
                  {/* Per-source config panel */}
                  <Show when={isExpanded()}>
                    <div class="px-3 py-2 border-t border-border bg-bg space-y-2">
                      {/* Compression */}
                      <Show when={showCompression()}>
                        <div class="flex items-center gap-2">
                          <span class="text-2xs text-txt-muted w-20 shrink-0">Compression</span>
                          <select
                            class="text-2xs px-1.5 py-0.5 bg-bg-secondary border border-border rounded flex-1"
                            value={src.config.compression || "none"}
                            onChange={(e) => p.onUpdateConfig(src.path, { compression: e.currentTarget.value as "none" | "fast" | "best" })}
                          >
                            <option value="none">None</option>
                            <option value="fast">Fast</option>
                            <option value="best">Best</option>
                          </select>
                        </div>
                      </Show>
                      {/* Segment size */}
                      <Show when={showSegment()}>
                        <div class="flex items-center gap-2">
                          <span class="text-2xs text-txt-muted w-20 shrink-0">Split Size</span>
                          <select
                            class="text-2xs px-1.5 py-0.5 bg-bg-secondary border border-border rounded flex-1"
                            value={String((src.config.segmentSize || 0) / (1024 * 1024))}
                            onChange={(e) => p.onUpdateConfig(src.path, { segmentSize: Number(e.currentTarget.value) * 1024 * 1024 })}
                          >
                            <option value="0">No splitting</option>
                            <option value="650">650 MB (CD)</option>
                            <option value="2048">2 GB (default)</option>
                            <option value="4096">4 GB (FAT32)</option>
                            <option value="4700">4.7 GB (DVD)</option>
                          </select>
                        </div>
                      </Show>
                      {/* Hash algorithms */}
                      <Show when={showHashToggles()}>
                        <div class="flex items-center gap-2">
                          <span class="text-2xs text-txt-muted w-20 shrink-0">Hash</span>
                          <div class="flex items-center gap-3">
                            <label class="flex items-center gap-1 cursor-pointer">
                              <input type="checkbox" checked={src.config.hashMd5 ?? false}
                                onChange={(e) => p.onUpdateConfig(src.path, { hashMd5: e.currentTarget.checked })}
                                class="w-3 h-3 rounded border border-border accent-accent" />
                              <span class="text-2xs text-txt">MD5</span>
                            </label>
                            <label class="flex items-center gap-1 cursor-pointer">
                              <input type="checkbox" checked={src.config.hashSha1 ?? false}
                                onChange={(e) => p.onUpdateConfig(src.path, { hashSha1: e.currentTarget.checked })}
                                class="w-3 h-3 rounded border border-border accent-accent" />
                              <span class="text-2xs text-txt">SHA-1</span>
                            </label>
                            <label class="flex items-center gap-1 cursor-pointer">
                              <input type="checkbox" checked={src.config.hashSha256 ?? false}
                                onChange={(e) => p.onUpdateConfig(src.path, { hashSha256: e.currentTarget.checked })}
                                class="w-3 h-3 rounded border border-border accent-accent" />
                              <span class="text-2xs text-txt">SHA-256</span>
                            </label>
                          </div>
                        </div>
                      </Show>
                    </div>
                  </Show>
                </div>
              );}}
            </For>
          </div>
        </Show>
      </div>

      {/* Start button */}
      <div class="flex items-center justify-between gap-2 px-3 py-1.5 border-t border-border bg-bg-secondary shrink-0 mt-auto">
        <div class="flex items-center gap-2 text-xs text-txt-muted">
          <Show when={itemCount() > 0}>
            <span>{itemCount()} task{itemCount() !== 1 ? "s" : ""} queued</span>
          </Show>
        </div>
        <button
          class="btn btn-primary gap-1.5"
          disabled={!p.canStart()}
          onClick={p.onStart}
        >
          <HiOutlinePlayCircle class="w-4 h-4" />
          Start Acquisition
        </button>
      </div>
    </div>
  );
};

// =============================================================================
// Process Phase
// =============================================================================

interface ProcessPhaseProps {
  tasks: Accessor<AcquisitionTask[]>;
  phase: Accessor<AcquisitionPhase>;
  currentTaskId: Accessor<string | null>;
  onCancel: () => Promise<void>;
  onToggleCollection: (id: string) => void;
  onReset: () => void;
  completedCount: Accessor<number>;
  failedCount: Accessor<number>;
  totalTasks: Accessor<number>;
  caseNumber?: Accessor<string | undefined>;
  projectName?: Accessor<string | undefined>;
  examinerName?: Accessor<string | undefined>;
  discoveredFiles?: Accessor<DiscoveredFile[]>;
  fileInfoMap?: Accessor<Map<string, ContainerInfo>>;
}

const ProcessPhase: Component<ProcessPhaseProps> = (p) => {
  const isRunning = () => p.phase() === "running";
  const isComplete = () => p.phase() === "complete";

  // Sort tasks by priority for display
  const sortedTasks = createMemo(() =>
    [...p.tasks()].sort(
      (a, b) => ACQUISITION_PRIORITY[a.type] - ACQUISITION_PRIORITY[b.type],
    ),
  );

  return (
    <div class="flex flex-col gap-2 p-2.5">
      {/* Header */}
      <div class="flex items-center justify-between gap-2">
        <div class="flex items-center gap-1.5">
          <Show when={isRunning()}>
            <div class="w-1.5 h-1.5 rounded-full bg-accent animate-pulse-slow" />
            <span class="text-xs font-semibold text-txt">Acquiring...</span>
          </Show>
          <Show when={isComplete()}>
            <HiOutlineCheckCircle class="w-4 h-4 text-success" />
            <span class="text-xs font-semibold text-txt">
              Complete — {p.completedCount()}/{p.totalTasks()} succeeded
            </span>
          </Show>
        </div>
        <div class="flex items-center gap-1.5">
          <Show when={isRunning()}>
            <button
              class="btn-sm text-error gap-1"
              onClick={() => p.onCancel()}
            >
              <HiOutlineStopCircle class="w-4 h-4" />
              Cancel
            </button>
          </Show>
          <Show when={isComplete()}>
            <button class="btn-sm gap-1" onClick={p.onReset}>
              New Acquisition
            </button>
          </Show>
        </div>
      </div>

      {/* Overall progress */}
      <Show when={isRunning()}>
        <div class="h-1 bg-bg-secondary rounded-full overflow-hidden mx-2.5">
          <div
            class="h-full bg-accent rounded-full transition-all"
            style={{
              width: `${(p.completedCount() / Math.max(p.totalTasks(), 1)) * 100}%`,
            }}
          />
        </div>
      </Show>

      {/* Task list */}
      <div class="flex flex-col gap-1.5">
        <For each={sortedTasks()}>
          {(task) => (
            <TaskCard
              task={task}
              isCurrent={p.currentTaskId() === task.id}
              onToggleCollection={() => p.onToggleCollection(task.id)}
              caseNumber={p.caseNumber}
              projectName={p.projectName}
              examinerName={p.examinerName}
              discoveredFiles={p.discoveredFiles}
              fileInfoMap={p.fileInfoMap}
            />
          )}
        </For>
      </div>
    </div>
  );
};

// =============================================================================
// Task Card
// =============================================================================

interface TaskCardProps {
  task: AcquisitionTask;
  isCurrent: boolean;
  onToggleCollection: () => void;
  caseNumber?: Accessor<string | undefined>;
  projectName?: Accessor<string | undefined>;
  examinerName?: Accessor<string | undefined>;
  discoveredFiles?: Accessor<DiscoveredFile[]>;
  fileInfoMap?: Accessor<Map<string, ContainerInfo>>;
}

const TASK_TYPE_ICONS: Record<AcquisitionTaskType, Component<{ class?: string }>> = {
  memory: HiOutlineCpuChip,
  triage: HiOutlineShieldExclamation,
  physical: HiOutlineCircleStack,
  aff4: HiOutlineCircleStack,
  logical: HiOutlineFolder,
  export: HiOutlineArchiveBox,
};

const TASK_TYPE_COLORS: Record<AcquisitionTaskType, string> = {
  memory: "text-orange-400",
  triage: "text-red-400",
  physical: "text-blue-400",
  aff4: "text-violet-400",
  logical: "text-emerald-400",
  export: "text-amber-400",
};

const TaskCard: Component<TaskCardProps> = (p) => {
  const Icon = TASK_TYPE_ICONS[p.task.type];
  const color = TASK_TYPE_COLORS[p.task.type];
  const isRunning = () => p.task.status === "running";
  const isCompleted = () => p.task.status === "completed";
  const isFailed = () => p.task.status === "failed";
  const isCancelled = () => p.task.status === "cancelled";
  const isPending = () => p.task.status === "pending";

  return (
    <div
      class="flex flex-col border border-border rounded-lg overflow-hidden"
      classList={{
        "border-accent/30 bg-accent/5": isRunning(),
        "border-success/30 bg-success/5": isCompleted(),
        "border-error/30 bg-error/5": isFailed(),
        "border-warning/30 bg-warning/5": isCancelled(),
      }}
    >
      {/* Task header */}
      <div class="flex items-center gap-2 px-2 py-1.5">
        <Icon class={`w-4 h-4 ${color} shrink-0`} />
        <span class="text-xs font-medium text-txt flex-1 truncate">{p.task.label}</span>
        <span class="text-2xs text-txt-muted truncate max-w-[160px]" title={p.task.source}>
          {p.task.sourceLabel}
        </span>
        {/* Status badge */}
        <Show when={isPending()}>
          <span class="text-2xs px-1.5 py-0.5 rounded font-medium bg-bg-secondary text-txt-muted">Pending</span>
        </Show>
        <Show when={isRunning()}>
          <span class="text-2xs px-1.5 py-0.5 rounded font-medium bg-accent/10 text-accent">Running</span>
        </Show>
        <Show when={isCompleted()}>
          <span class="text-2xs px-1.5 py-0.5 rounded font-medium bg-success/10 text-success">Done</span>
        </Show>
        <Show when={isFailed()}>
          <span class="text-2xs px-1.5 py-0.5 rounded font-medium bg-error/10 text-error">Failed</span>
        </Show>
        <Show when={isCancelled()}>
          <span class="text-2xs px-1.5 py-0.5 rounded font-medium bg-warning/10 text-warning">Cancelled</span>
        </Show>
      </div>

      {/* Progress bar for running tasks */}
      <Show when={isRunning() && p.task.progress}>
        <div class="flex flex-col gap-0.5 px-2 pb-1.5">
          <CoreProgressBar progress={p.task.progress!.percent} height={4} showSpinner={false} />
          <div class="flex items-center gap-2">
            <span class="text-2xs text-txt-muted">{p.task.progress!.percent.toFixed(1)}%</span>
            <Show when={p.task.progress!.phase}>
              <span class="text-2xs text-txt-muted">{p.task.progress!.phase}</span>
            </Show>
            <Show when={p.task.progress!.bytesProcessed > 0}>
              <span class="text-2xs text-txt-muted">{formatBytes(p.task.progress!.bytesProcessed)}</span>
            </Show>
          </div>
          <Show when={p.task.progress!.currentFile}>
            <span class="text-2xs text-txt-muted truncate">{p.task.progress!.currentFile}</span>
          </Show>
        </div>
      </Show>

      {/* Result summary for completed tasks */}
      <Show when={isCompleted() && p.task.result}>
        <div class="flex flex-col gap-0.5 px-2 pb-1.5 border-t border-border pt-1.5">
          <div class="flex items-center justify-between gap-2">
            <span class="text-2xs text-txt-muted">Output</span>
            <span class="text-2xs font-mono text-txt truncate" title={p.task.result!.outputPath}>
              {basename(p.task.result!.outputPath)}
            </span>
          </div>
          <div class="flex items-center justify-between gap-2">
            <span class="text-2xs text-txt-muted">Size</span>
            <span class="text-2xs text-txt">{formatBytes(p.task.result!.outputSize)}</span>
          </div>
          <div class="flex items-center justify-between gap-2">
            <span class="text-2xs text-txt-muted">Duration</span>
            <span class="text-2xs text-txt">{formatDuration(p.task.result!.durationMs)}</span>
          </div>
          <Show when={Object.keys(p.task.result!.hashes).length > 0}>
            <For each={Object.entries(p.task.result!.hashes)}>
              {([algo, hash]) => (
                <div class="flex items-center justify-between gap-2">
                  <span class="text-2xs text-txt-muted uppercase">{algo}</span>
                  <span class="text-compact font-mono text-txt truncate">{hash}</span>
                </div>
              )}
            </For>
          </Show>
        </div>

        {/* Collapsible evidence collection form */}
        <Show when={p.task.collectionId}>
          <button
            class="flex items-center gap-1.5 px-2 py-1 text-2xs text-accent cursor-pointer hover:bg-bg-hover"
            onClick={p.onToggleCollection}
          >
            <Show when={p.task.collectionExpanded} fallback={<ChevronRightIcon class="w-3 h-3" />}>
              <ChevronDownIcon class="w-3 h-3" />
            </Show>
            <span>Evidence Collection Form</span>
          </button>
          <Show when={p.task.collectionExpanded}>
            <div class="border-t border-border">
              <Suspense fallback={<div class="p-3 text-xs text-txt-muted">Loading form...</div>}>
                <EvidenceCollectionPanel
                  collectionId={p.task.collectionId}
                  caseNumber={p.caseNumber?.()}
                  projectName={p.projectName?.()}
                  examinerName={p.examinerName?.()}
                  discoveredFiles={p.discoveredFiles?.() ?? []}
                  fileInfoMap={p.fileInfoMap?.() ?? new Map()}
                />
              </Suspense>
            </div>
          </Show>
        </Show>
      </Show>

      {/* Error message */}
      <Show when={isFailed() && p.task.error}>
        <div class="flex items-center gap-1.5 px-2 py-1 bg-error/5">
          <HiOutlineXCircle class="w-3.5 h-3.5 text-error shrink-0" />
          <span class="text-2xs text-error truncate">{p.task.error}</span>
        </div>
      </Show>
    </div>
  );
};

export default AcquireDashboard;
