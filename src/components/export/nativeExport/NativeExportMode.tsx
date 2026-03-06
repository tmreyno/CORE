// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

// NativeExportMode - Combined file export + 7z archive creation with sub-tabs

import { Component, Show } from "solid-js";
import { HiOutlineArchiveBox, HiOutlineArrowUpTray } from "../../icons";
import { FileExportTab } from "./FileExportTab";
import { ArchiveTab } from "./ArchiveTab";
import type { NativeExportModeProps } from "./types";

export const NativeExportMode: Component<NativeExportModeProps> = (props) => {
  const isFiles = () => props.activeTab() === "files";
  const isArchive = () => props.activeTab() === "archive";

  return (
    <div class="space-y-4">
      {/* Sub-tab Toggle */}
      <div class="grid grid-cols-2 gap-2">
        <button
          class={`flex items-center justify-center gap-2 p-2.5 rounded-lg border transition-colors ${
            isFiles()
              ? "border-accent bg-accent/10 text-accent"
              : "border-border bg-bg-secondary text-txt-secondary hover:border-border-hover"
          }`}
          onClick={() => props.setActiveTab("files")}
        >
          <HiOutlineArrowUpTray class="w-4 h-4" />
          <span class="text-sm font-medium">File Export</span>
        </button>
        <button
          class={`flex items-center justify-center gap-2 p-2.5 rounded-lg border transition-colors ${
            isArchive()
              ? "border-accent bg-accent/10 text-accent"
              : "border-border bg-bg-secondary text-txt-secondary hover:border-border-hover"
          }`}
          onClick={() => props.setActiveTab("archive")}
        >
          <HiOutlineArchiveBox class="w-4 h-4" />
          <span class="text-sm font-medium">7z Archive</span>
        </button>
      </div>

      {/* File Export Tab */}
      <Show when={isFiles()}>
        <FileExportTab
          exportName={props.exportName}
          setExportName={props.setExportName}
          computeHashes={props.computeHashes}
          setComputeHashes={props.setComputeHashes}
          verifyAfterCopy={props.verifyAfterCopy}
          setVerifyAfterCopy={props.setVerifyAfterCopy}
          generateJsonManifest={props.generateJsonManifest}
          setGenerateJsonManifest={props.setGenerateJsonManifest}
          generateTxtReport={props.generateTxtReport}
          setGenerateTxtReport={props.setGenerateTxtReport}
        />
      </Show>

      {/* 7z Archive Tab */}
      <Show when={isArchive()}>
        <ArchiveTab
          archiveName={props.archiveName}
          setArchiveName={props.setArchiveName}
          compressionLevel={props.compressionLevel}
          setCompressionLevel={props.setCompressionLevel}
          estimatedUncompressed={props.estimatedUncompressed}
          estimatedCompressed={props.estimatedCompressed}
          password={props.password}
          setPassword={props.setPassword}
          showPassword={props.showPassword}
          setShowPassword={props.setShowPassword}
          showAdvanced={props.showAdvanced}
          setShowAdvanced={props.setShowAdvanced}
          solid={props.solid}
          setSolid={props.setSolid}
          numThreads={props.numThreads}
          setNumThreads={props.setNumThreads}
          splitSizeMb={props.splitSizeMb}
          setSplitSizeMb={props.setSplitSizeMb}
          generateManifest={props.generateManifest}
          setGenerateManifest={props.setGenerateManifest}
          verifyAfterCreate={props.verifyAfterCreate}
          setVerifyAfterCreate={props.setVerifyAfterCreate}
          hashAlgorithm={props.hashAlgorithm}
          setHashAlgorithm={props.setHashAlgorithm}
          includeExaminerInfo={props.includeExaminerInfo}
          setIncludeExaminerInfo={props.setIncludeExaminerInfo}
          examinerName={props.examinerName}
          setExaminerName={props.setExaminerName}
          caseNumber={props.caseNumber}
          setCaseNumber={props.setCaseNumber}
          evidenceDescription={props.evidenceDescription}
          setEvidenceDescription={props.setEvidenceDescription}
        />
      </Show>
    </div>
  );
};
