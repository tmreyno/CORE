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

import { createSignal, Show } from "solid-js";
import { open, save } from "@tauri-apps/plugin-dialog";
import {
  HiOutlineCheckCircle,
  HiOutlineXCircle,
  HiOutlineWrench,
  HiOutlineDocumentMagnifyingGlass,
  HiOutlineArchiveBox,
} from "./icons";
import {
  testArchive,
  repairArchive,
  validateArchive,
  extractSplitArchive,
  listenToRepairProgress,
  listenToSplitExtractProgress,
  getLastArchiveError,
  type ArchiveValidationResult,
  type DetailedArchiveError,
  type ArchiveRepairProgress,
  type SplitExtractProgress,
} from "../api/archiveCreate";
import { useToast } from "./Toast";
import { getErrorMessage } from "../utils/errorUtils";
import { TestTab } from "./archive-tools/TestTab";
import { RepairTab } from "./archive-tools/RepairTab";
import { ValidateTab } from "./archive-tools/ValidateTab";
import { ExtractTab } from "./archive-tools/ExtractTab";

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
    } catch (error: unknown) {
      toast.error("Test Error", getErrorMessage(error));
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
    } catch (error: unknown) {
      toast.error("Repair Failed", getErrorMessage(error));
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
    } catch (error: unknown) {
      toast.error("Validation Error", getErrorMessage(error));
      setValidateResult({
        isValid: false,
        errorMessage: getErrorMessage(error),
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
    } catch (error: unknown) {
      toast.error("Extraction Failed", getErrorMessage(error));
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
            <TestTab
              testArchivePath={testArchivePath}
              setTestArchivePath={setTestArchivePath}
              testPassword={testPassword}
              setTestPassword={setTestPassword}
              testResult={testResult}
              testInProgress={testInProgress}
              lastError={lastError}
              onTest={handleTestArchive}
              onSelectArchive={handleSelectTestArchive}
            />
          </Show>

          {/* REPAIR TAB */}
          <Show when={activeTab() === "repair"}>
            <RepairTab
              repairCorruptedPath={repairCorruptedPath}
              setRepairCorruptedPath={setRepairCorruptedPath}
              repairOutputPath={repairOutputPath}
              setRepairOutputPath={setRepairOutputPath}
              repairProgress={repairProgress}
              repairStatus={repairStatus}
              repairInProgress={repairInProgress}
              repairResult={repairResult}
              onRepair={handleRepairArchive}
              onSelectInput={handleSelectRepairInput}
              onSelectOutput={handleSelectRepairOutput}
            />
          </Show>

          {/* VALIDATE TAB */}
          <Show when={activeTab() === "validate"}>
            <ValidateTab
              validateArchivePath={validateArchivePath}
              setValidateArchivePath={setValidateArchivePath}
              validateResult={validateResult}
              validateInProgress={validateInProgress}
              lastError={lastError}
              onValidate={handleValidateArchive}
              onSelectArchive={handleSelectValidateArchive}
            />
          </Show>

          {/* EXTRACT SPLIT TAB */}
          <Show when={activeTab() === "extract"}>
            <ExtractTab
              extractFirstVolume={extractFirstVolume}
              setExtractFirstVolume={setExtractFirstVolume}
              extractOutputDir={extractOutputDir}
              setExtractOutputDir={setExtractOutputDir}
              extractPassword={extractPassword}
              setExtractPassword={setExtractPassword}
              extractProgress={extractProgress}
              extractStatus={extractStatus}
              extractInProgress={extractInProgress}
              extractResult={extractResult}
              onExtract={handleExtractSplit}
              onSelectFirstVolume={handleSelectFirstVolume}
              onSelectOutput={handleSelectExtractOutput}
            />
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
