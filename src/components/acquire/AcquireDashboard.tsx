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
  createEffect,
  on,
  onMount,
  Accessor,
  type JSX,
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
  HiOutlineXMark,
  HiOutlineGlobeAlt,
  ChevronDownIcon,
  ChevronRightIcon,
} from "../icons";
import { APP_NAME } from "../../utils/edition";
import type { PortableConfig } from "../../api/portable";
import type { DbExportRecord } from "../../types/projectDb";
import { getExportHistory } from "../../api/exportHistory";
import { listDrives, formatDriveSize, type DriveInfo } from "../../api/drives";
import { listPhysicalDisks, type PhysicalDisk } from "../../api/device";
import type { SystemStats } from "../../hooks";
import {
  HiOutlineLockClosed,
} from "../icons";
import AcquireCollectionSummary from "./AcquireCollectionSummary";
import { DriveTreeBrowser } from "../export-panel/DriveTreeBrowser";
import { generateEvidenceFolderName } from "../../utils/evidenceNaming";
import { RecentProjectsList } from "../RecentProjectsList";
import { useToast } from "../Toast";

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
  /** Open a specific recent project by path */
  onOpenRecentProject?: (path: string) => void;
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
  /** Render function for inline-expanded card content (browse still navigates) */
  renderExpandedContent?: (action: AcquireAction, onCollapse: () => void) => JSX.Element;
  /** Evidence base path (project's evidence_path — where item folders go) */
  evidenceBasePath?: string;
  /** Called when the evidence item folder is created after identification */
  onEvidenceItemFolderCreated?: (folderPath: string) => void;
  /** Current username (for evidence folder naming) */
  currentUsername?: string;
}

// =============================================================================
// Constants
// =============================================================================

