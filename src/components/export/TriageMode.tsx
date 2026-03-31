// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

// TriageMode — Forensic triage collection + credential scanning UI
//
// Three-phase layout: Setup (profile/category chips/container format),
// Collecting (system info + live progress), Done (summary + secrets).

import { Show, For, onMount, createMemo, createEffect, on } from "solid-js";
import type { Accessor, Setter } from "solid-js";
import { CoreSpinner } from "@core-suite/icons";
import {
  HiOutlineShieldCheck,
  HiOutlineShieldExclamation,
  HiOutlineKey,
  HiOutlineCheckCircle,
  HiOutlineExclamationTriangle,
  HiOutlineFolderOpen,
  HiOutlineTag,
} from "../icons";
import type {
  TriageProfile,
  TriageCategory,
  TriageProgress,
  TriageResult,
  SecretFinding,
  CategoryResult,
} from "../../api/triage";
import { createProgressTracker } from "../../hooks/useProgressTracker";
import { systemCommands } from "../../api/commands";

// --- Props ---

export interface TriageModeProps {
  triageProfiles: Accessor<TriageProfile[]>;
  triageCategories: Accessor<TriageCategory[]>;
  triageProfilesLoading: Accessor<boolean>;
  selectedTriageProfile: Accessor<string>;
  setSelectedTriageProfile: (profileId: string) => void;
  selectedTriageCategories: Accessor<string[]>;
  toggleTriageCategory: (categoryId: string) => void;
  triageScanForSecrets: Accessor<boolean>;
  setTriageScanForSecrets: Setter<boolean>;
  triageContainerFormat: Accessor<string>;
  setTriageContainerFormat: Setter<string>;
  triageProgress: Accessor<TriageProgress | null>;
  triageResult: Accessor<TriageResult | null>;
  isCollecting: Accessor<boolean>;
  onLoadProfiles: () => void;
  /** Pre-collected system stats from Identify phase for display during collection. */
  systemStats?: { hostname?: string; systemModel?: string; systemSerialNumber?: string; systemManufacturer?: string; osName?: string; osVersion?: string } | null;
  /** Active triage activity from App-level (survives panel remount) */
  activeTriageActivity?: Accessor<import("../../types/activity").Activity | undefined>;
}

// --- Helpers ---

function formatSize(bytes: number): string {
  if (bytes >= 1024 * 1024 * 1024) {
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
  }
  if (bytes >= 1024 * 1024) {
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }
  if (bytes >= 1024) {
    return `${(bytes / 1024).toFixed(1)} KB`;
  }
  return `${bytes} B`;
}

function confidenceColor(confidence: string): string {
  switch (confidence) {
    case "high":
      return "text-error";
    case "medium":
      return "text-warning";
    default:
      return "text-txt-muted";
  }
}

function confidenceBg(confidence: string): string {
  switch (confidence) {
    case "high":
      return "bg-error/10 border-error/30";
    case "medium":
      return "bg-warning/10 border-warning/30";
    default:
      return "bg-bg-secondary border-border/30";
  }
}

/** Category icons + short tooltip descriptions for quick reference. */
const CATEGORY_META: Record<string, { icon: string; tip: string }> = {
  registry: { icon: "🗃️", tip: "Windows registry hives (SAM, SECURITY, SYSTEM, SOFTWARE)" },
  eventlogs: { icon: "📋", tip: "System event logs, audit trails, and forensic log sources" },
  system: { icon: "⚙️", tip: "OS-level config files, scheduled tasks, security databases" },
  credentials: { icon: "🔑", tip: "SSH keys, cloud credentials, keychains, API tokens" },
  browser: { icon: "🌐", tip: "Browser profiles, history, cookies, saved passwords" },
  useractivity: { icon: "👤", tip: "Shell history, recent docs, application usage" },
  network: { icon: "🌍", tip: "Hosts file, WiFi profiles, firewall rules, DNS config" },
  systeminfo: { icon: "🖥️", tip: "Hardware UUID, serial number, hostname, OS version" },
};

/** Profile descriptions for tooltip display when hovering profile selector options. */
const PROFILE_TIPS: Record<string, string> = {
  security: "Collects registry hives, event logs, and system security databases — ideal for incident response and intrusion analysis.",
  credentials: "Targets authentication material: SSH keys, cloud provider tokens, browser password databases, and certificates.",
  useractivity: "Captures user behavior artifacts: shell history, recent documents, browser history, and application usage patterns.",
  network: "Network configuration artifacts: WiFi profiles, DNS settings, hosts file, and firewall rules.",
  full: "Comprehensive collection of ALL artifact categories — recommended for thorough forensic triage.",
  identification: "Essential system identifiers (UUID, serial number, hostname, OS version) for evidence forms and chain of custody.",
};

