// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * AcquireImageWizard — Step-by-step imaging wizard for CORE Acquire edition.
 *
 * Three-step flow inspired by FTK Imager:
 *   Step 1: Select Source (drive, folder, or files)
 *   Step 2: Configure Image (name, format, compression, case metadata)
 *   Step 3: Select Destination & Start
 *
 * Supports two imaging modes:
 *   - "physical" → E01 disk image (via ewf_create_image)
 *   - "logical"  → L01 logical evidence (via l01_create_image)
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
  type Accessor,
} from "solid-js";
import {
  HiOutlineCircleStack,
  HiOutlineFolder,
  HiOutlineDocument,
  HiOutlineArrowRight,
  HiOutlineArrowLeft,
  HiOutlineCheck,
  HiOutlineFolderOpen,
  HiOutlineXMark,
  HiOutlineExclamationTriangle,
} from "../icons";
import { SplitSizeSelector } from "../export/SplitSizeSelector";
import type { DriveInfo } from "../../api/drives";
import { listDrives, formatDriveSize } from "../../api/drives";
import type { ExportMode } from "../../hooks/export/types";

// =============================================================================
// Types
// =============================================================================

export interface AcquireImageWizardProps {
  /** "physical" for E01, "logical" for L01 */
  mode: Accessor<ExportMode>;
  /** Go back to dashboard */
  onBack: () => void;
  /** Called when imaging starts — returns handler props for the export state hooks */
  onStartImaging: (config: ImagingConfig) => void;
  /** Pre-filled source paths from source panel or queue */
  prefilledSources?: string[] | null;
}

/** Configuration produced by the wizard, consumed by the imaging handler */
export interface ImagingConfig {
  mode: "physical" | "logical";
  sources: string[];
  destination: string;
  imageName: string;
  // E01-specific
  ewfFormat?: string;
  compression: string;
  compressionMethod?: string;
  computeMd5: boolean;
  computeSha1: boolean;
  // Shared
  segmentSizeMb: number;
  caseNumber: string;
  evidenceNumber: string;
  examinerName: string;
  description: string;
  notes: string;
  // Post-acquisition verification
  hashSegments: boolean;
  hashSegmentsIndividually: boolean;
  segmentHashAlgorithm: string;
}

// =============================================================================
// Component
// =============================================================================

