// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * MergeProjectsWizard — Modal wizard for merging multiple .cffx projects.
 *
 * Steps:
 * 1. Select .cffx files to merge
 * 2. Review project summaries and configure merge settings
 * 3. Execute merge and show results
 *
 * Sub-components live in sibling files:
 *   - SelectStep, ProjectSummaryCard, GlobalExaminersPanel
 *   - MergingStep, CompleteStep
 *   - helpers.ts (formatting), types.ts (interfaces)
 */

import { Component, createSignal, For, Show, createMemo } from "solid-js";
import { open, save } from "@tauri-apps/plugin-dialog";
import {
  HiOutlineDocumentDuplicate,
  HiOutlineExclamationTriangle,
  HiOutlineXMark,
  HiOutlineFolder,
} from "../icons";
import { analyzeProjects, executeMerge } from "../../api/projectMerge";
import type {
  MergeProjectsWizardProps,
  WizardStep,
  ProjectMergeSummary,
  MergeResult,
  MergeSourceAssignment,
  GlobalExaminer,
} from "./types";
import { SelectStep } from "./SelectStep";
import { ProjectSummaryCard } from "./ProjectSummaryCard";
import { GlobalExaminersPanel } from "./GlobalExaminersPanel";
import { MergingStep } from "./MergingStep";
import { CompleteStep } from "./CompleteStep";

