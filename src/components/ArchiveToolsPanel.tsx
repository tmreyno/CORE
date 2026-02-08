// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ArchiveToolsPanel - Archive testing, repair, validation, and extraction tools
 * 
 * Provides forensic archive operations:
 * - Test: Verify archive integrity without extraction
 * - Repair: Recover corrupted archives
 * - Validate: Detailed validation with error context
 * - Split Extract: Extract multi-volume archives
 */

import { createSignal, Show, For } from "solid-js";
import { open, save } from "@tauri-apps/plugin-dialog";
import {
  HiOutlineCheckCircle,
  HiOutlineXCircle,
  HiOutlineWrench,
  HiOutlineDocumentMagnifyingGlass,
  HiOutlineArchiveBox,
  HiOutlineFolderOpen,
  HiOutlinePlay,
  HiOutlineInformationCircle,
  HiOutlineExclamationTriangle,
  HiOutlineDocumentText,
} from "./icons";
import {
  testArchive,
  repairArchive,
  validateArchive,
  extractSplitArchive,
  listenToRepairProgress,
  listenToSplitExtractProgress,
  getLastArchiveError,
  clearLastArchiveError,
  type ArchiveValidationResult,
  type DetailedArchiveError,
  type ArchiveRepairProgress,
  type SplitExtractProgress,
} from "../api/archiveCreate";
import { useToast } from "./Toast";

type ToolTab = "test" | "repair" | "validate" | "extract";

export interface ArchiveToolsPanelProps {
  onClose?: () => void;
}