// --- Component ---

export function TriageMode(props: TriageModeProps) {
  onMount(() => {
    if (props.triageProfiles().length === 0) {
      props.onLoadProfiles();
    }
  });

  const progress = createMemo(() => props.triageProgress());
  const result = createMemo(() => props.triageResult());
  const selectedCats = createMemo(() => props.selectedTriageCategories());
  const isCollecting = createMemo(() => props.isCollecting());

  // Smoothed speed/ETA tracker
  const tracker = createProgressTracker();
  createEffect(on(progress, (p) => {
    if (p) {
      tracker.update({ bytesProcessed: p.bytesCollected, bytesTotal: p.filesTotal > 0 ? (p.bytesCollected / Math.max(p.percent, 0.1)) * 100 : 0, percent: p.percent });
    }
  }));

  // Detect an active triage activity from the App-level activity tracker.
  const hasActiveTriageFromActivity = createMemo(() => {
    if (isCollecting() || progress() || result()) return false;
    const act = props.activeTriageActivity?.();
    return act != null && (act.status === "running" || act.status === "pending");
  });

  // Phase: "setup" | "collecting" | "done"
  const phase = createMemo(() => {
    if (result()) return "done" as const;
    if (isCollecting() || progress() || hasActiveTriageFromActivity()) return "collecting" as const;
    return "setup" as const;
  });

  // System summary line for display during collection
  const systemSummary = createMemo(() => {
    const s = props.systemStats;
    if (!s) return null;
    const parts: string[] = [];
    if (s.hostname) parts.push(s.hostname);
    if (s.systemModel) parts.push(s.systemModel);
    if (s.osName) {
      parts.push(s.osVersion ? `${s.osName} ${s.osVersion}` : s.osName);
    }
    return parts.length > 0 ? parts.join(" · ") : null;
  });

  return (
    <div class="flex flex-col gap-2.5">
      {/* ── COLLECTING PHASE ── */}
      <Show when={phase() === "collecting"}>
        {/* System identification summary */}
        <Show when={systemSummary()}>
          <div class="flex items-center gap-2 px-2 py-1.5 rounded bg-bg-secondary border border-border">
            <span class="text-xs">🖥️</span>
            <span class="text-xs text-txt-muted">{systemSummary()}</span>
            <Show when={props.systemStats?.systemSerialNumber}>
              <span class="text-xs text-txt-muted/50">·</span>
              <span class="font-mono text-compact text-txt-muted">S/N {props.systemStats!.systemSerialNumber}</span>
            </Show>
          </div>
        </Show>

        {/* Active triage from App-level activity (panel remounted) */}
        <Show when={hasActiveTriageFromActivity()}>
          {(_) => {
            const act = () => props.activeTriageActivity!()!;
            return (
              <div class="callout">
                <div class="flex items-center gap-2 mb-2">
                  <div class="w-4 h-4 border-2 border-accent border-t-transparent rounded-full animate-spin" />
                  <span class="text-sm font-medium text-txt">Triage collection in progress...</span>
                </div>
                <Show when={act().progress}>
                  <div class="w-full h-2 bg-bg-secondary rounded-full overflow-hidden" role="progressbar" aria-valuenow={Math.round(Math.min(act().progress!.percent, 100))} aria-valuemin={0} aria-valuemax={100} aria-label="Triage collection progress">
                    <div
                      class="h-full bg-accent rounded-full transition-all duration-200"
                      style={{ width: `${Math.min(act().progress!.percent, 100)}%` }}
                    />
                  </div>
                  <div class="flex justify-between mt-1 text-xs text-txt-muted">
                    <Show when={act().progress!.filesProcessed != null}>
                      <span>{act().progress!.filesProcessed}{act().progress!.totalFiles ? ` / ${act().progress!.totalFiles}` : ""} files</span>
                    </Show>
                    <span>{act().progress!.percent.toFixed(1)}%</span>
                  </div>
                </Show>
              </div>
            );
          }}
        </Show>

        {/* Initializing indicator */}
        <Show when={isCollecting() && !progress() && !result()}>
          <div class="callout">
            <div class="flex items-center gap-2">
              <div class="w-4 h-4 border-2 border-accent border-t-transparent rounded-full animate-spin" />
              <span class="text-sm font-medium text-txt">Initializing triage collection...</span>
            </div>
            <div class="text-xs text-txt-muted mt-1">
              Enumerating system artifacts in {selectedCats().length} categor{selectedCats().length !== 1 ? "ies" : "y"}
            </div>
          </div>
        </Show>

        {/* Live progress */}
        <Show when={progress()}>
          {(_) => {
            const p = () => props.triageProgress()!;
            const s = () => tracker.stats();
            return (
              <div class="callout">
                <div class="flex items-center justify-between mb-2">
                  <div class="flex items-center gap-2">
                    <CoreSpinner size={12} />
                    <span class="text-xs font-medium text-txt">
                      {p().phase === "collecting"
                        ? `Collecting ${p().currentCategory}...`
                        : p().phase === "scanning"
                          ? "Scanning for secrets..."
                          : p().phase === "packaging"
                            ? "Packaging into container..."
                            : p().phase === "complete"
                              ? "Finalizing..."
                              : p().phase}
                    </span>
                  </div>
                  <span class="text-sm font-semibold text-accent">{p().percent.toFixed(1)}%</span>
                </div>
                <div class="w-full h-2 bg-bg-secondary rounded-full overflow-hidden" role="progressbar" aria-valuenow={Math.round(Math.min(p().percent, 100))} aria-valuemin={0} aria-valuemax={100} aria-label="Triage collection progress">
                  <div
                    class="progress-fill-active rounded-full"
                    style={{ width: `${Math.min(p().percent, 100)}%` }}
                  />
                </div>
                <div class="flex items-center justify-between mt-1.5 text-xs text-txt-muted">
                  <span>{p().filesCollected} / {p().filesTotal} files • {formatSize(p().bytesCollected)}</span>
                  <div class="flex items-center gap-2">
                    <Show when={s().speedFormatted}>
                      <span class="text-accent font-medium">{s().speedFormatted}</span>
                    </Show>
                    <Show when={s().etaFormatted}>
                      <span>ETA {s().etaFormatted}</span>
                    </Show>
                  </div>
                </div>
                <Show when={p().currentFile}>
                  <div class="text-xs text-txt-muted mt-0.5 truncate" title={p().currentFile}>
                    {p().currentFile}
                  </div>
                </Show>
                <Show when={s().elapsedMs >= 1000}>
                  <div class="text-xs text-txt-muted mt-0.5">
                    Elapsed: {s().elapsedFormatted}
                  </div>
                </Show>
              </div>
            );
          }}
        </Show>
      </Show>

      {/* ── RESULT PHASE ── */}
      <Show when={phase() === "done"}>
        {(_) => {
          const r = () => props.triageResult()!;
          return (
            <div class="space-y-3">
              <div class={`card border ${r().cancelled ? "border-warning/30" : "border-success/30"}`}>
                <div class="flex items-center justify-between mb-3">
                  <div class="flex items-center gap-2">
                    <Show
                      when={!r().cancelled}
                      fallback={
                        <>
                          <HiOutlineExclamationTriangle class="w-icon-sm h-icon-sm text-warning" />
                          <span class="text-sm font-medium text-warning">Triage Cancelled</span>
                        </>
                      }
                    >
                      <HiOutlineCheckCircle class="w-icon-sm h-icon-sm text-success" />
                      <span class="text-sm font-medium text-success">Collection Complete</span>
                    </Show>
                  </div>
                  <button
                    class="btn-sm flex items-center gap-1.5"
                    onClick={() => systemCommands.openPath(r().containerPath || r().outputDir)}
                    title="Open the output location"
                  >
                    <HiOutlineFolderOpen class="w-4 h-4" />
                    <span>Open</span>
                  </button>
                </div>

                {/* Compact stats */}
                <div class="grid grid-cols-4 gap-2 mb-3">
                  <div class="stat-box">
                    <div class="text-txt-muted text-2xs">Files</div>
                    <div class="text-sm font-semibold text-txt">{r().filesCollected}</div>
                  </div>
                  <div class="stat-box">
                    <div class="text-txt-muted text-2xs">Size</div>
                    <div class="text-sm font-semibold text-txt">{formatSize(r().bytesCollected)}</div>
                  </div>
                  <div class="stat-box">
                    <div class="text-txt-muted text-2xs">Time</div>
                    <div class="text-sm font-semibold text-txt">
                      {r().durationSecs < 60
                        ? `${r().durationSecs.toFixed(1)}s`
                        : `${Math.floor(r().durationSecs / 60)}m ${Math.floor(r().durationSecs % 60)}s`}
                    </div>
                  </div>
                  <div class="stat-box">
                    <div class="text-txt-muted text-2xs">Secrets</div>
                    <div class={`text-sm font-semibold ${r().secretFindings.length > 0 ? "text-warning" : "text-txt"}`}>
                      {r().secretFindings.length}
                    </div>
                  </div>
                </div>

                {/* Output location */}
                <div class="text-xs text-txt-muted bg-bg-secondary rounded p-1.5 mb-2">
                  <span class="font-medium">{r().containerPath ? "Container: " : "Output: "}</span>
                  <span class="font-mono text-compact break-all">{r().containerPath || r().outputDir}</span>
                </div>

                {/* Per-category breakdown */}
                <Show when={r().categoriesCollected.length > 0}>
                  <div class="space-y-1">
                    <div class="flex items-center gap-1.5 mb-1">
                      <HiOutlineTag class="w-3 h-3 text-txt-muted shrink-0" />
                      <span class="text-2xs font-medium text-txt-muted uppercase tracking-wider">Collected by Category</span>
                    </div>
                    <For each={r().categoriesCollected}>
                      {(cat) => {
                        const detail = () => r().categoryDetails?.[cat] as CategoryResult | undefined;
                        const meta = CATEGORY_META[cat] || { icon: "📁", tip: cat };
                        return (
                          <div class="bg-bg-secondary rounded p-2">
                            <div class="flex items-center justify-between">
                              <div class="flex items-center gap-1.5">
                                <span class="text-xs">{meta.icon}</span>
                                <span class="text-xs font-medium text-txt capitalize">{cat}</span>
                              </div>
                              <Show when={detail()}>
                                <div class="flex items-center gap-2 text-xs text-txt-muted">
                                  <span>{detail()!.filesCollected} file{detail()!.filesCollected !== 1 ? "s" : ""}</span>
                                  <span class="text-txt-muted/50">·</span>
                                  <span>{formatSize(detail()!.bytesCollected)}</span>
                                  <Show when={detail()!.filesSkipped > 0}>
                                    <span class="text-txt-muted/50">·</span>
                                    <span class="text-warning">{detail()!.filesSkipped} skipped</span>
                                  </Show>
                                  <Show when={detail()!.filesFailed > 0}>
                                    <span class="text-txt-muted/50">·</span>
                                    <span class="text-error">{detail()!.filesFailed} failed</span>
                                  </Show>
                                </div>
                              </Show>
                            </div>
                          </div>
                        );
                      }}
                    </For>
                  </div>
                </Show>

                <Show when={r().filesSkipped > 0}>
                  <div class="text-xs text-txt-muted mt-1.5">
                    {r().filesSkipped} file{r().filesSkipped !== 1 ? "s" : ""} skipped
                  </div>
                </Show>
              </div>

              {/* Secret Findings */}
              <Show when={r().secretFindings.length > 0}>
                <div class="flex flex-col gap-2 pb-3 border-b border-border last:border-b-0 last:pb-0">
                  <div class="flex items-center gap-2 mb-2">
                    <HiOutlineShieldExclamation class="w-icon-sm h-icon-sm text-warning" />
                    <span class="text-xs font-medium text-warning">
                      Credential & Secret Findings ({r().secretFindings.length})
                    </span>
                  </div>
                  <div class="space-y-1.5 max-h-48 overflow-y-auto">
                    <For each={r().secretFindings}>
                      {(finding) => <SecretFindingRow finding={finding} />}
                    </For>
                  </div>
                </div>
              </Show>

              <Show when={r().secretFindings.length === 0 && !r().cancelled}>
                <div class="text-xs text-txt-muted p-2 bg-bg-secondary rounded text-center">
                  No credentials or secrets detected in collected files
                </div>
              </Show>
            </div>
          );
        }}
      </Show>

      {/* ── SETUP PHASE ── */}
      <Show when={phase() === "setup"}>
        {/* System summary (if available from Identify) */}
        <Show when={systemSummary()}>
          <div class="flex items-center gap-2 px-2 py-1.5 rounded bg-bg-secondary border border-border">
            <span class="text-xs">🖥️</span>
            <span class="text-xs text-txt">{systemSummary()}</span>
          </div>
        </Show>

        {/* Profile Selection */}
        <div class="flex flex-col gap-2 pb-3 border-b border-border last:border-b-0 last:pb-0">
          <div class="flex items-center gap-2">
            <HiOutlineShieldCheck class="w-icon-sm h-icon-sm text-accent" />
            <span class="text-xs font-semibold text-txt">Collection Profile</span>
          </div>

          <Show when={props.triageProfilesLoading()}>
            <div class="text-xs text-txt-muted animate-pulse-slow py-2">
              Detecting platform artifacts...
            </div>
          </Show>

          <Show when={!props.triageProfilesLoading() && props.triageProfiles().length > 0}>
            <select
              class="input-sm w-full text-xs"
              value={props.selectedTriageProfile()}
              onChange={(e) => props.setSelectedTriageProfile(e.currentTarget.value)}
            >
              <For each={props.triageProfiles()}>
                {(profile) => (
                  <option value={profile.id}>{profile.name}</option>
                )}
              </For>
              <option value="custom">Custom Selection</option>
            </select>

            <Show when={props.selectedTriageProfile() !== "custom"}>
              {(() => {
                const profile = createMemo(() =>
                  props.triageProfiles().find((p) => p.id === props.selectedTriageProfile()),
                );
                return (
                  <Show when={profile()}>
                    <div class="text-xs text-txt-muted p-1.5 bg-bg-secondary rounded mt-1.5">
                      {PROFILE_TIPS[profile()!.id] || profile()!.description}
                    </div>
                  </Show>
                );
              })()}
            </Show>
          </Show>
        </div>

        {/* Artifact Categories — Simplified chip grid (no expand) */}
        <Show when={props.triageCategories().length > 0}>
          <div class="flex flex-col gap-2 pb-3 border-b border-border last:border-b-0 last:pb-0">
            <div class="flex items-center gap-2">
              <span class="text-xs font-semibold text-txt">Artifact Categories</span>
              <span class="text-2xs text-txt-muted ml-auto">
                {selectedCats().length}/{props.triageCategories().length}
              </span>
            </div>
            <div class="flex flex-wrap gap-1.5">
              <For each={props.triageCategories()}>
                {(cat) => {
                  const meta = CATEGORY_META[cat.id] || { icon: "📁", tip: cat.description };
                  const isSelected = () => selectedCats().includes(cat.id);
                  return (
                    <button
                      class="flex items-center gap-1 px-2 py-1 rounded-md border text-xs transition-colors"
                      classList={{
                        "border-accent/40 bg-accent/10 text-txt": isSelected(),
                        "border-border/30 bg-bg-secondary/50 text-txt-muted": !isSelected(),
                      }}
                      onClick={() => props.toggleTriageCategory(cat.id)}
                      title={meta.tip}
                    >
                      <span>{meta.icon}</span>
                      <span>{cat.name}</span>
                      <span class="text-2xs opacity-60">{cat.artifactCount}</span>
                    </button>
                  );
                }}
              </For>
            </div>
          </div>
        </Show>

        {/* Container Format + Secret Scanning — single card */}
        <div class="flex flex-col gap-2 pb-3 border-b border-border last:border-b-0 last:pb-0 space-y-3">
          {/* Container format selector */}
          <div class="flex items-center gap-2">
            <span class="text-xs font-medium text-txt w-24 shrink-0">Container</span>
            <select
              class="input-sm flex-1 text-xs"
              value={props.triageContainerFormat()}
              onChange={(e) => props.setTriageContainerFormat(e.currentTarget.value)}
            >
              <option value="7z">7z Archive (Store)</option>
              <option value="">No container (raw files)</option>
            </select>
          </div>

          {/* Credential scanning toggle */}
          <label class="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              checked={props.triageScanForSecrets()}
              onChange={(e) => props.setTriageScanForSecrets(e.currentTarget.checked)}
              class="rounded border-border shrink-0"
            />
            <HiOutlineKey class="w-3.5 h-3.5 text-txt-muted shrink-0" />
            <span class="text-xs text-txt">Scan for credentials & secrets</span>
          </label>
        </div>
      </Show>
    </div>
  );
}

// --- Sub-component ---

function SecretFindingRow(props: { finding: SecretFinding }) {
  const f = () => props.finding;

  return (
    <div class={`p-1.5 rounded border ${confidenceBg(f().confidence)}`}>
      <div class="flex items-center gap-2 mb-0.5">
        <span class={`text-2xs font-medium uppercase ${confidenceColor(f().confidence)}`}>
          {f().confidence}
        </span>
        <span class="text-xs font-medium text-txt">{f().secretType}</span>
      </div>
      <div class="text-xs text-txt-muted">{f().description}</div>
      <div class="font-mono text-compact text-txt-muted mt-0.5 truncate" title={f().preview}>
        {f().preview}
      </div>
      <div class="text-xs text-txt-muted mt-0.5 truncate" title={f().filePath}>
        {f().filePath}:{f().lineNumber}
      </div>
    </div>
  );
}