/** All action cards keyed by ID — ordered by forensic acquisition workflow */
const ALL_CARDS: Record<AcquireAction, ActionCard> = {
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
    description: "Capture live RAM before it's lost — volatile data must be collected first",
    icon: HiOutlineCpuChip,
    accent: "text-orange-400",
  },
  browse: {
    id: "browse",
    title: "Browse Evidence",
    description: "Open and explore E01, AD1, L01, and archive containers",
    icon: HiOutlineArchiveBox,
    accent: "text-purple-400",
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
    subtitle: "Assess the scene — triage artifacts, capture volatile memory, then survey drives",
    cardIds: ["triage", "memory", "browse"],
    identifyPhase: true,
  },
  {
    step: 3,
    title: "Acquire & Package",
    subtitle: "Image drives, collect files, and archive with integrity manifests",
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

// =============================================================================
// Types for physical-disk-grouped drive display
// =============================================================================

/** A physical disk with its child volumes grouped together */
interface DiskGroup {
  disk: PhysicalDisk | null; // null = ungrouped volumes (no parent info)
  volumes: DriveInfo[];
}

/** Build a hierarchical tree: physical disks → child volumes */
function buildDiskVolumeTree(disks: PhysicalDisk[], volumes: DriveInfo[]): DiskGroup[] {
  // Map parentDisk → volumes
  const byParent = new Map<string, DriveInfo[]>();
  const ungrouped: DriveInfo[] = [];
  for (const v of volumes) {
    if (v.parentDisk) {
      const list = byParent.get(v.parentDisk) || [];
      list.push(v);
      byParent.set(v.parentDisk, list);
    } else {
      ungrouped.push(v);
    }
  }

  const groups: DiskGroup[] = [];
  const usedParents = new Set<string>();

  // Match physical disks to their volumes
  for (const d of disks) {
    const vols = byParent.get(d.wholeDiskPath) || [];
    if (vols.length > 0) {
      groups.push({ disk: d, volumes: vols });
      usedParents.add(d.wholeDiskPath);
    }
  }

  // Any volumes whose parentDisk didn't match a PhysicalDisk
  for (const [parent, vols] of byParent) {
    if (!usedParents.has(parent)) {
      groups.push({ disk: null, volumes: vols });
    }
  }

  // Volumes with no parent at all
  if (ungrouped.length > 0) {
    groups.push({ disk: null, volumes: ungrouped });
  }

  return groups;
}

/** Expandable physical disk row — shows disk hardware summary; click to reveal volumes */
const PhysicalDiskRow: Component<{
  group: DiskGroup;
  expanded: boolean;
  onToggle: () => void;
  onAction: (action: AcquireAction) => void;
}> = (props) => {
  const disk = () => props.group.disk;
  const volumes = () => props.group.volumes;
  const totalSize = () => disk()?.sizeBytes || volumes().reduce((s, v) => s + (v.totalBytes || 0), 0);

  const diskLabel = () => {
    if (disk()) {
      const d = disk()!;
      const parts: string[] = [];
      if (d.vendor) parts.push(d.vendor);
      if (d.model) parts.push(d.model);
      return parts.join(" ") || d.wholeDiskPath;
    }
    // Fallback for ungrouped volumes (e.g. APFS synthesized container disks)
    const vols = volumes();
    if (vols.length > 0) {
      const v = vols[0];
      if (v.model) return v.model;
      if (v.vendor) return v.vendor;
      if (v.isSystemDisk) return "System Disk";
      if (v.parentDisk) return v.parentDisk;
    }
    return "Unknown Disk";
  };

  const locationBadge = () => {
    if (disk()?.isBootDisk) return { label: "System", cls: "badge-internal" };
    if (disk()?.isRemovable) return { label: "Removable", cls: "badge-portable" };
    if (disk()) return { label: "External", cls: "badge-external" };
    // Fallback for ungrouped volumes — use volume metadata
    const vols = volumes();
    if (vols.some(v => v.isSystemDisk)) return { label: "System", cls: "badge-internal" };
    if (vols.some(v => v.isRemovable)) return { label: "Removable", cls: "badge-portable" };
    return { label: "Internal", cls: "badge-internal" };
  };

  const DiskIcon = () => {
    if (disk()?.isRemovable) return HiOutlineCircleStack;
    if (disk()?.isBootDisk) return HiOutlineComputerDesktop;
    if (disk()) return HiOutlineServer;
    // Fallback for ungrouped volumes
    const vols = volumes();
    if (vols.some(v => v.isSystemDisk)) return HiOutlineComputerDesktop;
    if (vols.some(v => v.isRemovable)) return HiOutlineCircleStack;
    return HiOutlineServer;
  };

  return (
    <div class="acquire-disk-group">
      <button class="acquire-disk-header" onClick={props.onToggle}>
        <Show when={props.expanded} fallback={
          <ChevronRightIcon class="w-3 h-3 text-txt-muted shrink-0" />
        }>
          <ChevronDownIcon class="w-3 h-3 text-txt-muted shrink-0" />
        </Show>
        {(() => { const I = DiskIcon(); return <I class="w-4 h-4 text-txt-muted shrink-0" />; })()}
        <span class="acquire-disk-name truncate">{diskLabel()}</span>
        <span class={`acquire-drive-badge ${locationBadge().cls}`}>{locationBadge().label}</span>
        <Show when={disk()?.connectionType}>
          <span class="acquire-drive-badge badge-connection">{disk()!.connectionType}</span>
        </Show>
        <Show when={disk()?.mediaType && disk()!.mediaType !== "Unknown"}>
          <span class="acquire-drive-badge badge-kind">{disk()!.mediaType}</span>
        </Show>
        <Show when={volumes().some(v => v.isEncrypted)}>
          <HiOutlineLockClosed class="w-3 h-3 text-warning shrink-0" title="Encrypted volume(s)" />
        </Show>
        <span class="acquire-disk-size">{formatDriveSize(totalSize())}</span>
        <span class="acquire-disk-vol-count">{volumes().length} vol{volumes().length !== 1 ? "s" : ""}</span>
      </button>

      <Show when={props.expanded}>
        <div class="acquire-disk-volumes">
          {/* Disk hardware detail row */}
          <Show when={disk()}>
            <div class="acquire-disk-hw-detail">
              <Show when={disk()!.serial}>
                <div class="acquire-detail-item">
                  <span class="acquire-detail-label">Serial</span>
                  <span class="acquire-detail-value font-mono">{disk()!.serial}</span>
                </div>
              </Show>
              <Show when={disk()!.vendor}>
                <div class="acquire-detail-item">
                  <span class="acquire-detail-label">Vendor</span>
                  <span class="acquire-detail-value">{disk()!.vendor}</span>
                </div>
              </Show>
              <Show when={disk()!.model}>
                <div class="acquire-detail-item">
                  <span class="acquire-detail-label">Model</span>
                  <span class="acquire-detail-value">{disk()!.model}</span>
                </div>
              </Show>
              <Show when={volumes()[0]?.partitionScheme}>
                <div class="acquire-detail-item">
                  <span class="acquire-detail-label">Partition</span>
                  <span class="acquire-detail-value">{volumes()[0].partitionScheme}</span>
                </div>
              </Show>
              <Show when={disk()!.partitions.length > 0}>
                <div class="acquire-detail-item">
                  <span class="acquire-detail-label">Partitions</span>
                  <span class="acquire-detail-value">{disk()!.partitions.length}</span>
                </div>
              </Show>
            </div>
          </Show>

          {/* Volume rows */}
          <For each={volumes()}>
            {(vol) => <VolumeRow volume={vol} onAction={props.onAction} />}
          </For>
        </div>
      </Show>
    </div>
  );
};

/** Individual volume row nested under a physical disk */
const VolumeRow: Component<{ volume: DriveInfo; onAction: (action: AcquireAction) => void }> = (props) => {
  const v = () => props.volume;
  const usedPct = () => {
    const total = v().totalBytes;
    if (!total) return 0;
    return Math.min(100, ((v().usedBytes || 0) / total) * 100);
  };

  return (
    <div class="acquire-volume-row">
      <div class="acquire-volume-info">
        <div class="acquire-volume-primary">
          <span class="acquire-volume-mount truncate" title={v().mountPoint || v().devicePath}>
            {v().mountPoint || v().devicePath}
          </span>
          <Show when={v().name && v().name !== v().mountPoint}>
            <span class="acquire-volume-name truncate">({v().name})</span>
          </Show>
          <Show when={v().fileSystem}>
            <span class="acquire-drive-fs">{v().fileSystem}</span>
          </Show>
          <Show when={v().isReadOnly}>
            <span class="acquire-drive-badge badge-ro">RO</span>
          </Show>
          <Show when={v().isEncrypted}>
            <span class="acquire-drive-badge badge-encrypted" title={v().encryptionType || "Encrypted"}>
              <HiOutlineLockClosed class="w-2.5 h-2.5 inline-block" />
              {v().encryptionType || "Encrypted"}
            </span>
          </Show>
        </div>
        <div class="acquire-volume-capacity">
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
            {formatDriveSize(v().availableBytes || 0)} free / {formatDriveSize(v().totalBytes || 0)}
          </span>
        </div>
      </div>
      <div class="acquire-volume-actions">
        <button
          class="acquire-vol-action"
          onClick={() => props.onAction("physical")}
          title="Create disk image (E01)"
        >
          Image
        </button>
        <button
          class="acquire-vol-action"
          onClick={() => props.onAction("export")}
          title="Export files"
        >
          Export
        </button>
      </div>
    </div>
  );
};

// =============================================================================
// Component
// =============================================================================

const AcquireDashboard: Component<AcquireDashboardProps> = (props) => {
  const toast = useToast();
  // Seed local state from previously-collected data (persisted in parent)
  const hasInitialData = !!(props.initialSystemStats && props.initialDrives?.length);
  const [drives, setDrives] = createSignal<DriveInfo[]>(props.initialDrives ?? []);
  const [physicalDisks, setPhysicalDisks] = createSignal<PhysicalDisk[]>([]);
  const [identifyState, setIdentifyState] = createSignal<"idle" | "loading" | "done">(hasInitialData ? "done" : "idle");
  const [systemStats, setSystemStats] = createSignal<SystemStats | null>(props.initialSystemStats ?? null);
  const [showSystemDetail, setShowSystemDetail] = createSignal(hasInitialData);
  const [expandedDisks, setExpandedDisks] = createSignal<Set<string>>(new Set());
  const [expandedCard, setExpandedCard] = createSignal<AcquireAction | null>(null);
  const [selectedSources, setSelectedSources] = createSignal<Set<string>>(new Set());
  const [evidenceFolder, setEvidenceFolder] = createSignal<string>("");
  const [rightSections, setRightSections] = createSignal<Set<string>>(new Set(["system", "collections", "recent"]));

  const toggleRightSection = (section: string) => {
    setRightSections((prev) => {
      const next = new Set(prev);
      if (next.has(section)) next.delete(section);
      else next.add(section);
      return next;
    });
  };

  const handleSelectSource = (path: string) => {
    setSelectedSources((prev) => {
      const next = new Set(prev);
      if (next.has(path)) next.delete(path);
      else next.add(path);
      return next;
    });
  };

  /** Toggle inline expansion — "browse" always navigates away (needs full layout). */
  const toggleCard = (cardId: AcquireAction) => {
    if (cardId === "browse") {
      props.onAction(cardId);
      return;
    }
    if (!props.renderExpandedContent) {
      props.onAction(cardId);
      return;
    }
    setExpandedCard(prev => prev === cardId ? null : cardId);
  };

  // Run system identification — collects drives + system stats, creates
  // evidence item folder, and auto-opens the evidence collection form.
  const runIdentify = async () => {
    setIdentifyState("loading");
    try {
      const [list, stats, disks] = await Promise.all([
        listDrives(),
        invoke<SystemStats>("get_system_stats"),
        listPhysicalDisks().catch(() => [] as PhysicalDisk[]),
      ]);
      setDrives(list);
      setPhysicalDisks(disks);
      setSystemStats(stats);
      setIdentifyState("done");
      setShowSystemDetail(true);
      props.onDrivesLoaded?.(list);
      props.onSystemStatsLoaded?.(stats);

      // Create per-item evidence folder under the project's evidence path
      if (props.evidenceBasePath) {
        try {
          const folderName = generateEvidenceFolderName(
            props.projectName(),
            stats,
            props.currentUsername,
          );
          const folderPath = `${props.evidenceBasePath}/${folderName}`;
          await invoke("create_directory", { path: folderPath });
          setEvidenceFolder(folderPath);
          props.onEvidenceItemFolderCreated?.(folderPath);
          toast.success("Evidence Folder Created", folderPath.split("/").pop() || folderPath);
        } catch (e) {
          console.warn("Failed to create evidence item folder:", e);
        }
      }

      // Auto-open the evidence collection form inline
      setExpandedCard("collection");
    } catch {
      setIdentifyState("idle");
    }
  };

  // Auto-trigger system identification when a project is opened/created
  createEffect(on(
    () => props.hasProject(),
    (hasProject) => {
      if (hasProject && identifyState() === "idle") {
        runIdentify();
      }
    },
  ));

  // Build hierarchical disk → volumes tree
  const diskVolumeTree = createMemo(() => buildDiskVolumeTree(physicalDisks(), drives()));

  const toggleDiskExpand = (key: string) => {
    setExpandedDisks(prev => {
      const next = new Set(prev);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return next;
    });
  };

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

      {/* Three-panel layout: drives | workflow | activity */}
      <div class="acquire-dashboard-layout">

      {/* Left panel — Drive & Source Browser */}
      <Show when={props.hasProject()}>
        <div class="acquire-left-panel">
          <div class="acquire-left-header">
            <HiOutlineCircleStack class="w-icon-sm h-icon-sm text-accent shrink-0" />
            <span class="text-xs font-medium text-txt">Drives & Sources</span>
          </div>
          <div class="acquire-left-body">
            <DriveTreeBrowser
              onSelectSource={handleSelectSource}
              selectedPaths={() => selectedSources()}
              fillHeight
            />
          </div>
        </div>
      </Show>

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
                      {/* Recent projects */}
                      <Show when={props.onOpenRecentProject}>
                        <div class="col-span-2">
                          <RecentProjectsList
                            onOpenProject={props.onOpenRecentProject!}
                            maxItems={3}
                            compact
                          />
                        </div>
                      </Show>
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
                        <Show when={systemStats()!.systemModel}>
                          <span class="font-mono">{systemStats()!.systemModel}</span>
                          <span class="text-txt-muted/60">&middot;</span>
                        </Show>
                        <Show when={systemStats()!.systemSerialNumber}>
                          <span class="font-mono text-txt-muted">{systemStats()!.systemSerialNumber}</span>
                          <span class="text-txt-muted/60">&middot;</span>
                        </Show>
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
                        <Show when={evidenceFolder()}>
                          <span class="text-txt-muted/60">&middot;</span>
                          <HiOutlineFolder class="w-3.5 h-3.5 text-success" />
                          <span class="font-mono text-success" title={evidenceFolder()}>
                            {evidenceFolder().split("/").pop()}
                          </span>
                        </Show>
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
                          <Show when={systemStats()!.systemModel}>
                            <div class="acquire-detail-item">
                              <span class="acquire-detail-label">Model</span>
                              <span class="acquire-detail-value font-mono">{systemStats()!.systemModel}</span>
                            </div>
                          </Show>
                          <Show when={systemStats()!.systemSerialNumber}>
                            <div class="acquire-detail-item">
                              <span class="acquire-detail-label">Serial Number</span>
                              <span class="acquire-detail-value font-mono">{systemStats()!.systemSerialNumber}</span>
                            </div>
                          </Show>
                          <Show when={systemStats()!.systemManufacturer}>
                            <div class="acquire-detail-item">
                              <span class="acquire-detail-label">Manufacturer</span>
                              <span class="acquire-detail-value">{systemStats()!.systemManufacturer}</span>
                            </div>
                          </Show>
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
                          <Show when={evidenceFolder()}>
                            <div class="acquire-detail-item" style={{ "grid-column": "1 / -1" }}>
                              <span class="acquire-detail-label">Evidence Folder</span>
                              <span class="acquire-detail-value font-mono text-compact text-success" title={evidenceFolder()}>
                                {evidenceFolder()}
                              </span>
                            </div>
                          </Show>
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

                    {/* Hierarchical drives panel */}
                    <Show when={drives().length > 0} fallback={
                      <div class="acquire-drives-empty">
                        <HiOutlineCircleStack class="w-6 h-6 opacity-30" />
                        <span>No drives detected</span>
                      </div>
                    }>
                      <div class="acquire-drives-panel">
                        <span class="acquire-drives-label">Drives & Volumes</span>
                        <div class="acquire-disk-list">
                          <For each={diskVolumeTree()}>
                            {(group, i) => {
                              const key = group.disk?.wholeDiskPath || `ungrouped-${i()}`;
                              return (
                                <PhysicalDiskRow
                                  group={group}
                                  expanded={expandedDisks().has(key)}
                                  onToggle={() => toggleDiskExpand(key)}
                                  onAction={props.onAction}
                                />
                              );
                            }}
                          </For>
                        </div>
                      </div>
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
                            classList={{ "acquire-card-expanded": expandedCard() === card.id }}
                            onClick={() => toggleCard(card.id)}
                          >
                            <div class={`acquire-card-icon ${card.accent}`}>
                              <Icon class="w-5 h-5" />
                            </div>
                            <div class="acquire-card-content">
                              <h3 class="acquire-card-title">{card.title}</h3>
                              <p class="acquire-card-desc">{card.description}</p>
                            </div>
                            <Show when={expandedCard() === card.id}>
                              <HiOutlineChevronUp class="w-4 h-4 text-txt-muted shrink-0" />
                            </Show>
                          </button>
                        );
                      }}
                    </For>
                  </div>
                  {/* Inline expanded content for the active card in this phase */}
                  <Show when={expandedCard() !== null && phase.cardIds.includes(expandedCard()!) && props.renderExpandedContent}>
                    <div class="acquire-expanded-content" style={{ animation: "acquire-detail-expand 0.15s ease-out" }}>
                      <div class="acquire-expanded-header">
                        <span class="text-xs font-medium text-txt">{ALL_CARDS[expandedCard()!].title}</span>
                        <button class="icon-btn-sm" onClick={() => setExpandedCard(null)} title="Collapse">
                          <HiOutlineXMark class="w-4 h-4" />
                        </button>
                      </div>
                      {props.renderExpandedContent!(expandedCard()!, () => setExpandedCard(null))}
                    </div>
                  </Show>
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
                          classList={{ "acquire-card-expanded": expandedCard() === card.id }}
                          onClick={() => toggleCard(card.id)}
                        >
                          <div class={`acquire-card-icon ${card.accent}`}>
                            <Icon class="w-5 h-5" />
                          </div>
                          <div class="acquire-card-content">
                            <h3 class="acquire-card-title">{card.title}</h3>
                            <p class="acquire-card-desc">{card.description}</p>
                          </div>
                          <Show when={expandedCard() === card.id}>
                            <HiOutlineChevronUp class="w-4 h-4 text-txt-muted shrink-0" />
                          </Show>
                        </button>
                      );
                    }}
                  </For>
                </div>
                {/* Inline expanded content for the active card in this phase */}
                <Show when={expandedCard() !== null && phase.cardIds.includes(expandedCard()!) && props.renderExpandedContent}>
                  <div class="acquire-expanded-content" style={{ animation: "acquire-detail-expand 0.15s ease-out" }}>
                    <div class="acquire-expanded-header">
                      <span class="text-xs font-medium text-txt">{ALL_CARDS[expandedCard()!].title}</span>
                      <button class="icon-btn-sm" onClick={() => setExpandedCard(null)} title="Collapse">
                        <HiOutlineXMark class="w-4 h-4" />
                      </button>
                    </div>
                    {props.renderExpandedContent!(expandedCard()!, () => setExpandedCard(null))}
                  </div>
                </Show>
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

      </div>{/* end acquire-dashboard-main */}

      {/* Right panel — System Info, Collections, Recent Activity */}
      <Show when={props.hasProject()}>
        <div class="acquire-right-panel">
          <div class="acquire-right-panel-body">

            {/* System Info section */}
            <Show when={identifyState() === "done" && systemStats()}>
              <div class="acquire-right-section">
                <div
                  class="acquire-right-section-header"
                  onClick={() => toggleRightSection("system")}
                  role="button"
                  tabIndex={0}
                >
                  <HiOutlineComputerDesktop class="w-icon-compact h-icon-compact text-accent shrink-0" />
                  <span class="flex-1">System Info</span>
                  <Show when={rightSections().has("system")} fallback={
                    <ChevronRightIcon class="w-icon-micro h-icon-micro text-txt-muted" />
                  }>
                    <ChevronDownIcon class="w-icon-micro h-icon-micro text-txt-muted" />
                  </Show>
                </div>
                <Show when={rightSections().has("system")}>
                  <div class="acquire-right-section-content">
                    <Show when={systemStats()!.systemModel}>
                      <div class="acquire-right-meta-row">
                        <span class="acquire-right-meta-label">Model</span>
                        <span class="acquire-right-meta-value font-mono text-compact">{systemStats()!.systemModel}</span>
                      </div>
                    </Show>
                    <Show when={systemStats()!.systemSerialNumber}>
                      <div class="acquire-right-meta-row">
                        <span class="acquire-right-meta-label">Serial</span>
                        <span class="acquire-right-meta-value font-mono text-compact">{systemStats()!.systemSerialNumber}</span>
                      </div>
                    </Show>
                    <Show when={systemStats()!.hostname}>
                      <div class="acquire-right-meta-row">
                        <span class="acquire-right-meta-label">Hostname</span>
                        <span class="acquire-right-meta-value font-mono text-compact">{systemStats()!.hostname}</span>
                      </div>
                    </Show>
                    <Show when={systemStats()!.osName}>
                      <div class="acquire-right-meta-row">
                        <span class="acquire-right-meta-label">OS</span>
                        <span class="acquire-right-meta-value">{systemStats()!.osName} {systemStats()!.osVersion}</span>
                      </div>
                    </Show>
                    <div class="acquire-right-meta-row">
                      <span class="acquire-right-meta-label">CPU</span>
                      <span class="acquire-right-meta-value">{systemStats()!.cpuCores} cores</span>
                    </div>
                    <div class="acquire-right-meta-row">
                      <span class="acquire-right-meta-label">Memory</span>
                      <span class="acquire-right-meta-value">{formatSystemBytes(systemStats()!.memoryTotal)}</span>
                    </div>
                    <div class="acquire-right-meta-row">
                      <span class="acquire-right-meta-label">Volumes</span>
                      <span class="acquire-right-meta-value">{drives().length}</span>
                    </div>
                    <Show when={evidenceFolder()}>
                      <div class="acquire-right-meta-row">
                        <span class="acquire-right-meta-label">Evidence</span>
                        <span class="acquire-right-meta-value font-mono text-compact text-success" title={evidenceFolder()}>
                          {evidenceFolder().split("/").pop()}
                        </span>
                      </div>
                    </Show>
                  </div>
                </Show>
              </div>
            </Show>

            {/* Evidence Collections section */}
            <AcquireCollectionSummary
              embedded
              hasProject={props.hasProject}
              onViewCollection={props.onViewCollection}
              onNewCollection={() => props.onAction("collection")}
            />

            {/* Recent Acquisitions section */}
            <div class="acquire-right-section">
              <div
                class="acquire-right-section-header"
                onClick={() => toggleRightSection("recent")}
                role="button"
                tabIndex={0}
              >
                <HiOutlineClock class="w-icon-compact h-icon-compact text-accent shrink-0" />
                <span class="flex-1">Recent Acquisitions</span>
                <Show when={rightSections().has("recent")} fallback={
                  <ChevronRightIcon class="w-icon-micro h-icon-micro text-txt-muted" />
                }>
                  <ChevronDownIcon class="w-icon-micro h-icon-micro text-txt-muted" />
                </Show>
              </div>
              <Show when={rightSections().has("recent")}>
                <RecentAcquisitions />
              </Show>
            </div>

          </div>
        </div>
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
      <div class="acquire-right-section-content">
        <div class="space-y-1">
          <For each={records()}>
            {(rec) => {
              const isOk = rec.status === "completed";
              const isFail = rec.status === "failed";
              const label = EXPORT_TYPE_LABELS[rec.exportType] ?? rec.exportType;
              const dest = rec.destination?.split("/").pop() || rec.destination;

              return (
                <div class="flex items-center gap-2 px-2 py-1.5 rounded bg-bg-secondary hover:bg-bg-hover transition-colors text-xs">
                  <Show when={isOk} fallback={
                    <Show when={isFail} fallback={
                      <div class="w-1.5 h-1.5 rounded-full bg-accent animate-pulse-slow shrink-0" />
                    }>
                      <HiOutlineXCircle class="w-icon-compact h-icon-compact text-error shrink-0" />
                    </Show>
                  }>
                    <HiOutlineCheckCircle class="w-icon-compact h-icon-compact text-success shrink-0" />
                  </Show>

                  <div class="flex flex-col min-w-0 flex-1">
                    <span class="font-medium text-txt truncate">{label}</span>
                    <span class="text-txt-muted truncate" title={rec.destination}>{dest}</span>
                  </div>

                  <span class="text-txt-muted/60 shrink-0">{formatRelativeTime(rec.startedAt)}</span>
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
