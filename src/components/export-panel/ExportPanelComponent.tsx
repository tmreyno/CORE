// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show, createEffect, untrack } from "solid-js";
import { useToast } from "../Toast";
import { useExportState } from "../../hooks/useExportState";
import { PhysicalImageMode } from "../export/PhysicalImageMode";
import { LogicalImageMode } from "../export/LogicalImageMode";
import { NativeExportMode } from "../export/NativeExportMode";
import { ToolsMode } from "../export/ToolsMode";
import DriveSelector from "../export/DriveSelector";
import { ExportHeader } from "./ExportHeader";
import { ExportSourceSection } from "./ExportSourceSection";
import { ExportFooter } from "./ExportFooter";
import type { ExportPanelProps } from "./types";

export function ExportPanelComponent(props: ExportPanelProps) {
  const toast = useToast();

  const state = useExportState({
    initialSources: props.initialSources,
    initialExaminerName: props.initialExaminerName,
    initialMode: props.initialMode,
    onComplete: props.onComplete,
    onActivityCreate: props.onActivityCreate,
    onActivityUpdate: props.onActivityUpdate,
    toast,
  });

  // Watch for pending drive sources from the left-panel drives browser
  createEffect(() => {
    const pending = props.pendingDriveSources?.() ?? [];
    const mode = props.pendingExportMode?.();
    const dest = props.pendingDestination?.();

    // Nothing pending — skip
    if (pending.length === 0 && !mode && !dest) return;

    // Use untrack to prevent sources() inside addUniqueSources from becoming
    // a tracked dependency of this effect (only the pending signals trigger it)
    const count = pending.length;
    untrack(() => {
      if (mode) state.setMode(mode);
      if (dest) state.setDestination(dest);
      for (const path of pending) {
        state.handleAddDriveSource(path);
      }
    });

    if (count > 1) {
      toast.success("Sources Added", `${count} items added to export`);
    }

    props.onPendingSourcesConsumed?.();
  });

  // Watch for pending removals from the drive panel (bidirectional sync)
  createEffect(() => {
    const removals = props.pendingRemoveSources?.() ?? [];
    if (removals.length === 0) return;

    untrack(() => {
      for (const path of removals) {
        state.removeSourceByPath(path);
      }
    });

    props.onPendingRemoveConsumed?.();
  });

  return (
    <>
      <div class="flex flex-col h-full bg-bg">
        <ExportHeader
          mode={state.mode}
          setMode={state.setMode}
          onReset={state.handleReset}
          onClose={props.onClose}
        />

        {/* Content */}
        <div class="flex-1 overflow-y-auto p-3 space-y-3">
          {/* Source/Destination for non-tools modes */}
          <Show when={state.mode() !== "tools"}>
            <ExportSourceSection
              mode={state.mode}
              sources={state.sources}
              destination={state.destination}
              driveSources={state.driveSources}
              mountDrivesReadOnly={state.mountDrivesReadOnly}
              onRemoveSource={state.handleRemoveSource}
              onSelectDestination={state.handleSelectDestination}
            />

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

          {/* Archive Tools UI */}
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

        <ExportFooter
          mode={state.mode}
          sources={state.sources}
          destination={state.destination}
          isProcessing={state.isProcessing}
          nativeExportTab={state.nativeExportTab}
          toolsTab={state.toolsTab}
          testArchivePath={state.testArchivePath}
          repairCorruptedPath={state.repairCorruptedPath}
          repairOutputPath={state.repairOutputPath}
          validateArchivePath={state.validateArchivePath}
          extractFirstVolume={state.extractFirstVolume}
          extractOutputDir={state.extractOutputDir}
          lzmaInputPath={state.lzmaInputPath}
          lzmaOutputPath={state.lzmaOutputPath}
          lzmaDecompressInput={state.lzmaDecompressInput}
          lzmaDecompressOutput={state.lzmaDecompressOutput}
          activeExportOperationId={state.activeExportOperationId}
          onStart={state.handleStart}
          onToolAction={state.handleToolAction}
          onCancelExport={state.handleCancelExport}
        />
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