const MergeProjectsWizard: Component<MergeProjectsWizardProps> = (props) => {
  // --- State ---
  const [step, setStep] = createSignal<WizardStep>("select");
  const [cffxPaths, setCffxPaths] = createSignal<string[]>([]);
  const [summaries, setSummaries] = createSignal<ProjectMergeSummary[]>([]);
  const [ownerOverrides, setOwnerOverrides] = createSignal<Record<string, string>>({});
  const [mergedName, setMergedName] = createSignal("Merged Project");
  const [outputPath, setOutputPath] = createSignal("");
  const [mergeResult, setMergeResult] = createSignal<MergeResult | null>(null);
  const [error, setError] = createSignal<string | null>(null);
  const [analyzing, setAnalyzing] = createSignal(false);
  const [expandedSections, setExpandedSections] = createSignal<Set<string>>(new Set());
  const [customOwnerMode, setCustomOwnerMode] = createSignal<Set<string>>(new Set());

  // --- Section expand/collapse ---
  const toggleSection = (key: string) => {
    setExpandedSections((prev) => {
      const next = new Set(prev);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return next;
    });
  };
  const isSectionExpanded = (key: string) => expandedSections().has(key);

  // --- Custom owner toggle ---
  const toggleCustomOwner = (cffxPath: string) => {
    setCustomOwnerMode((prev) => {
      const next = new Set(prev);
      if (next.has(cffxPath)) next.delete(cffxPath);
      else next.add(cffxPath);
      return next;
    });
  };
  const isCustomOwner = (cffxPath: string) => customOwnerMode().has(cffxPath);

  // --- Derived state ---
  const canProceedToReview = createMemo(() => cffxPaths().length >= 2);

  const globalExaminers = createMemo<GlobalExaminer[]>(() => {
    const seen = new Map<string, GlobalExaminer>();
    for (const s of summaries()) {
      for (const ex of s.examiners) {
        const key = (ex.displayName || ex.name).toLowerCase();
        const existing = seen.get(key);
        if (existing) {
          if (!existing.projects.includes(s.name)) existing.projects.push(s.name);
        } else {
          seen.set(key, { ...ex, projects: [s.name] });
        }
      }
    }
    return [...seen.values()];
  });

  const totalStats = createMemo(() => {
    const s = summaries();
    return {
      evidence: s.reduce((a, p) => a + p.evidenceFileCount, 0),
      hashes: s.reduce((a, p) => a + p.hashCount, 0),
      sessions: s.reduce((a, p) => a + p.sessionCount, 0),
      activity: s.reduce((a, p) => a + p.activityCount, 0),
      bookmarks: s.reduce((a, p) => a + p.bookmarkCount, 0),
      notes: s.reduce((a, p) => a + p.noteCount, 0),
      reports: s.reduce((a, p) => a + p.reportCount, 0),
    };
  });

  // --- Step 1: Add project files ---
  const handleAddProjects = async () => {
    const selected = await open({
      multiple: true,
      filters: [{ name: "CORE-FFX Projects", extensions: ["cffx"] }],
      title: "Select .cffx project files to merge",
    });
    if (selected) {
      const paths = Array.isArray(selected) ? selected : [selected];
      const newPaths = [...cffxPaths()];
      for (const p of paths) {
        if (!newPaths.includes(p)) newPaths.push(p);
      }
      setCffxPaths(newPaths);
    }
  };

  const handleRemoveProject = (path: string) => {
    setCffxPaths((prev) => prev.filter((p) => p !== path));
    setSummaries((prev) => prev.filter((s) => s.cffxPath !== path));
  };

  // --- Step 1 → 2: Analyze ---
  const handleAnalyze = async () => {
    setAnalyzing(true);
    setError(null);
    try {
      const results = await analyzeProjects(cffxPaths());
      setSummaries(results);

      // Initialize owner overrides
      const initialOwners: Record<string, string> = {};
      for (const s of results) {
        if (s.ownerName) {
          initialOwners[s.cffxPath] = s.ownerName;
        } else if (s.examiners.length > 0) {
          const owner =
            s.examiners.find((e) => e.role === "project owner") ||
            s.examiners.find((e) => e.role === "session user") ||
            s.examiners[0];
          initialOwners[s.cffxPath] = owner.displayName || owner.name;
        } else {
          initialOwners[s.cffxPath] = "";
        }
      }
      setOwnerOverrides(initialOwners);

      if (results.length > 0 && mergedName() === "Merged Project") {
        setMergedName(results[0].name);
      }
      if (results.length > 0 && !outputPath()) {
        const firstPath = results[0].cffxPath;
        const dir = firstPath.substring(0, firstPath.lastIndexOf("/"));
        setOutputPath(`${dir}/${results[0].name}_merged.cffx`);
      }

      setStep("review");
    } catch (e) {
      setError(`Failed to analyze projects: ${e}`);
    } finally {
      setAnalyzing(false);
    }
  };

  // --- Step 2: Choose output path ---
  const handleChooseOutput = async () => {
    const selected = await save({
      filters: [{ name: "CORE-FFX Project", extensions: ["cffx"] }],
      title: "Save merged project as",
      defaultPath: outputPath() || undefined,
    });
    if (selected) {
      setOutputPath(selected);
      const name = selected.split("/").pop()?.replace(".cffx", "") || mergedName();
      setMergedName(name);
    }
  };

  // --- Step 2 → 3: Execute merge ---
  const handleMerge = async () => {
    if (!outputPath()) {
      setError("Please choose an output path");
      return;
    }
    setStep("merging");
    setError(null);
    try {
      const overrides = ownerOverrides();
      const ownerAssignments: MergeSourceAssignment[] = cffxPaths().map((path) => ({
        cffxPath: path,
        ownerName: overrides[path] || "",
      }));
      const result = await executeMerge(cffxPaths(), outputPath(), mergedName(), undefined, ownerAssignments);
      setMergeResult(result);
      if (result.success) {
        setStep("complete");
      } else {
        setError(result.error || "Unknown merge error");
        setStep("review");
      }
    } catch (e) {
      setError(`Merge failed: ${e}`);
      setStep("review");
    }
  };

  // --- Step 4: Load merged project ---
  const handleLoadMerged = () => {
    const result = mergeResult();
    if (result?.cffxPath) props.onMergeComplete?.(result.cffxPath);
    props.onClose();
  };

  return (
    <div class="modal-overlay" onClick={(e) => e.target === e.currentTarget && props.onClose()}>
      <div class="modal-content w-[680px] max-h-[85vh] flex flex-col">
        {/* Header */}
        <div class="modal-header">
          <div class="flex items-center gap-2">
            <HiOutlineDocumentDuplicate class="w-5 h-5 text-accent" />
            <h2 class="text-lg font-semibold text-txt">Merge Projects</h2>
          </div>
          <button class="icon-btn-sm" onClick={props.onClose}>
            <HiOutlineXMark class="w-4 h-4" />
          </button>
        </div>

        {/* Step indicator */}
        <div class="px-5 pt-3 pb-2 flex items-center gap-2 text-xs text-txt-muted border-b border-border">
          <span classList={{ "text-accent font-semibold": step() === "select" }}>1. Select</span>
          <span>→</span>
          <span classList={{ "text-accent font-semibold": step() === "review" }}>2. Review</span>
          <span>→</span>
          <span classList={{ "text-accent font-semibold": step() === "merging" || step() === "complete" }}>3. Merge</span>
        </div>

        {/* Body */}
        <div class="modal-body overflow-y-auto flex-1">
          {/* Error */}
          <Show when={error()}>
            <div class="flex items-center gap-2 p-3 rounded-lg bg-error/10 text-error text-sm mb-4">
              <HiOutlineExclamationTriangle class="w-4 h-4 shrink-0" />
              <span>{error()}</span>
            </div>
          </Show>

          {/* Step 1 */}
          <Show when={step() === "select"}>
            <SelectStep
              cffxPaths={cffxPaths()}
              onAddProjects={handleAddProjects}
              onRemoveProject={handleRemoveProject}
            />
          </Show>

          {/* Step 2 */}
          <Show when={step() === "review"}>
            <div class="col gap-4">
              {/* Project summaries */}
              <div>
                <h3 class="text-sm font-semibold text-txt mb-2">Projects to Merge</h3>
                <div class="col gap-3">
                  <For each={summaries()}>
                    {(summary) => (
                      <ProjectSummaryCard
                        summary={summary}
                        ownerOverride={ownerOverrides()[summary.cffxPath] || ""}
                        globalExaminers={globalExaminers()}
                        isCustomOwner={isCustomOwner(summary.cffxPath)}
                        onToggleCustomOwner={() => toggleCustomOwner(summary.cffxPath)}
                        onOwnerChange={(val) =>
                          setOwnerOverrides((prev) => ({ ...prev, [summary.cffxPath]: val }))
                        }
                        isSectionExpanded={isSectionExpanded}
                        onToggleSection={toggleSection}
                      />
                    )}
                  </For>
                </div>
              </div>

              {/* Global examiners */}
              <Show when={globalExaminers().length > 0}>
                <GlobalExaminersPanel
                  examiners={globalExaminers()}
                  isExpanded={isSectionExpanded("__global_examiners")}
                  onToggle={() => toggleSection("__global_examiners")}
                />
              </Show>

              {/* Merge totals */}
              <div class="p-3 rounded-lg bg-bg-panel border border-border">
                <h3 class="text-sm font-semibold text-txt mb-2">Merge Summary (before dedup)</h3>
                <div class="grid grid-cols-4 gap-2 text-xs text-txt-secondary">
                  <span>Evidence: {totalStats().evidence}</span>
                  <span>Hashes: {totalStats().hashes}</span>
                  <span>Sessions: {totalStats().sessions}</span>
                  <span>Activity: {totalStats().activity}</span>
                  <span>Bookmarks: {totalStats().bookmarks}</span>
                  <span>Notes: {totalStats().notes}</span>
                  <span>Reports: {totalStats().reports}</span>
                  <span>Projects: {summaries().length}</span>
                </div>
              </div>

              {/* Merged project settings */}
              <div>
                <h3 class="text-sm font-semibold text-txt mb-2">Merged Project</h3>
                <div class="col gap-3">
                  <div class="form-group">
                    <label class="label">Project Name</label>
                    <input
                      class="input"
                      value={mergedName()}
                      onInput={(e) => setMergedName(e.currentTarget.value)}
                      placeholder="Merged project name"
                    />
                  </div>
                  <div class="form-group">
                    <label class="label">Output Path</label>
                    <div class="flex items-center gap-2">
                      <input
                        class="input flex-1"
                        value={outputPath()}
                        onInput={(e) => setOutputPath(e.currentTarget.value)}
                        placeholder="Path for the merged .cffx file"
                      />
                      <button class="btn btn-secondary" onClick={handleChooseOutput}>
                        <HiOutlineFolder class="w-4 h-4" />
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </Show>

          {/* Step 3 */}
          <Show when={step() === "merging"}>
            <MergingStep projectCount={summaries().length} />
          </Show>

          {/* Step 4 */}
          <Show when={step() === "complete"}>
            <CompleteStep mergeResult={mergeResult()} />
          </Show>
        </div>

        {/* Footer */}
        <div class="modal-footer justify-end">
          <Show when={step() === "select"}>
            <button class="btn btn-secondary" onClick={props.onClose}>Cancel</button>
            <button
              class="btn btn-primary"
              disabled={!canProceedToReview() || analyzing()}
              onClick={handleAnalyze}
            >
              {analyzing() ? "Analyzing…" : "Review & Merge"}
            </button>
          </Show>
          <Show when={step() === "review"}>
            <button class="btn btn-secondary" onClick={() => setStep("select")}>Back</button>
            <button
              class="btn btn-primary"
              disabled={!outputPath() || !mergedName()}
              onClick={handleMerge}
            >
              Merge Projects
            </button>
          </Show>
          <Show when={step() === "complete"}>
            <button class="btn btn-secondary" onClick={props.onClose}>Close</button>
            <Show when={mergeResult()?.cffxPath}>
              <button class="btn btn-primary" onClick={handleLoadMerged}>
                Open Merged Project
              </button>
            </Show>
          </Show>
        </div>
      </div>
    </div>
  );
};

export default MergeProjectsWizard;
