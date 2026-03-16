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
  onMount,
  Accessor,
} from "solid-js";
import {
  HiOutlineCircleStack,
  HiOutlineFolder,
  HiOutlineArchiveBox,
  HiOutlineFingerPrint,
  HiOutlineArrowUpTray,
  HiOutlineFolderOpen,
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
} from "../icons";
import { APP_NAME } from "../../utils/edition";
import type { PortableConfig } from "../../api/portable";
import type { DbExportRecord } from "../../types/projectDb";
import { getExportHistory } from "../../api/exportHistory";

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
    title: "Triage Collection",
    description: "Quick forensic triage — collect key artifacts and scan for credentials",
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
    description: "Acquire a physical or logical drive as an E01 forensic image with hash verification",
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
    description: "Copy or archive selected files to a 7z container or folder with manifests",
    icon: HiOutlineArrowUpTray,
    accent: "text-amber-400",
  },
  verify: {
    id: "verify",
    title: "Verify Hashes",
    description: "Compute and verify MD5, SHA-1, or SHA-256 hashes of evidence files",
    icon: HiOutlineFingerPrint,
    accent: "text-rose-400",
  },
  collection: {
    id: "collection",
    title: "Evidence Collection",
    description: "Document on-site evidence collection with chain of custody tracking",
    icon: HiOutlineFolderOpen,
    accent: "text-cyan-400",
  },
};

/** Workflow phases that guide the user through an evidence collection workflow */
interface WorkflowPhase {
  step: number;
  title: string;
  cardIds: AcquireAction[];
}

const WORKFLOW_PHASES: WorkflowPhase[] = [
  {
    step: 1,
    title: "Identify & Collect",
    cardIds: ["browse", "triage", "memory"],
  },
  {
    step: 2,
    title: "Acquire & Image",
    cardIds: ["physical", "logical"],
  },
  {
    step: 3,
    title: "Export & Package",
    cardIds: ["export"],
  },
  {
    step: 4,
    title: "Verify & Document",
    cardIds: ["verify", "collection"],
  },
];

// =============================================================================
// Component
// =============================================================================

const AcquireDashboard: Component<AcquireDashboardProps> = (props) => {
  const [hoveredCard, setHoveredCard] = createSignal<string | null>(null);

  const freeSpaceGb = () => {
    const cfg = props.portableConfig();
    if (!cfg) return null;
    return (cfg.freeSpaceBytes / (1024 * 1024 * 1024)).toFixed(1);
  };

  return (
    <div class="acquire-dashboard">
      {/* Top bar — branding + project info + utility buttons */}
      <header class="acquire-topbar">
        <div class="flex items-center gap-3">
          <span class="text-sm font-semibold text-txt tracking-tight">{APP_NAME}</span>
          <Show when={props.isPortable()}>
            <div class="flex items-center gap-1 px-2 py-0.5 bg-emerald-500/10 border border-emerald-500/20 rounded" title={`Portable mode — data stored on removable media\n${props.portableConfig()?.dataDir || ""}`}>
              <HiOutlineServer class="w-3.5 h-3.5 text-emerald-400" />
              <span class="text-2xs font-medium text-emerald-400">Portable</span>
              <Show when={freeSpaceGb() !== null}>
                <span class="text-2xs text-emerald-400/60">{freeSpaceGb()} GB</span>
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

      {/* Workflow Phases */}
      <div class="acquire-workflow">
        <For each={WORKFLOW_PHASES}>
          {(phase) => (
            <div class="acquire-phase">
              <div class="acquire-phase-header">
                <div class="acquire-phase-number">{phase.step}</div>
                <span class="acquire-phase-title">{phase.title}</span>
              </div>
              <div class="acquire-phase-cards">
                <For each={phase.cardIds}>
                  {(cardId) => {
                    const card = ALL_CARDS[cardId];
                    const Icon = card.icon;
                    return (
                      <button
                        class="acquire-card"
                        classList={{
                          "acquire-card-hover": hoveredCard() === card.id,
                        }}
                        onMouseEnter={() => setHoveredCard(card.id)}
                        onMouseLeave={() => setHoveredCard(null)}
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
          )}
        </For>
      </div>

      {/* Quick Verify — fast file picker shortcut */}
      <Show when={props.onQuickVerify}>
        <div class="px-6 pb-2">
          <button
            class="w-full flex items-center gap-3 px-4 py-2.5 rounded-lg border border-dashed border-border hover:border-accent/40 hover:bg-accent/5 transition-colors text-left group"
            onClick={() => props.onQuickVerify?.()}
          >
            <HiOutlineFingerPrint class="w-5 h-5 text-txt-muted group-hover:text-accent transition-colors shrink-0" />
            <div>
              <span class="text-sm font-medium text-txt group-hover:text-accent transition-colors">Quick Hash File</span>
              <span class="text-xs text-txt-muted ml-2">Select files to immediately compute and verify hashes</span>
            </div>
          </button>
        </div>
      </Show>

      {/* Recent Acquisitions */}
      <Show when={props.hasProject()}>
        <RecentAcquisitions />
      </Show>
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
        <div class="flex items-center gap-2 mb-3">
          <HiOutlineClock class="w-4 h-4 text-txt-muted" />
          <span class="text-xs font-medium text-txt-muted uppercase tracking-wider">Recent Activity</span>
        </div>
        <div class="space-y-1.5">
          <For each={records()}>
            {(rec) => {
              const isOk = rec.status === "completed";
              const isFail = rec.status === "failed";
              const label = EXPORT_TYPE_LABELS[rec.exportType] ?? rec.exportType;
              const dest = rec.destination?.split("/").pop() || rec.destination;

              return (
                <div class="flex items-center gap-3 px-3 py-2 rounded-lg bg-bg-secondary/50 text-xs">
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
