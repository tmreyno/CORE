// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ExportPanel - Forensic export panel (render-only)
 *
 * All state and handlers are managed by useExportState hook.
 * This component is responsible only for rendering the UI.
 *
 * Provides four export modes organized by output type:
 * 1. Physical Image - E01 disk image creation (raw byte stream containers)
 * 2. Logical Image - L01 logical evidence creation (file-based containers)
 * 3. Native Export - File export (copy with hashes) or 7z archive creation
 * 4. Tools - Archive test, repair, validate, extract, LZMA compress/decompress
 */

import { Show, For } from "solid-js";
import {
  HiOutlineFolderOpen,
  HiOutlineCircleStack,
  HiOutlineDocumentDuplicate,
  HiOutlineArrowUpTray,
  HiOutlinePlay,
  HiOutlineXMark,
  HiOutlineWrench,
  HiOutlineServer,
  HiOutlineLockClosed,
} from "./icons";
import { useToast } from "./Toast";
import type { Activity } from "../types/activity";
import { useExportState } from "../hooks/useExportState";
import { PhysicalImageMode } from "./export/PhysicalImageMode";
import { LogicalImageMode } from "./export/LogicalImageMode";
import { NativeExportMode } from "./export/NativeExportMode";
import { ToolsMode } from "./export/ToolsMode";
import DriveSelector from "./export/DriveSelector";
import { getBasename } from "../utils/pathUtils";

/** Re-export ExportMode for existing consumers */
export type { ExportMode } from "../hooks/useExportState";

/** Export panel props */
export interface ExportPanelProps {
  /** Pre-selected source files (optional) */
  initialSources?: string[];
  /** Callback when export completes */
  onComplete?: (destination: string) => void;
  /** Callback when panel is closed */
  onClose?: () => void;
  /** Callback when an activity is created */
  onActivityCreate?: (activity: Activity) => void;
  /** Callback when an activity is updated */
  onActivityUpdate?: (id: string, updates: Partial<Activity>) => void;
}