const AcquireImageWizard: Component<AcquireImageWizardProps> = (props) => {
  // ---- Step tracking ----
  const [step, setStep] = createSignal(1);

  // ---- Step 1: Sources ----
  const [sources, setSources] = createSignal<string[]>([]);
  const [drives, setDrives] = createSignal<DriveInfo[]>([]);
  const [drivesLoading, setDrivesLoading] = createSignal(false);

  // ---- Step 2: Configuration ----
  const [imageName, setImageName] = createSignal("evidence");
  const [ewfFormat, setEwfFormat] = createSignal("e01");
  const [compression, setCompression] = createSignal("none");
  const [compressionMethod, setCompressionMethod] = createSignal("deflate");
  const [computeMd5, setComputeMd5] = createSignal(true);
  const [computeSha1, setComputeSha1] = createSignal(false);
  const [segmentSizeMb, setSegmentSizeMb] = createSignal(2048);
  const [caseNumber, setCaseNumber] = createSignal("");
  const [evidenceNumber, setEvidenceNumber] = createSignal("");
  const [examinerName, setExaminerName] = createSignal("");
  const [description, setDescription] = createSignal("");
  const [notes, setNotes] = createSignal("");

  // ---- Post-acquisition verification ----
  const [hashSegments, setHashSegments] = createSignal(true);
  const [hashSegmentsIndividually, setHashSegmentsIndividually] = createSignal(false);
  const [segmentHashAlgorithm, setSegmentHashAlgorithm] = createSignal("SHA-256");

  // ---- Step 3: Destination ----
  const [destination, setDestination] = createSignal("");

  // ---- Derived ----
  const isPhysical = createMemo(() => props.mode() === "physical");
  const imageExtension = createMemo(() => isPhysical() ? ".E01" : ".L01");

  const canProceedStep1 = createMemo(() => sources().length > 0);
  const canProceedStep2 = createMemo(() => imageName().trim().length > 0);
  const canStart = createMemo(() =>
    sources().length > 0 &&
    imageName().trim().length > 0 &&
    destination().trim().length > 0
  );

  // ---- Drive loading ----
  const loadDrives = async () => {
    setDrivesLoading(true);
    try {
      const list = await listDrives();
      setDrives(list.filter(d => !d.isSystemDisk));
    } catch {
      // Silently handle — user can still add files/folders
    } finally {
      setDrivesLoading(false);
    }
  };

  onMount(() => {
    if (isPhysical()) loadDrives();
  });

  // Reactively merge prefilled sources whenever they change (works on initial
  // mount AND when the user clicks "Acquire" from the queue while the wizard
  // is already visible).
  createEffect(on(
    () => props.prefilledSources,
    (prefilled) => {
      if (prefilled && prefilled.length > 0) {
        const existing = new Set(sources());
        const newPaths = prefilled.filter(p => !existing.has(p));
        if (newPaths.length > 0) setSources(prev => [...prev, ...newPaths]);
      }
    },
  ));

  // ---- Handlers ----
  const handleAddFiles = async () => {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({ multiple: true, title: "Select source files" });
      if (selected) {
        const paths = Array.isArray(selected) ? selected : [selected];
        const newPaths = paths.filter(p => !sources().includes(p));
        if (newPaths.length > 0) setSources(prev => [...prev, ...newPaths]);
      }
    } catch { /* user cancelled */ }
  };

  const handleAddFolder = async () => {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({ directory: true, title: "Select source folder" });
      if (selected && !sources().includes(selected)) {
        setSources(prev => [...prev, selected]);
      }
    } catch { /* user cancelled */ }
  };

  const handleSelectDrive = (drive: DriveInfo) => {
    const path = drive.mountPoint;
    if (!sources().includes(path)) {
      setSources(prev => [...prev, path]);
    }
  };

  const handleRemoveSource = (index: number) => {
    setSources(prev => prev.filter((_, i) => i !== index));
  };

  const handleSelectDestination = async () => {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({ directory: true, title: "Select destination folder" });
      if (selected) setDestination(selected);
    } catch { /* user cancelled */ }
  };

  const handleStart = () => {
    const config: ImagingConfig = {
      mode: isPhysical() ? "physical" : "logical",
      sources: sources(),
      destination: destination(),
      imageName: imageName(),
      compression: compression(),
      computeMd5: computeMd5(),
      computeSha1: computeSha1(),
      segmentSizeMb: segmentSizeMb(),
      caseNumber: caseNumber(),
      evidenceNumber: evidenceNumber(),
      examinerName: examinerName(),
      description: description(),
      notes: notes(),
      hashSegments: hashSegments(),
      hashSegmentsIndividually: hashSegmentsIndividually(),
      segmentHashAlgorithm: segmentHashAlgorithm(),
    };
    if (isPhysical()) {
      config.ewfFormat = ewfFormat();
      config.compressionMethod = compressionMethod();
    }
    props.onStartImaging(config);
  };

  // ---- Step navigation ----
  const nextStep = () => setStep(s => Math.min(s + 1, 3));
  const prevStep = () => setStep(s => Math.max(s - 1, 1));

  // ---- Basename helper ----
  const basename = (path: string) => {
    const parts = path.replace(/\\/g, "/").split("/");
    return parts[parts.length - 1] || path;
  };

  return (
    <div class="acquire-wizard">
      {/* Wizard Header */}
      <div class="acquire-wizard-header">
        <button class="btn btn-ghost gap-1.5" onClick={props.onBack}>
          <HiOutlineArrowLeft class="w-4 h-4" />
          Back
        </button>
        <h2 class="text-lg font-medium text-txt">
          {isPhysical() ? "Create Disk Image (E01)" : "Create Logical Image (L01)"}
        </h2>
        <div class="w-20" />
      </div>

      {/* Step indicator */}
      <div class="acquire-steps">
        <For each={[
          { num: 1, label: "Select Source" },
          { num: 2, label: "Configure" },
          { num: 3, label: "Destination" },
        ]}>
          {(s) => (
            <div
              class="acquire-step"
              classList={{
                "acquire-step-active": step() === s.num,
                "acquire-step-done": step() > s.num,
              }}
            >
              <div class="acquire-step-circle">
                <Show when={step() > s.num} fallback={<span>{s.num}</span>}>
                  <HiOutlineCheck class="w-3.5 h-3.5" />
                </Show>
              </div>
              <span class="acquire-step-label">{s.label}</span>
            </div>
          )}
        </For>
      </div>

      {/* Step content */}
      <div class="acquire-wizard-body">
        {/* Step 1: Select Source */}
        <Show when={step() === 1}>
          <div class="acquire-wizard-step">
            <div class="space-y-4">
              <div>
                <h3 class="text-sm font-medium text-txt mb-1">
                  {isPhysical() ? "Select a source drive or files to image" : "Select files or folders to include"}
                </h3>
                <p class="text-xs text-txt-muted">
                  {isPhysical()
                    ? "Choose a physical drive, logical volume, or folder to create an E01 forensic image."
                    : "Choose files and folders to package into an L01 logical evidence container."}
                </p>
              </div>

              {/* Action buttons */}
              <div class="flex items-center gap-2">
                <Show when={!isPhysical()}>
                  <button class="btn btn-secondary gap-1.5" onClick={handleAddFiles}>
                    <HiOutlineDocument class="w-4 h-4" />
                    Add Files
                  </button>
                </Show>
                <button class="btn btn-secondary gap-1.5" onClick={handleAddFolder}>
                  <HiOutlineFolderOpen class="w-4 h-4" />
                  Add Folder
                </button>
              </div>

              {/* Drive list for physical mode */}
              <Show when={isPhysical()}>
                <div class="mt-4">
                  <div class="flex items-center justify-between mb-2">
                    <h4 class="text-xs font-medium text-txt-muted uppercase tracking-wider">
                      Available Drives
                    </h4>
                    <button
                      class="btn-text text-xs"
                      onClick={loadDrives}
                      disabled={drivesLoading()}
                    >
                      Refresh
                    </button>
                  </div>
                  <Show
                    when={!drivesLoading()}
                    fallback={
                      <div class="text-xs text-txt-muted py-4 text-center">
                        Scanning drives…
                      </div>
                    }
                  >
                    <Show
                      when={drives().length > 0}
                      fallback={
                        <div class="text-xs text-txt-muted py-4 text-center">
                          No external drives found. Connect a drive and click Refresh.
                        </div>
                      }
                    >
                      <div class="space-y-1.5">
                        <For each={drives()}>
                          {(drive) => {
                            const isSelected = createMemo(() =>
                              sources().includes(drive.mountPoint)
                            );
                            return (
                              <button
                                class="acquire-drive-item"
                                classList={{ "acquire-drive-selected": isSelected() }}
                                onClick={() => handleSelectDrive(drive)}
                              >
                                <HiOutlineCircleStack class="w-5 h-5 text-txt-muted shrink-0" />
                                <div class="flex-1 min-w-0 text-left">
                                  <div class="text-sm font-medium text-txt truncate">
                                    {drive.name || basename(drive.mountPoint)}
                                  </div>
                                  <div class="text-xs text-txt-muted">
                                    {drive.mountPoint} · {drive.fileSystem.toUpperCase()} · {formatDriveSize(drive.totalBytes)}
                                    {drive.isRemovable && " · Removable"}
                                  </div>
                                </div>
                                <Show when={isSelected()}>
                                  <HiOutlineCheck class="w-4 h-4 text-accent shrink-0" />
                                </Show>
                                <Show when={drive.isSystemDisk}>
                                  <HiOutlineExclamationTriangle class="w-4 h-4 text-warning shrink-0" />
                                </Show>
                              </button>
                            );
                          }}
                        </For>
                      </div>
                    </Show>
                  </Show>
                </div>
              </Show>

              {/* Selected sources list */}
              <Show when={sources().length > 0}>
                <div class="mt-4">
                  <h4 class="text-xs font-medium text-txt-muted uppercase tracking-wider mb-2">
                    Selected Sources ({sources().length})
                  </h4>
                  <div class="space-y-1">
                    <For each={sources()}>
                      {(source, index) => (
                        <div class="flex items-center gap-2 px-3 py-2 bg-bg-secondary rounded-lg border border-border">
                          <HiOutlineFolder class="w-4 h-4 text-txt-muted shrink-0" />
                          <span class="text-sm text-txt truncate flex-1">{source}</span>
                          <button
                            class="icon-btn-sm text-txt-muted hover:text-error"
                            onClick={() => handleRemoveSource(index())}
                          >
                            <HiOutlineXMark class="w-3.5 h-3.5" />
                          </button>
                        </div>
                      )}
                    </For>
                  </div>
                </div>
              </Show>
            </div>
          </div>
        </Show>

        {/* Step 2: Configure Image */}
        <Show when={step() === 2}>
          <div class="acquire-wizard-step">
            <div class="space-y-4">
              {/* Image Name */}
              <div class="form-group">
                <label class="label">Image Name</label>
                <div class="flex items-center gap-2">
                  <input
                    class="input flex-1"
                    type="text"
                    value={imageName()}
                    onInput={(e) => setImageName(e.currentTarget.value)}
                    placeholder="evidence"
                  />
                  <span class="text-sm text-txt-muted">{imageExtension()}</span>
                </div>
              </div>

              {/* E01 Format (physical only) */}
              <Show when={isPhysical()}>
                <div class="form-group">
                  <label class="label">Image Format</label>
                  <select
                    class="input"
                    value={ewfFormat()}
                    onChange={(e) => setEwfFormat(e.currentTarget.value)}
                  >
                    <option value="e01">E01 (EnCase 5/6 — most compatible)</option>
                    <option value="encase6">EnCase 6</option>
                    <option value="encase7">EnCase 7 (.Ex01)</option>
                    <option value="v2encase7">V2 EnCase 7</option>
                    <option value="ftk">FTK Imager (SMART)</option>
                  </select>
                </div>
              </Show>

              {/* Compression */}
              <div class="form-group">
                <label class="label">Compression</label>
                <div class="grid grid-cols-2 gap-2">
                  <select
                    class="input"
                    value={compression()}
                    onChange={(e) => setCompression(e.currentTarget.value)}
                  >
                    <option value="none">No compression (fastest)</option>
                    <option value="fast">Fast</option>
                    <option value="best">Best (smallest)</option>
                  </select>
                  <Show when={isPhysical()}>
                    <select
                      class="input"
                      value={compressionMethod()}
                      onChange={(e) => setCompressionMethod(e.currentTarget.value)}
                    >
                      <option value="deflate">Deflate (standard)</option>
                      <option value="bzip2" disabled={!["encase7", "v2encase7"].includes(ewfFormat())}>
                        Bzip2 (EnCase 7+ only)
                      </option>
                    </select>
                  </Show>
                </div>
              </div>

              {/* Hash options */}
              <div class="form-group">
                <label class="label">Hash Verification</label>
                <Show
                  when={isPhysical()}
                  fallback={
                    <p class="text-xs text-txt-muted">
                      L01 containers always compute and embed both MD5 and SHA-1 hashes.
                    </p>
                  }
                >
                  <div class="flex items-center gap-4">
                    <label class="flex items-center gap-2 text-sm text-txt cursor-pointer">
                      <input
                        type="checkbox"
                        checked={computeMd5()}
                        onChange={(e) => setComputeMd5(e.currentTarget.checked)}
                      />
                      MD5
                    </label>
                    <label class="flex items-center gap-2 text-sm text-txt cursor-pointer">
                      <input
                        type="checkbox"
                        checked={computeSha1()}
                        onChange={(e) => setComputeSha1(e.currentTarget.checked)}
                      />
                      SHA-1
                    </label>
                  </div>
                </Show>
              </div>

              {/* Segment Size */}
              <SplitSizeSelector
                valueMb={segmentSizeMb}
                setValueMb={setSegmentSizeMb}
                label="Segment Size"
              />

              {/* Post-acquisition segment verification */}
              <div class="form-group">
                <label class="label">Post-Acquisition Verification</label>
                <div class="space-y-2">
                  <label class="flex items-center gap-2 text-sm text-txt cursor-pointer">
                    <input
                      type="checkbox"
                      checked={hashSegments()}
                      onChange={(e) => setHashSegments(e.currentTarget.checked)}
                    />
                    Verify container (hash all segments as one stream)
                  </label>
                  <label class="flex items-center gap-2 text-sm text-txt cursor-pointer">
                    <input
                      type="checkbox"
                      checked={hashSegmentsIndividually()}
                      onChange={(e) => setHashSegmentsIndividually(e.currentTarget.checked)}
                    />
                    Hash each segment file individually
                  </label>
                  <Show when={hashSegments() || hashSegmentsIndividually()}>
                    <div class="pl-6">
                      <select
                        class="input-sm w-40"
                        value={segmentHashAlgorithm()}
                        onChange={(e) => setSegmentHashAlgorithm(e.currentTarget.value)}
                      >
                        <option value="MD5">MD5</option>
                        <option value="SHA-1">SHA-1</option>
                        <option value="SHA-256">SHA-256 (recommended)</option>
                        <option value="SHA-512">SHA-512</option>
                        <option value="BLAKE3">BLAKE3</option>
                      </select>
                    </div>
                  </Show>
                </div>
                <p class="text-xs text-txt-muted mt-1">
                  Hashes created segment files after imaging to verify integrity.
                </p>
              </div>

              {/* Case Metadata */}
              <details class="acquire-details">
                <summary class="acquire-details-summary">
                  Case Information (optional)
                </summary>
                <div class="acquire-details-content space-y-3">
                  <div class="grid grid-cols-2 gap-2">
                    <div class="form-group">
                      <label class="label">Case Number</label>
                      <input
                        class="input"
                        type="text"
                        value={caseNumber()}
                        onInput={(e) => setCaseNumber(e.currentTarget.value)}
                        placeholder="e.g. 2024-001"
                      />
                    </div>
                    <div class="form-group">
                      <label class="label">Evidence Number</label>
                      <input
                        class="input"
                        type="text"
                        value={evidenceNumber()}
                        onInput={(e) => setEvidenceNumber(e.currentTarget.value)}
                        placeholder="e.g. EV-001"
                      />
                    </div>
                  </div>
                  <div class="form-group">
                    <label class="label">Examiner Name</label>
                    <input
                      class="input"
                      type="text"
                      value={examinerName()}
                      onInput={(e) => setExaminerName(e.currentTarget.value)}
                      placeholder="e.g. Jane Doe"
                    />
                  </div>
                  <div class="form-group">
                    <label class="label">Description</label>
                    <input
                      class="input"
                      type="text"
                      value={description()}
                      onInput={(e) => setDescription(e.currentTarget.value)}
                      placeholder="e.g. Western Digital HDD from suspect PC"
                    />
                  </div>
                  <div class="form-group">
                    <label class="label">Notes</label>
                    <textarea
                      class="textarea"
                      rows="2"
                      value={notes()}
                      onInput={(e) => setNotes(e.currentTarget.value)}
                      placeholder="Additional notes…"
                    />
                  </div>
                </div>
              </details>
            </div>
          </div>
        </Show>

        {/* Step 3: Destination */}
        <Show when={step() === 3}>
          <div class="acquire-wizard-step">
            <div class="space-y-4">
              <div>
                <h3 class="text-sm font-medium text-txt mb-1">Select Destination</h3>
                <p class="text-xs text-txt-muted">
                  Choose where to save the {isPhysical() ? "E01 disk image" : "L01 logical image"}.
                  Make sure the destination has enough free space.
                </p>
              </div>

              <div class="flex items-center gap-2">
                <input
                  class="input flex-1"
                  type="text"
                  value={destination()}
                  readOnly
                  placeholder="Click Browse to select a destination folder…"
                />
                <button class="btn btn-secondary" onClick={handleSelectDestination}>
                  Browse
                </button>
              </div>

              {/* Summary */}
              <Show when={canStart()}>
                <div class="acquire-summary">
                  <h4 class="text-xs font-medium text-txt-muted uppercase tracking-wider mb-3">
                    Review
                  </h4>
                  <div class="space-y-2">
                    <div class="acquire-summary-row">
                      <span class="text-txt-muted">Type</span>
                      <span class="text-txt font-medium">
                        {isPhysical() ? "E01 Disk Image" : "L01 Logical Image"}
                      </span>
                    </div>
                    <div class="acquire-summary-row">
                      <span class="text-txt-muted">Source(s)</span>
                      <span class="text-txt">{sources().length} item{sources().length !== 1 ? "s" : ""}</span>
                    </div>
                    <div class="acquire-summary-row">
                      <span class="text-txt-muted">Output</span>
                      <span class="text-txt font-mono text-compact truncate">
                        {destination()}/{imageName()}{imageExtension()}
                      </span>
                    </div>
                    <div class="acquire-summary-row">
                      <span class="text-txt-muted">Compression</span>
                      <span class="text-txt">{compression() === "none" ? "None" : compression()}</span>
                    </div>
                    <div class="acquire-summary-row">
                      <span class="text-txt-muted">Segment Size</span>
                      <span class="text-txt">
                        {segmentSizeMb() === 0 ? "No splitting" : `${segmentSizeMb()} MB`}
                      </span>
                    </div>
                    <Show when={caseNumber()}>
                      <div class="acquire-summary-row">
                        <span class="text-txt-muted">Case #</span>
                        <span class="text-txt">{caseNumber()}</span>
                      </div>
                    </Show>
                  </div>
                </div>
              </Show>
            </div>
          </div>
        </Show>
      </div>

      {/* Wizard Footer */}
      <div class="acquire-wizard-footer">
        <Show
          when={step() > 1}
          fallback={<div />}
        >
          <button class="btn btn-secondary gap-1.5" onClick={prevStep}>
            <HiOutlineArrowLeft class="w-4 h-4" />
            Previous
          </button>
        </Show>

        <Show
          when={step() < 3}
          fallback={
            <button
              class="btn btn-primary gap-1.5"
              disabled={!canStart()}
              onClick={handleStart}
            >
              {isPhysical() ? "Create E01 Image" : "Create L01 Image"}
              <HiOutlineArrowRight class="w-4 h-4" />
            </button>
          }
        >
          <button
            class="btn btn-primary gap-1.5"
            disabled={step() === 1 ? !canProceedStep1() : !canProceedStep2()}
            onClick={nextStep}
          >
            Next
            <HiOutlineArrowRight class="w-4 h-4" />
          </button>
        </Show>
      </div>
    </div>
  );
};

export default AcquireImageWizard;
