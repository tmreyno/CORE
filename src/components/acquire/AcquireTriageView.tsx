// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * AcquireTriageView — Standalone triage view for the Acquire edition.
 *
 * Provides a minimal UI: profile selector → start button → progress → results.
 * Uses useTriageState directly (no heavyweight useExportState / ExportPanel).
 */

import { Component, Show, For, createSignal, createMemo, onMount } from "solid-js";
import type { Accessor } from "solid-js";
import {
  HiOutlineArrowLeft,
  HiOutlineShieldCheck,
  HiOutlineShieldExclamation,
  HiOutlineKey,
  HiOutlineCheckCircle,
  HiOutlineExclamationTriangle,
  HiOutlineFolderOpen,
  HiOutlineTag,
  HiOutlinePlay,
  HiOutlineStop,
} from "../icons";
import { useTriageState } from "../../hooks/export/useTriageState";
import type { ExportCommonState } from "../../hooks/export/useExportCommon";
import { useToast } from "../Toast";
import type { Activity } from "../../types/activity";
import type { SecretFinding, CategoryResult } from "../../api/triage";
import { systemCommands } from "../../api/commands";

// =============================================================================
// Helpers
// =============================================================================

function formatSize(bytes: number): string {
  if (bytes >= 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
  if (bytes >= 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  if (bytes >= 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${bytes} B`;
}

function confidenceColor(confidence: string): string {
  switch (confidence) {
    case "high": return "text-error";
    case "medium": return "text-warning";
    default: return "text-txt-muted";
  }
}

function confidenceBg(confidence: string): string {
  switch (confidence) {
    case "high": return "bg-error/10 border-error/30";
    case "medium": return "bg-warning/10 border-warning/30";
    default: return "bg-bg-secondary border-border/30";
  }
}

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

const PROFILE_TIPS: Record<string, string> = {
  security: "Registry hives, event logs, and system security databases — ideal for incident response.",
  credentials: "SSH keys, cloud tokens, browser password databases, and certificates.",
  useractivity: "Shell history, recent documents, browser history, and application usage patterns.",
  network: "WiFi profiles, DNS settings, hosts file, and firewall rules.",
  full: "ALL artifact categories — recommended for thorough forensic triage.",
  identification: "System identifiers (UUID, serial, hostname, OS) for evidence forms and COC.",
};

// =============================================================================
// Props
// =============================================================================

export interface AcquireTriageViewProps {
  onBack: () => void;
  initialDestination: string;
  onComplete?: (destination: string) => void;
  onActivityCreate?: (activity: Activity) => void;
  onActivityUpdate?: (id: string, updates: Partial<Activity>) => void;
  caseNumber?: Accessor<string | undefined>;
  examinerName?: Accessor<string | undefined>;
  systemStats?: Accessor<{ hostname?: string; systemModel?: string; systemSerialNumber?: string; systemManufacturer?: string; osName?: string; osVersion?: string } | null>;
  activeTriageActivity?: Accessor<Activity | undefined>;
  inline?: boolean;
}

// =============================================================================
// Component
// =============================================================================

const AcquireTriageView: Component<AcquireTriageViewProps> = (props) => {
  const toast = useToast();

  // Minimal common state — useTriageState only needs destination + processing flags
  const [destination] = createSignal(props.initialDestination);
  const [isProcessing, setIsProcessing] = createSignal(false);
  const [isAcquiring, setIsAcquiring] = createSignal(false);

  const common = {
    destination,
    setIsProcessing,
    setIsAcquiring,
    isProcessing,
    isAcquiring,
  } as unknown as ExportCommonState;

  const triage = useTriageState({
    toast,
    common,
    caseNumber: props.caseNumber?.(),
    examinerName: props.examinerName?.(),
    systemStats: props.systemStats?.(),
    onActivityCreate: props.onActivityCreate,
    onActivityUpdate: props.onActivityUpdate,
    onComplete: props.onComplete,
  });

  onMount(() => {
    if (triage.triageProfiles().length === 0) {
      triage.loadTriageProfiles();
    }
  });

  const progress = createMemo(() => triage.triageProgress());
  const result = createMemo(() => triage.triageResult());
  const selectedCats = createMemo(() => triage.selectedTriageCategories());
  const collecting = createMemo(() => isProcessing());

  const hasActiveTriageFromActivity = createMemo(() => {
    if (collecting() || progress() || result()) return false;
    const act = props.activeTriageActivity?.();
    return act != null && (act.status === "running" || act.status === "pending");
  });

  const phase = createMemo(() => {
    if (result()) return "done" as const;
    if (collecting() || progress() || hasActiveTriageFromActivity()) return "collecting" as const;
    return "setup" as const;
  });

  const systemSummary = createMemo(() => {
    const s = props.systemStats?.();
    if (!s) return null;
    const parts: string[] = [];
    if (s.hostname) parts.push(s.hostname);
    if (s.systemModel) parts.push(s.systemModel);
    if (s.osName) parts.push(s.osVersion ? `${s.osName} ${s.osVersion}` : s.osName);
    return parts.length > 0 ? parts.join(" · ") : null;
  });

  const handleStart = () => {
    if (!destination()) {
      toast.error("No Destination", "Evidence folder not set. Run Identify System first.");
      return;
    }
    triage.handleTriageCollect();
  };

  const handleNewTriage = () => {
    triage.resetTriageState();
  };

  return (
    <div class="flex flex-col flex-1 min-h-0 overflow-hidden">
      {/* Header bar */}
      <div class="flex items-center gap-2 px-3 py-1.5 border-b border-border bg-bg-secondary shrink-0">
        <button class="btn btn-ghost gap-1 text-xs py-1 px-2" onClick={props.onBack}>
          <HiOutlineArrowLeft class="w-icon-sm h-icon-sm" />
          Dashboard
        </button>
        <span class="text-2xs text-txt-muted uppercase tracking-wider font-medium">Quick Triage</span>
      </div>

      {/* Content */}
      <div class="flex-1 overflow-y-auto p-4">
        <div class="max-w-lg mx-auto space-y-3">

          {/* ── SETUP PHASE ── */}
          <Show when={phase() === "setup"}>
            {/* System summary */}
            <Show when={systemSummary()}>
              <div class="flex items-center gap-2 bg-bg-secondary rounded px-2.5 py-1.5">
                <span class="text-xs">🖥️</span>
                <span class="text-xs text-txt">{systemSummary()}</span>
              </div>
            </Show>

            {/* Profile Selection */}
            <div class="card">
              <div class="flex items-center gap-2 mb-2">
                <HiOutlineShieldCheck class="w-icon-sm h-icon-sm text-accent" />
                <span class="text-xs font-medium text-txt">Collection Profile</span>
              </div>

              <Show when={triage.triageProfilesLoading()}>
                <div class="text-xs text-txt-muted animate-pulse-slow py-2">
                  Detecting platform artifacts...
                </div>
              </Show>

              <Show when={!triage.triageProfilesLoading() && triage.triageProfiles().length > 0}>
                <select
                  class="input-sm w-full text-xs"
                  value={triage.selectedTriageProfile()}
                  onChange={(e) => triage.setSelectedTriageProfile(e.currentTarget.value)}
                >
                  <For each={triage.triageProfiles()}>
                    {(profile) => <option value={profile.id}>{profile.name}</option>}
                  </For>
                  <option value="custom">Custom Selection</option>
                </select>

                <Show when={triage.selectedTriageProfile() !== "custom"}>
                  {(() => {
                    const profile = createMemo(() =>
                      triage.triageProfiles().find((p) => p.id === triage.selectedTriageProfile()),
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

            {/* Artifact Categories */}
            <Show when={triage.triageCategories().length > 0}>
              <div class="card">
                <div class="flex items-center gap-2 mb-2">
                  <span class="text-xs font-medium text-txt">Artifact Categories</span>
                  <span class="text-2xs text-txt-muted ml-auto">
                    {selectedCats().length}/{triage.triageCategories().length}
                  </span>
                </div>
                <div class="flex flex-wrap gap-1.5">
                  <For each={triage.triageCategories()}>
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
                          onClick={() => triage.toggleTriageCategory(cat.id)}
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

            {/* Options: container format + secrets toggle */}
            <div class="card space-y-3">
              <div class="flex items-center gap-2">
                <span class="text-xs font-medium text-txt w-24 shrink-0">Container</span>
                <select
                  class="input-sm flex-1 text-xs"
                  value={triage.triageContainerFormat()}
                  onChange={(e) => triage.setTriageContainerFormat(e.currentTarget.value)}
                >
                  <option value="7z">7z Archive (Store)</option>
                  <option value="">No container (raw files)</option>
                </select>
              </div>
              <label class="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={triage.triageScanForSecrets()}
                  onChange={(e) => triage.setTriageScanForSecrets(e.currentTarget.checked)}
                  class="rounded border-border shrink-0"
                />
                <HiOutlineKey class="w-3.5 h-3.5 text-txt-muted shrink-0" />
                <span class="text-xs text-txt">Scan for credentials & secrets</span>
              </label>
            </div>

            {/* Destination (read-only display) */}
            <Show when={destination()}>
              <div class="text-xs text-txt-muted bg-bg-secondary rounded p-2">
                <span class="font-medium">Output: </span>
                <span class="font-mono text-compact break-all">{destination()}</span>
              </div>
            </Show>

            {/* Start button */}
            <button
              class="btn btn-primary w-full gap-2"
              onClick={handleStart}
              disabled={triage.triageProfilesLoading() || selectedCats().length === 0}
            >
              <HiOutlinePlay class="w-icon-sm h-icon-sm" />
              Start Triage
            </button>
          </Show>

          {/* ── COLLECTING PHASE ── */}
          <Show when={phase() === "collecting"}>
            <Show when={systemSummary()}>
              <div class="flex items-center gap-2 bg-bg-secondary rounded px-2.5 py-1.5">
                <span class="text-xs">🖥️</span>
                <span class="text-xs text-txt-muted">{systemSummary()}</span>
                <Show when={props.systemStats?.()?.systemSerialNumber}>
                  <span class="text-xs text-txt-muted/50">·</span>
                  <span class="font-mono text-compact text-txt-muted">S/N {props.systemStats!()!.systemSerialNumber}</span>
                </Show>
              </div>
            </Show>

            {/* Active triage from App-level activity (panel remounted) */}
            <Show when={hasActiveTriageFromActivity()}>
              {(_) => {
                const act = () => props.activeTriageActivity!()!;
                return (
                  <div class="card border border-accent/30">
                    <div class="flex items-center gap-2 mb-2">
                      <div class="w-4 h-4 border-2 border-accent border-t-transparent rounded-full animate-spin" />
                      <span class="text-sm font-medium text-txt">Triage collection in progress...</span>
                    </div>
                    <Show when={act().progress}>
                      <div class="w-full h-2 bg-bg-secondary rounded-full overflow-hidden" role="progressbar" aria-valuenow={Math.round(Math.min(act().progress!.percent, 100))} aria-valuemin={0} aria-valuemax={100}>
                        <div class="h-full bg-accent rounded-full transition-all duration-200" style={{ width: `${Math.min(act().progress!.percent, 100)}%` }} />
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

            {/* Initializing */}
            <Show when={collecting() && !progress() && !result()}>
              <div class="card border border-accent/30">
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
                const p = () => triage.triageProgress()!;
                return (
                  <div class="card border border-accent/30">
                    <div class="flex items-center justify-between mb-2">
                      <span class="text-xs font-medium text-txt">
                        {p().phase === "collecting" ? `Collecting ${p().currentCategory}...`
                          : p().phase === "scanning" ? "Scanning for secrets..."
                          : p().phase === "packaging" ? "Packaging into container..."
                          : p().phase === "complete" ? "Finalizing..."
                          : p().phase}
                      </span>
                      <span class="text-xs text-txt-muted">{p().percent.toFixed(1)}%</span>
                    </div>
                    <div class="w-full h-2 bg-bg-secondary rounded-full overflow-hidden" role="progressbar" aria-valuenow={Math.round(Math.min(p().percent, 100))} aria-valuemin={0} aria-valuemax={100}>
                      <div class="h-full bg-accent rounded-full transition-all duration-200" style={{ width: `${Math.min(p().percent, 100)}%` }} />
                    </div>
                    <div class="flex justify-between mt-1 text-xs text-txt-muted">
                      <span>{p().filesCollected} / {p().filesTotal} files</span>
                      <span>{formatSize(p().bytesCollected)}</span>
                    </div>
                    <Show when={p().currentFile}>
                      <div class="text-xs text-txt-muted mt-1 truncate" title={p().currentFile}>{p().currentFile}</div>
                    </Show>
                  </div>
                );
              }}
            </Show>

            {/* Cancel button */}
            <button class="btn btn-secondary w-full gap-2" onClick={triage.handleCancelTriage}>
              <HiOutlineStop class="w-icon-sm h-icon-sm" />
              Cancel Triage
            </button>
          </Show>

          {/* ── RESULT PHASE ── */}
          <Show when={phase() === "done"}>
            {(_) => {
              const r = () => triage.triageResult()!;
              return (
                <div class="space-y-3">
                  <div class={`card border ${r().cancelled ? "border-warning/30" : "border-success/30"}`}>
                    <div class="flex items-center justify-between mb-3">
                      <div class="flex items-center gap-2">
                        <Show
                          when={!r().cancelled}
                          fallback={<><HiOutlineExclamationTriangle class="w-icon-sm h-icon-sm text-warning" /><span class="text-sm font-medium text-warning">Triage Cancelled</span></>}
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

                    {/* Stats */}
                    <div class="grid grid-cols-4 gap-2 mb-3">
                      <div class="stat-box"><div class="text-txt-muted text-2xs">Files</div><div class="text-sm font-semibold text-txt">{r().filesCollected}</div></div>
                      <div class="stat-box"><div class="text-txt-muted text-2xs">Size</div><div class="text-sm font-semibold text-txt">{formatSize(r().bytesCollected)}</div></div>
                      <div class="stat-box"><div class="text-txt-muted text-2xs">Time</div><div class="text-sm font-semibold text-txt">{r().durationSecs < 60 ? `${r().durationSecs.toFixed(1)}s` : `${Math.floor(r().durationSecs / 60)}m ${Math.floor(r().durationSecs % 60)}s`}</div></div>
                      <div class="stat-box"><div class="text-txt-muted text-2xs">Secrets</div><div class={`text-sm font-semibold ${r().secretFindings.length > 0 ? "text-warning" : "text-txt"}`}>{r().secretFindings.length}</div></div>
                    </div>

                    {/* Output */}
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
                      <div class="text-xs text-txt-muted mt-1.5">{r().filesSkipped} file{r().filesSkipped !== 1 ? "s" : ""} skipped</div>
                    </Show>
                  </div>

                  {/* Secret Findings */}
                  <Show when={r().secretFindings.length > 0}>
                    <div class="card border border-warning/30">
                      <div class="flex items-center gap-2 mb-2">
                        <HiOutlineShieldExclamation class="w-icon-sm h-icon-sm text-warning" />
                        <span class="text-xs font-medium text-warning">Credential & Secret Findings ({r().secretFindings.length})</span>
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

                  {/* Actions */}
                  <div class="flex gap-2">
                    <button class="btn btn-secondary flex-1" onClick={handleNewTriage}>
                      Run Another Triage
                    </button>
                    <button class="btn btn-primary flex-1" onClick={props.onBack}>
                      Back to Dashboard
                    </button>
                  </div>
                </div>
              );
            }}
          </Show>
        </div>
      </div>
    </div>
  );
};

// --- Sub-component ---

function SecretFindingRow(props: { finding: SecretFinding }) {
  const f = () => props.finding;
  return (
    <div class={`p-1.5 rounded border ${confidenceBg(f().confidence)}`}>
      <div class="flex items-center gap-2 mb-0.5">
        <span class={`text-2xs font-medium uppercase ${confidenceColor(f().confidence)}`}>{f().confidence}</span>
        <span class="text-xs font-medium text-txt">{f().secretType}</span>
      </div>
      <div class="text-xs text-txt-muted">{f().description}</div>
      <div class="font-mono text-compact text-txt-muted mt-0.5 truncate" title={f().preview}>{f().preview}</div>
      <div class="text-xs text-txt-muted mt-0.5 truncate" title={f().filePath}>{f().filePath}:{f().lineNumber}</div>
    </div>
  );
}

export default AcquireTriageView;