export function ExportPanel(props: ExportPanelProps) {
  const toast = useToast();

  const state = useExportState({
    initialSources: props.initialSources,
    onComplete: props.onComplete,
    onActivityCreate: props.onActivityCreate,
    onActivityUpdate: props.onActivityUpdate,
    toast,
  });

  return (
    <>
      <div class="flex flex-col h-full bg-bg">
        {/* Header */}
        <div class="panel-header">
          <h2 class="text-lg font-semibold text-txt">Export & Archive</h2>
          <div class="flex items-center gap-2">
            <Show when={props.onClose}>
              <button
                class="icon-btn-sm"
                onClick={props.onClose}
                title="Close"
              >
                <HiOutlineXMark class="w-4 h-4" />
              </button>
            </Show>
          </div>
        </div>

        {/* Mode Selector */}
        <div class="p-4 border-b border-border">
          <div class="flex gap-2 items-center justify-between">
            <div class="flex gap-2">
              <button
                class={state.mode() === "physical" ? "btn-sm-primary" : "btn-sm"}
                onClick={() => state.setMode("physical")}
                title="Create E01 disk image (physical/raw byte stream)"
              >
                <HiOutlineCircleStack class="w-4 h-4" />
                Physical
              </button>

              <button
                class={state.mode() === "logical" ? "btn-sm-primary" : "btn-sm"}
                onClick={() => state.setMode("logical")}
                title="Create L01 logical evidence container (file-based)"
              >
                <HiOutlineDocumentDuplicate class="w-4 h-4" />
                Logical
              </button>

              <button
                class={state.mode() === "native" ? "btn-sm-primary" : "btn-sm"}
                onClick={() => state.setMode("native")}
                title="Export files or create 7z archive"
              >
                <HiOutlineArrowUpTray class="w-4 h-4" />
                Native
              </button>

              <button
                class={state.mode() === "tools" ? "btn-sm-primary" : "btn-sm"}
                onClick={() => state.setMode("tools")}
                title="Archive Tools (Test, Repair, Validate, Extract)"
              >
                <HiOutlineWrench class="w-4 h-4" />
                Tools
              </button>
            </div>

            {/* Clear Form Button */}
            <button
              class="btn-sm"
              onClick={state.handleReset}
              title="Clear all form fields"
            >
              <HiOutlineXMark class="w-4 h-4" />
              Clear
            </button>
          </div>

          {/* Mode Description */}
          <div class="mt-2 text-xs text-txt-secondary">
            <Show when={state.mode() === "physical"}>
              Wrap raw images and evidence files into E01 containers with case metadata and hashes
            </Show>
            <Show when={state.mode() === "logical"}>
              Package files and folders into L01 logical evidence containers with per-file hashes
            </Show>
            <Show when={state.mode() === "native"}>
              Copy files with forensic manifests, or create compressed 7z archives
            </Show>
            <Show when={state.mode() === "tools"}>
              Test, repair, validate, or extract split archives
            </Show>
          </div>
        </div>

        {/* Content */}
        <div class="flex-1 overflow-y-auto p-4 space-y-4">
          {/* Show source/destination for all modes except tools */}
          <Show when={state.mode() !== "tools"}>
            {/* Source Files */}
            <div class="space-y-2">
              <label class="label">
                {state.mode() === "physical" || state.mode() === "logical" ? "Source" : "Source Files"}
              </label>
              <div class="flex gap-2 flex-wrap">
                <button class="btn-sm" onClick={state.handleAddSources}>
                  <HiOutlineFolderOpen class="w-4 h-4" />
                  Add Files
                </button>
                <button class="btn-sm" onClick={state.handleAddFolder}>
                  <HiOutlineFolderOpen class="w-4 h-4" />
                  Add Folder
                </button>
                <Show when={state.mode() === "physical" || state.mode() === "logical"}>
                  <button class="btn-sm" onClick={() => state.setShowDriveSelector(true)}>
                    <HiOutlineServer class="w-4 h-4" />
                    Select Drive
                  </button>
                </Show>
              </div>
              <Show when={state.mode() === "physical"}>
                <p class="text-xs text-txt-muted">
                  Select raw disk images (.dd, .raw, .img), memory dumps (.mem),
                  folders, drives/volumes, or other evidence files to wrap in an E01 container.
                </p>
              </Show>
              <Show when={state.mode() === "logical"}>
                <p class="text-xs text-txt-muted">
                  Select files, folders, or drives/volumes to package into an L01 logical evidence container.
                </p>
              </Show>

              {/* Source List */}
              <Show when={state.sources().length > 0}>
                <div class="space-y-1 mt-2">
                  <For each={state.sources()}>
                    {(source, index) => {
                      const isDrive = () => state.driveSources().has(source);
                      return (
                        <div class="flex items-center gap-2 p-2 bg-bg-secondary rounded text-sm">
                          <Show when={isDrive()}>
                            <HiOutlineServer class="w-4 h-4 text-accent shrink-0" />
                          </Show>
                          <span class="flex-1 truncate text-txt" title={source}>
                            {isDrive() ? source : (getBasename(source) || source)}
                          </span>
                          <Show when={isDrive()}>
                            <span class="badge badge-warning text-[10px] shrink-0">Drive</span>
                            <Show when={state.mountDrivesReadOnly()}>
                              <span
                                class="badge badge-success text-[10px] shrink-0 flex items-center gap-0.5"
                                title="Will be remounted read-only before imaging"
                              >
                                <HiOutlineLockClosed class="w-2.5 h-2.5" />
                                RO
                              </span>
                            </Show>
                          </Show>
                          <button
                            class="icon-btn-sm"
                            onClick={() => state.handleRemoveSource(index())}
                            title="Remove"
                          >
                            <HiOutlineXMark class="w-3 h-3" />
                          </button>
                        </div>
                      );
                    }}
                  </For>
                </div>
              </Show>

              <Show when={state.sources().length === 0}>
                <div class="text-sm text-txt-muted italic">No files selected</div>
              </Show>
            </div>

            {/* Destination */}
            <div class="space-y-2">
              <label class="label">Destination</label>
              <div class="flex gap-2">
                <input
                  class="input flex-1"
                  type="text"
                  value={state.destination()}
                  placeholder="Select destination folder..."
                  readOnly
                />
                <button class="btn-sm" onClick={state.handleSelectDestination}>
                  <HiOutlineFolderOpen class="w-4 h-4" />
                  Browse
                </button>
              </div>
            </div>

            {/* Physical Image Mode (E01) */}
            <Show when={state.mode() === "physical"}>
              <PhysicalImageMode
                imageName={state.ewfImageName}
                setImageName={state.setEwfImageName}
                format={state.ewfFormat}
                setFormat={state.setEwfFormat}
                compression={state.ewfCompression}
                setCompression={state.setEwfCompression}
                compressionMethod={state.ewfCompressionMethod}
                setCompressionMethod={state.setEwfCompressionMethod}
                computeMd5={state.ewfComputeMd5}
                setComputeMd5={state.setEwfComputeMd5}
                computeSha1={state.ewfComputeSha1}
                setComputeSha1={state.setEwfComputeSha1}
                caseNumber={state.ewfCaseNumber}
                setCaseNumber={state.setEwfCaseNumber}
                evidenceNumber={state.ewfEvidenceNumber}
                setEvidenceNumber={state.setEwfEvidenceNumber}
                examinerName={state.ewfExaminerName}
                setExaminerName={state.setEwfExaminerName}
                description={state.ewfDescription}
                setDescription={state.setEwfDescription}
                notes={state.ewfNotes}
                setNotes={state.setEwfNotes}
                segmentSize={state.ewfSegmentSize}
                setSegmentSize={state.setEwfSegmentSize}
                showAdvanced={state.showAdvanced}
                setShowAdvanced={state.setShowAdvanced}
              />
            </Show>

            {/* Logical Image Mode (L01) */}
            <Show when={state.mode() === "logical"}>
              <LogicalImageMode
                imageName={state.l01ImageName}
                setImageName={state.setL01ImageName}
                compression={state.l01Compression}
                setCompression={state.setL01Compression}
                caseNumber={state.l01CaseNumber}
                setCaseNumber={state.setL01CaseNumber}
                evidenceNumber={state.l01EvidenceNumber}
                setEvidenceNumber={state.setL01EvidenceNumber}
                examinerName={state.l01ExaminerName}
                setExaminerName={state.setL01ExaminerName}
                description={state.l01Description}
                setDescription={state.setL01Description}
                notes={state.l01Notes}
                setNotes={state.setL01Notes}
                segmentSize={state.l01SegmentSize}
                setSegmentSize={state.setL01SegmentSize}
                showAdvanced={state.showAdvanced}
                setShowAdvanced={state.setShowAdvanced}
              />
            </Show>

            {/* Native Export Mode (Files + 7z Archive) */}
            <Show when={state.mode() === "native"}>
              <NativeExportMode
                activeTab={state.nativeExportTab}
                setActiveTab={state.setNativeExportTab}
                exportName={state.exportName}
                setExportName={state.setExportName}
                computeHashes={state.computeHashes}
                setComputeHashes={state.setComputeHashes}
                verifyAfterCopy={state.verifyAfterCopy}
                setVerifyAfterCopy={state.setVerifyAfterCopy}
                generateJsonManifest={state.generateJsonManifest}
                setGenerateJsonManifest={state.setGenerateJsonManifest}
                generateTxtReport={state.generateTxtReport}
                setGenerateTxtReport={state.setGenerateTxtReport}
                archiveName={state.archiveName}
                setArchiveName={state.setArchiveName}
                compressionLevel={state.compressionLevel}
                setCompressionLevel={state.setCompressionLevel}
                estimatedUncompressed={state.estimatedUncompressed}
                estimatedCompressed={state.estimatedCompressed}
                password={state.password}
                setPassword={state.setPassword}
                showPassword={state.showPassword}
                setShowPassword={state.setShowPassword}
                showAdvanced={state.showAdvanced}
                setShowAdvanced={state.setShowAdvanced}
                solid={state.solid}
                setSolid={state.setSolid}
                numThreads={state.numThreads}
                setNumThreads={state.setNumThreads}
                splitSizeMb={state.splitSizeMb}
                setSplitSizeMb={state.setSplitSizeMb}
                generateManifest={state.generateManifest}
                setGenerateManifest={state.setGenerateManifest}
                verifyAfterCreate={state.verifyAfterCreate}
                setVerifyAfterCreate={state.setVerifyAfterCreate}
                hashAlgorithm={state.hashAlgorithm}
                setHashAlgorithm={state.setHashAlgorithm}
                includeExaminerInfo={state.includeExaminerInfo}
                setIncludeExaminerInfo={state.setIncludeExaminerInfo}
                examinerName={state.examinerName}
                setExaminerName={state.setExaminerName}
                caseNumber={state.caseNumber}
                setCaseNumber={state.setCaseNumber}
                evidenceDescription={state.evidenceDescription}
                setEvidenceDescription={state.setEvidenceDescription}
              />
            </Show>
          </Show>

          {/* Show Archive Tools UI */}
          <Show when={state.mode() === "tools"}>
            <ToolsMode
              toolsTab={state.toolsTab}
              setToolsTab={state.setToolsTab}
              testArchivePath={state.testArchivePath}
              setTestArchivePath={state.setTestArchivePath}
              repairCorruptedPath={state.repairCorruptedPath}
              setRepairCorruptedPath={state.setRepairCorruptedPath}
              repairOutputPath={state.repairOutputPath}
              setRepairOutputPath={state.setRepairOutputPath}
              validateArchivePath={state.validateArchivePath}
              setValidateArchivePath={state.setValidateArchivePath}
              extractFirstVolume={state.extractFirstVolume}
              setExtractFirstVolume={state.setExtractFirstVolume}
              extractOutputDir={state.extractOutputDir}
              setExtractOutputDir={state.setExtractOutputDir}
              lzmaInputPath={state.lzmaInputPath}
              setLzmaInputPath={state.setLzmaInputPath}
              lzmaOutputPath={state.lzmaOutputPath}
              setLzmaOutputPath={state.setLzmaOutputPath}
              lzmaAlgorithm={state.lzmaAlgorithm}
              setLzmaAlgorithm={state.setLzmaAlgorithm}
              lzmaLevel={state.lzmaLevel}
              setLzmaLevel={state.setLzmaLevel}
              lzmaDecompressInput={state.lzmaDecompressInput}
              setLzmaDecompressInput={state.setLzmaDecompressInput}
              lzmaDecompressOutput={state.lzmaDecompressOutput}
              setLzmaDecompressOutput={state.setLzmaDecompressOutput}
            />
          </Show>
        </div>

        {/* Footer */}
        <div class="p-4 border-t border-border flex justify-between items-center">
          <Show when={state.mode() !== "tools"}>
            <div class="text-xs text-txt-muted">
              {state.sources().length} item{state.sources().length !== 1 ? "s" : ""} selected
            </div>

            <button
              class="btn-sm-primary"
              onClick={state.handleStart}
              disabled={state.isProcessing() || state.sources().length === 0 || !state.destination()}
            >
              <Show when={!state.isProcessing()} fallback={<span>Processing...</span>}>
                <HiOutlinePlay class="w-4 h-4" />
                Start{" "}
                {state.mode() === "physical"
                  ? "E01 Image"
                  : state.mode() === "logical"
                    ? "L01 Image"
                    : state.nativeExportTab() === "archive"
                      ? "Archive"
                      : "Export"}
              </Show>
            </button>
          </Show>

          <Show when={state.mode() === "tools"}>
            <div class="flex-1" />
            <button
              class="btn-sm-primary"
              onClick={state.handleToolAction}
              disabled={
                state.isProcessing() ||
                (state.toolsTab() === "test" && !state.testArchivePath()) ||
                (state.toolsTab() === "repair" && (!state.repairCorruptedPath() || !state.repairOutputPath())) ||
                (state.toolsTab() === "validate" && !state.validateArchivePath()) ||
                (state.toolsTab() === "extract" && (!state.extractFirstVolume() || !state.extractOutputDir())) ||
                (state.toolsTab() === "compress" && (!state.lzmaInputPath() || !state.lzmaOutputPath())) ||
                (state.toolsTab() === "decompress" &&
                  (!state.lzmaDecompressInput() || !state.lzmaDecompressOutput()))
              }
            >
              <Show when={!state.isProcessing()} fallback={<span>Processing...</span>}>
                <HiOutlinePlay class="w-4 h-4" />
                {state.toolsTab() === "test" && "Test Archive"}
                {state.toolsTab() === "repair" && "Repair Archive"}
                {state.toolsTab() === "validate" && "Validate Archive"}
                {state.toolsTab() === "extract" && "Extract Archive"}
                {state.toolsTab() === "compress" && "Compress"}
                {state.toolsTab() === "decompress" && "Decompress"}
              </Show>
            </button>
          </Show>
        </div>
      </div>

      {/* Drive Selector Modal */}
      <DriveSelector
        isOpen={state.showDriveSelector()}
        onClose={() => state.setShowDriveSelector(false)}
        onSelect={state.handleDriveSelected}
      />
    </>
  );
}