export function ArchiveToolsPanel(props: ArchiveToolsPanelProps) {
  const toast = useToast();

  // Tab state
  const [activeTab, setActiveTab] = createSignal<ToolTab>("test");

  // === Test Tab State ===
  const [testArchivePath, setTestArchivePath] = createSignal("");
  const [testPassword, setTestPassword] = createSignal("");
  const [testResult, setTestResult] = createSignal<boolean | null>(null);
  const [testInProgress, setTestInProgress] = createSignal(false);

  // === Repair Tab State ===
  const [repairCorruptedPath, setRepairCorruptedPath] = createSignal("");
  const [repairOutputPath, setRepairOutputPath] = createSignal("");
  const [repairProgress, setRepairProgress] = createSignal(0);
  const [repairStatus, setRepairStatus] = createSignal("");
  const [repairInProgress, setRepairInProgress] = createSignal(false);
  const [repairResult, setRepairResult] = createSignal("");

  // === Validate Tab State ===
  const [validateArchivePath, setValidateArchivePath] = createSignal("");
  const [validateResult, setValidateResult] = createSignal<ArchiveValidationResult | null>(null);
  const [validateInProgress, setValidateInProgress] = createSignal(false);
  const [lastError, setLastError] = createSignal<DetailedArchiveError | null>(null);

  // === Extract Tab State ===
  const [extractFirstVolume, setExtractFirstVolume] = createSignal("");
  const [extractOutputDir, setExtractOutputDir] = createSignal("");
  const [extractPassword, setExtractPassword] = createSignal("");
  const [extractProgress, setExtractProgress] = createSignal(0);
  const [extractStatus, setExtractStatus] = createSignal("");
  const [extractInProgress, setExtractInProgress] = createSignal(false);
  const [extractResult, setExtractResult] = createSignal("");

  // === Test Archive Handler ===
  const handleTestArchive = async () => {
    if (!testArchivePath()) {
      toast.error("No Archive", "Please select an archive to test");
      return;
    }

    setTestInProgress(true);
    setTestResult(null);

    try {
      const isValid = await testArchive(
        testArchivePath(),
        testPassword() || undefined
      );

      setTestResult(isValid);

      if (isValid) {
        toast.success("Test Passed", "Archive integrity verified successfully");
      } else {
        toast.error("Test Failed", "Archive integrity check failed");
        
        // Try to get detailed error info
        const error = await getLastArchiveError();
        if (error) {
          setLastError(error);
        }
      }
    } catch (error: any) {
      toast.error("Test Error", error.message || String(error));
      setTestResult(false);
    } finally {
      setTestInProgress(false);
    }
  };

  // === Repair Archive Handler ===
  const handleRepairArchive = async () => {
    if (!repairCorruptedPath()) {
      toast.error("No Archive", "Please select a corrupted archive");
      return;
    }

    if (!repairOutputPath()) {
      toast.error("No Output", "Please specify output path for repaired archive");
      return;
    }

    setRepairInProgress(true);
    setRepairProgress(0);
    setRepairStatus("Initializing...");
    setRepairResult("");

    const unlisten = await listenToRepairProgress((progress: ArchiveRepairProgress) => {
      setRepairProgress(progress.percent);
      setRepairStatus(progress.status);
    });

    try {
      const result = await repairArchive(
        repairCorruptedPath(),
        repairOutputPath()
      );

      setRepairResult(result);
      toast.success("Repair Complete", `Repaired archive saved to: ${result}`);
    } catch (error: any) {
      toast.error("Repair Failed", error.message || String(error));
      setRepairResult("");
    } finally {
      unlisten();
      setRepairInProgress(false);
      setRepairStatus("");
    }
  };

  // === Validate Archive Handler ===
  const handleValidateArchive = async () => {
    if (!validateArchivePath()) {
      toast.error("No Archive", "Please select an archive to validate");
      return;
    }

    setValidateInProgress(true);
    setValidateResult(null);
    setLastError(null);

    try {
      const result = await validateArchive(validateArchivePath());
      setValidateResult(result);

      if (result.isValid) {
        toast.success("Validation Passed", "Archive is valid and intact");
      } else {
        toast.warning("Validation Issues", result.errorMessage || "Archive has problems");
        
        // Try to get detailed error
        const error = await getLastArchiveError();
        if (error) {
          setLastError(error);
        }
      }
    } catch (error: any) {
      toast.error("Validation Error", error.message || String(error));
      setValidateResult({
        isValid: false,
        errorMessage: error.message || String(error),
      });
    } finally {
      setValidateInProgress(false);
    }
  };

  // === Extract Split Archive Handler ===
  const handleExtractSplit = async () => {
    if (!extractFirstVolume()) {
      toast.error("No Archive", "Please select the first volume (.001)");
      return;
    }

    if (!extractOutputDir()) {
      toast.error("No Output", "Please select output directory");
      return;
    }

    setExtractInProgress(true);
    setExtractProgress(0);
    setExtractStatus("Initializing...");
    setExtractResult("");

    const unlisten = await listenToSplitExtractProgress((progress: SplitExtractProgress) => {
      setExtractProgress(progress.percent);
      setExtractStatus(progress.status);
    });

    try {
      const result = await extractSplitArchive(
        extractFirstVolume(),
        extractOutputDir(),
        extractPassword() || undefined
      );

      setExtractResult(result);
      toast.success("Extraction Complete", `Files extracted to: ${result}`);
    } catch (error: any) {
      toast.error("Extraction Failed", error.message || String(error));
      setExtractResult("");
    } finally {
      unlisten();
      setExtractInProgress(false);
      setExtractStatus("");
    }
  };

  // === File Picker Handlers ===
  const handleSelectTestArchive = async () => {
    const selected = await open({
      multiple: false,
      directory: false,
      filters: [{ name: "Archives", extensions: ["7z", "zip", "rar"] }],
      title: "Select Archive to Test",
    });
    if (selected) setTestArchivePath(selected as string);
  };

  const handleSelectRepairInput = async () => {
    const selected = await open({
      multiple: false,
      directory: false,
      filters: [{ name: "Archives", extensions: ["7z", "zip"] }],
      title: "Select Corrupted Archive",
    });
    if (selected) setRepairCorruptedPath(selected as string);
  };

  const handleSelectRepairOutput = async () => {
    const selected = await save({
      filters: [{ name: "7z Archive", extensions: ["7z"] }],
      defaultPath: "repaired.7z",
      title: "Save Repaired Archive As",
    });
    if (selected) setRepairOutputPath(selected as string);
  };

  const handleSelectValidateArchive = async () => {
    const selected = await open({
      multiple: false,
      directory: false,
      filters: [{ name: "Archives", extensions: ["7z", "zip", "rar"] }],
      title: "Select Archive to Validate",
    });
    if (selected) setValidateArchivePath(selected as string);
  };

  const handleSelectFirstVolume = async () => {
    const selected = await open({
      multiple: false,
      directory: false,
      filters: [{ name: "Split Archives", extensions: ["001", "7z.001"] }],
      title: "Select First Volume",
    });
    if (selected) setExtractFirstVolume(selected as string);
  };

  const handleSelectExtractOutput = async () => {
    const selected = await open({
      multiple: false,
      directory: true,
      title: "Select Output Directory",
    });
    if (selected) setExtractOutputDir(selected as string);
  };

  return (
    <div class="modal-overlay">
      <div class="modal-content w-[700px] max-h-[80vh]">
        {/* Header */}
        <div class="modal-header">
          <h2 class="flex items-center gap-2">
            <HiOutlineArchiveBox class="w-5 h-5 text-accent" />
            Archive Tools
          </h2>
          <button
            class="icon-btn-sm"
            onClick={props.onClose}
            aria-label="Close"
          >
            <HiOutlineXCircle class="w-4 h-4" />
          </button>
        </div>

        {/* Tab Navigation */}
        <div class="flex gap-1 border-b border-border px-5">
          <button
            class={`px-4 py-2 -mb-px border-b-2 transition-colors ${
              activeTab() === "test"
                ? "border-accent text-accent"
                : "border-transparent text-txt-secondary hover:text-txt"
            }`}
            onClick={() => setActiveTab("test")}
          >
            <HiOutlineCheckCircle class="inline w-4 h-4 mr-1.5" />
            Test
          </button>
          <button
            class={`px-4 py-2 -mb-px border-b-2 transition-colors ${
              activeTab() === "repair"
                ? "border-accent text-accent"
                : "border-transparent text-txt-secondary hover:text-txt"
            }`}
            onClick={() => setActiveTab("repair")}
          >
            <HiOutlineWrench class="inline w-4 h-4 mr-1.5" />
            Repair
          </button>
          <button
            class={`px-4 py-2 -mb-px border-b-2 transition-colors ${
              activeTab() === "validate"
                ? "border-accent text-accent"
                : "border-transparent text-txt-secondary hover:text-txt"
            }`}
            onClick={() => setActiveTab("validate")}
          >
            <HiOutlineDocumentMagnifyingGlass class="inline w-4 h-4 mr-1.5" />
            Validate
          </button>
          <button
            class={`px-4 py-2 -mb-px border-b-2 transition-colors ${
              activeTab() === "extract"
                ? "border-accent text-accent"
                : "border-transparent text-txt-secondary hover:text-txt"
            }`}
            onClick={() => setActiveTab("extract")}
          >
            <HiOutlineArchiveBox class="inline w-4 h-4 mr-1.5" />
            Extract Split
          </button>
        </div>

        {/* Modal Body */}
        <div class="modal-body overflow-y-auto">
          {/* TEST TAB */}
          <Show when={activeTab() === "test"}>
            <div class="col gap-4">
              <div class="info-card">
                <HiOutlineInformationCircle class="w-5 h-5 text-info" />
                <div>
                  <div class="font-medium text-txt">Test Archive Integrity</div>
                  <div class="text-sm text-txt-secondary">
                    Verify archive integrity without extracting files. Fast and non-destructive.
                  </div>
                </div>
              </div>

              <div class="form-group">
                <label class="label">Archive Path</label>
                <div class="flex gap-2">
                  <input
                    class="input flex-1"
                    placeholder="/path/to/archive.7z"
                    value={testArchivePath()}
                    onInput={(e) => setTestArchivePath(e.currentTarget.value)}
                  />
                  <button class="btn-sm" onClick={handleSelectTestArchive}>
                    <HiOutlineFolderOpen class="w-4 h-4" />
                  </button>
                </div>
              </div>

              <div class="form-group">
                <label class="label">Password (optional)</label>
                <input
                  type="password"
                  class="input"
                  placeholder="Enter password if encrypted"
                  value={testPassword()}
                  onInput={(e) => setTestPassword(e.currentTarget.value)}
                />
              </div>

              <button
                class="btn-sm-primary"
                onClick={handleTestArchive}
                disabled={!testArchivePath() || testInProgress()}
              >
                <HiOutlinePlay class="w-4 h-4" />
                {testInProgress() ? "Testing..." : "Test Archive"}
              </button>

              <Show when={testResult() !== null}>
                <div
                  class={`card ${
                    testResult() ? "bg-success/10 border-success" : "bg-error/10 border-error"
                  }`}
                >
                  <div class="flex items-start gap-3">
                    {testResult() ? (
                      <HiOutlineCheckCircle class="w-6 h-6 text-success flex-shrink-0 mt-0.5" />
                    ) : (
                      <HiOutlineXCircle class="w-6 h-6 text-error flex-shrink-0 mt-0.5" />
                    )}
                    <div>
                      <div class="font-semibold text-txt">
                        {testResult() ? "Test Passed" : "Test Failed"}
                      </div>
                      <div class="text-sm text-txt-secondary mt-1">
                        {testResult()
                          ? "Archive integrity verified successfully"
                          : "Archive integrity check failed - try Repair or Validate"}
                      </div>
                    </div>
                  </div>
                </div>
              </Show>

              <Show when={lastError()}>
                <div class="card bg-warning/10 border-warning">
                  <div class="font-medium text-warning mb-2">Detailed Error Information</div>
                  <div class="text-sm space-y-1">
                    <div><strong>Code:</strong> {lastError()?.code}</div>
                    <div><strong>Message:</strong> {lastError()?.message}</div>
                    <div><strong>Context:</strong> {lastError()?.fileContext}</div>
                    <div><strong>Position:</strong> {lastError()?.position}</div>
                    <div class="pt-2 border-t border-warning/20">
                      <strong>Suggestion:</strong> {lastError()?.suggestion}
                    </div>
                  </div>
                </div>
              </Show>
            </div>
          </Show>

          {/* REPAIR TAB */}
          <Show when={activeTab() === "repair"}>
            <div class="col gap-4">
              <div class="info-card">
                <HiOutlineWrench class="w-5 h-5 text-warning" />
                <div>
                  <div class="font-medium text-txt">Repair Corrupted Archive</div>
                  <div class="text-sm text-txt-secondary">
                    Attempt to recover data from damaged or incomplete archives.
                  </div>
                </div>
              </div>

              <div class="form-group">
                <label class="label">Corrupted Archive</label>
                <div class="flex gap-2">
                  <input
                    class="input flex-1"
                    placeholder="/path/to/corrupted.7z"
                    value={repairCorruptedPath()}
                    onInput={(e) => setRepairCorruptedPath(e.currentTarget.value)}
                  />
                  <button class="btn-sm" onClick={handleSelectRepairInput}>
                    <HiOutlineFolderOpen class="w-4 h-4" />
                  </button>
                </div>
              </div>

              <div class="form-group">
                <label class="label">Output Path</label>
                <div class="flex gap-2">
                  <input
                    class="input flex-1"
                    placeholder="/path/to/repaired.7z"
                    value={repairOutputPath()}
                    onInput={(e) => setRepairOutputPath(e.currentTarget.value)}
                  />
                  <button class="btn-sm" onClick={handleSelectRepairOutput}>
                    <HiOutlineDocumentText class="w-4 h-4" />
                  </button>
                </div>
              </div>

              <button
                class="btn-sm-primary"
                onClick={handleRepairArchive}
                disabled={!repairCorruptedPath() || !repairOutputPath() || repairInProgress()}
              >
                <HiOutlinePlay class="w-4 h-4" />
                {repairInProgress() ? "Repairing..." : "Repair Archive"}
              </button>

              <Show when={repairInProgress()}>
                <div class="card">
                  <div class="text-sm text-txt-secondary mb-2">{repairStatus()}</div>
                  <div class="w-full bg-bg-secondary rounded-full h-2">
                    <div
                      class="bg-accent h-2 rounded-full transition-all"
                      style={{ width: `${repairProgress()}%` }}
                    />
                  </div>
                  <div class="text-sm text-txt-muted mt-1 text-center">
                    {repairProgress().toFixed(1)}%
                  </div>
                </div>
              </Show>

              <Show when={repairResult()}>
                <div class="card bg-success/10 border-success">
                  <div class="flex items-start gap-3">
                    <HiOutlineCheckCircle class="w-6 h-6 text-success flex-shrink-0 mt-0.5" />
                    <div>
                      <div class="font-semibold text-txt">Repair Complete</div>
                      <div class="text-sm text-txt-secondary mt-1">
                        Repaired archive saved to: {repairResult()}
                      </div>
                    </div>
                  </div>
                </div>
              </Show>
            </div>
          </Show>

          {/* VALIDATE TAB */}
          <Show when={activeTab() === "validate"}>
            <div class="col gap-4">
              <div class="info-card">
                <HiOutlineDocumentMagnifyingGlass class="w-5 h-5 text-info" />
                <div>
                  <div class="font-medium text-txt">Validate Archive</div>
                  <div class="text-sm text-txt-secondary">
                    Perform thorough validation with detailed error context and suggestions.
                  </div>
                </div>
              </div>

              <div class="form-group">
                <label class="label">Archive Path</label>
                <div class="flex gap-2">
                  <input
                    class="input flex-1"
                    placeholder="/path/to/archive.7z"
                    value={validateArchivePath()}
                    onInput={(e) => setValidateArchivePath(e.currentTarget.value)}
                  />
                  <button class="btn-sm" onClick={handleSelectValidateArchive}>
                    <HiOutlineFolderOpen class="w-4 h-4" />
                  </button>
                </div>
              </div>

              <button
                class="btn-sm-primary"
                onClick={handleValidateArchive}
                disabled={!validateArchivePath() || validateInProgress()}
              >
                <HiOutlinePlay class="w-4 h-4" />
                {validateInProgress() ? "Validating..." : "Validate Archive"}
              </button>

              <Show when={validateResult()}>
                <div
                  class={`card ${
                    validateResult()!.isValid
                      ? "bg-success/10 border-success"
                      : "bg-warning/10 border-warning"
                  }`}
                >
                  <div class="flex items-start gap-3">
                    {validateResult()!.isValid ? (
                      <HiOutlineCheckCircle class="w-6 h-6 text-success flex-shrink-0 mt-0.5" />
                    ) : (
                      <HiOutlineExclamationTriangle class="w-6 h-6 text-warning flex-shrink-0 mt-0.5" />
                    )}
                    <div class="flex-1">
                      <div class="font-semibold text-txt">
                        {validateResult()!.isValid ? "Validation Passed" : "Validation Issues"}
                      </div>
                      <Show when={validateResult()!.errorMessage}>
                        <div class="text-sm text-txt-secondary mt-1">
                          {validateResult()!.errorMessage}
                        </div>
                      </Show>
                      <Show when={validateResult()!.fileContext}>
                        <div class="text-sm text-txt-muted mt-1">
                          Context: {validateResult()!.fileContext}
                        </div>
                      </Show>
                      <Show when={validateResult()!.suggestion}>
                        <div class="text-sm text-accent mt-2 pt-2 border-t border-warning/20">
                          💡 {validateResult()!.suggestion}
                        </div>
                      </Show>
                    </div>
                  </div>
                </div>
              </Show>

              <Show when={lastError()}>
                <div class="card bg-error/10 border-error">
                  <div class="font-medium text-error mb-2">Detailed Error Information</div>
                  <div class="text-sm space-y-1">
                    <div><strong>Code:</strong> {lastError()?.code}</div>
                    <div><strong>Message:</strong> {lastError()?.message}</div>
                    <div><strong>Context:</strong> {lastError()?.fileContext}</div>
                    <div><strong>Position:</strong> {lastError()?.position}</div>
                    <div class="pt-2 border-t border-error/20">
                      <strong>Suggestion:</strong> {lastError()?.suggestion}
                    </div>
                  </div>
                </div>
              </Show>
            </div>
          </Show>

          {/* EXTRACT SPLIT TAB */}
          <Show when={activeTab() === "extract"}>
            <div class="col gap-4">
              <div class="info-card">
                <HiOutlineArchiveBox class="w-5 h-5 text-accent" />
                <div>
                  <div class="font-medium text-txt">Extract Split Archive</div>
                  <div class="text-sm text-txt-secondary">
                    Extract multi-volume archives (.001, .002, etc.) with automatic reassembly.
                  </div>
                </div>
              </div>

              <div class="form-group">
                <label class="label">First Volume (.001)</label>
                <div class="flex gap-2">
                  <input
                    class="input flex-1"
                    placeholder="/path/to/archive.7z.001"
                    value={extractFirstVolume()}
                    onInput={(e) => setExtractFirstVolume(e.currentTarget.value)}
                  />
                  <button class="btn-sm" onClick={handleSelectFirstVolume}>
                    <HiOutlineFolderOpen class="w-4 h-4" />
                  </button>
                </div>
              </div>

              <div class="form-group">
                <label class="label">Output Directory</label>
                <div class="flex gap-2">
                  <input
                    class="input flex-1"
                    placeholder="/path/to/output/"
                    value={extractOutputDir()}
                    onInput={(e) => setExtractOutputDir(e.currentTarget.value)}
                  />
                  <button class="btn-sm" onClick={handleSelectExtractOutput}>
                    <HiOutlineFolderOpen class="w-4 h-4" />
                  </button>
                </div>
              </div>

              <div class="form-group">
                <label class="label">Password (optional)</label>
                <input
                  type="password"
                  class="input"
                  placeholder="Enter password if encrypted"
                  value={extractPassword()}
                  onInput={(e) => setExtractPassword(e.currentTarget.value)}
                />
              </div>

              <button
                class="btn-sm-primary"
                onClick={handleExtractSplit}
                disabled={!extractFirstVolume() || !extractOutputDir() || extractInProgress()}
              >
                <HiOutlinePlay class="w-4 h-4" />
                {extractInProgress() ? "Extracting..." : "Extract Archive"}
              </button>

              <Show when={extractInProgress()}>
                <div class="card">
                  <div class="text-sm text-txt-secondary mb-2">{extractStatus()}</div>
                  <div class="w-full bg-bg-secondary rounded-full h-2">
                    <div
                      class="bg-accent h-2 rounded-full transition-all"
                      style={{ width: `${extractProgress()}%` }}
                    />
                  </div>
                  <div class="text-sm text-txt-muted mt-1 text-center">
                    {extractProgress().toFixed(1)}%
                  </div>
                </div>
              </Show>

              <Show when={extractResult()}>
                <div class="card bg-success/10 border-success">
                  <div class="flex items-start gap-3">
                    <HiOutlineCheckCircle class="w-6 h-6 text-success flex-shrink-0 mt-0.5" />
                    <div>
                      <div class="font-semibold text-txt">Extraction Complete</div>
                      <div class="text-sm text-txt-secondary mt-1">
                        Files extracted to: {extractResult()}
                      </div>
                    </div>
                  </div>
                </div>
              </Show>
            </div>
          </Show>
        </div>

        {/* Footer */}
        <div class="modal-footer justify-end">
          <button class="btn-sm" onClick={props.onClose}>
            Close
          </button>
        </div>
      </div>
    </div>
  );
}
