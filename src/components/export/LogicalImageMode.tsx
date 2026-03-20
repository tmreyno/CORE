// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

// LogicalImageMode - L01 logical evidence container creation UI

import { Component, Show, Accessor, Setter, createSignal } from "solid-js";
import {
  HiOutlineFingerPrint,
  HiOutlineChevronDown,
  HiOutlineChevronRight,
  HiOutlineCog6Tooth,
  HiOutlineInformationCircle,
  HiOutlineFunnel,
} from "../icons";
import { CaseMetadataSection } from "./CaseMetadataSection";
import { SplitSizeSelector } from "./SplitSizeSelector";

// --- Props ---

export interface LogicalImageModeProps {
  // Image name
  imageName: Accessor<string>;
  setImageName: Setter<string>;
  // Compression
  compression: Accessor<string>;
  setCompression: Setter<string>;
  // Case metadata
  caseNumber: Accessor<string>;
  setCaseNumber: Setter<string>;
  evidenceNumber: Accessor<string>;
  setEvidenceNumber: Setter<string>;
  examinerName: Accessor<string>;
  setExaminerName: Setter<string>;
  description: Accessor<string>;
  setDescription: Setter<string>;
  notes: Accessor<string>;
  setNotes: Setter<string>;
  // Advanced
  segmentSize: Accessor<number>;
  setSegmentSize: Setter<number>;
  showAdvanced: Accessor<boolean>;
  setShowAdvanced: Setter<boolean>;
  // File filters
  filterExtensions: Accessor<string>;
  setFilterExtensions: Setter<string>;
  excludeExtensions: Accessor<string>;
  setExcludeExtensions: Setter<string>;
  minFileSize: Accessor<number | undefined>;
  setMinFileSize: Setter<number | undefined>;
  maxFileSize: Accessor<number | undefined>;
  setMaxFileSize: Setter<number | undefined>;
}

// --- Component ---

export const LogicalImageMode: Component<LogicalImageModeProps> = (props) => {
  const [showCaseMetadata, setShowCaseMetadata] = createSignal(false);
  const [showFilters, setShowFilters] = createSignal(false);

  return (
    <div class="space-y-3">
      {/* Compact info callout */}
      <div class="flex items-start gap-2 bg-bg-secondary border border-border rounded-lg p-2.5">
        <HiOutlineInformationCircle class="w-4 h-4 text-info mt-0.5 flex-shrink-0" />
        <p class="text-xs text-txt-muted leading-relaxed">
          Creates an <span class="text-txt-secondary font-medium">L01 logical evidence container</span> that
          packages files and folders into a compressed, hash-verified container with forensic
          case metadata. Compatible with EnCase, FTK Imager, and AXIOM.
        </p>
      </div>

      {/* Image Name + Compression */}
      <div class="grid grid-cols-2 gap-2">
        <div class="space-y-1">
          <label class="label">Image Name</label>
          <div class="flex items-center gap-2">
            <input
              class="input-sm flex-1"
              type="text"
              value={props.imageName()}
              onInput={(e) => props.setImageName(e.currentTarget.value)}
              placeholder="evidence"
            />
            <span class="text-sm text-txt-muted font-mono">.L01</span>
          </div>
        </div>
        <div class="space-y-1">
          <label class="label text-xs">Compression</label>
          <select
            class="input-sm"
            value={props.compression()}
            onChange={(e) => props.setCompression(e.currentTarget.value)}
          >
            <option value="none">None (Store)</option>
            <option value="fast">Fast (zlib default)</option>
            <option value="best">Best (zlib maximum)</option>
          </select>
        </div>
      </div>

      {/* Embedded Hashes - always both MD5 + SHA-1 */}
      <div class="space-y-2">
        <label class="label flex items-center gap-1">
          <HiOutlineFingerPrint class="w-3.5 h-3.5" />
          Embedded Hashes
        </label>
        <div class="flex gap-4 pl-1">
          <label class="flex items-center gap-2 text-xs cursor-default">
            <input
              type="checkbox"
              checked={true}
              disabled
              class="accent-accent"
            />
            <span class="text-txt">MD5</span>
          </label>
          <label class="flex items-center gap-2 text-xs cursor-default">
            <input
              type="checkbox"
              checked={true}
              disabled
              class="accent-accent"
            />
            <span class="text-txt">SHA-1</span>
          </label>
        </div>
        <div class="text-xs text-txt-muted pl-1">
          L01 always embeds both MD5 and SHA-1 (per-file and image-level)
        </div>
      </div>

      {/* Case Metadata */}
      <CaseMetadataSection
        isOpen={showCaseMetadata}
        setIsOpen={setShowCaseMetadata}
        caseNumber={props.caseNumber}
        setCaseNumber={props.setCaseNumber}
        evidenceNumber={props.evidenceNumber}
        setEvidenceNumber={props.setEvidenceNumber}
        examinerName={props.examinerName}
        setExaminerName={props.setExaminerName}
        description={props.description}
        setDescription={props.setDescription}
        notes={props.notes}
        setNotes={props.setNotes}
      />

      {/* Advanced */}
      <div class="space-y-2">
        <button
          class="flex items-center gap-1 text-xs text-txt-secondary hover:text-txt"
          onClick={() => props.setShowAdvanced(!props.showAdvanced())}
        >
          <Show when={props.showAdvanced()} fallback={<HiOutlineChevronRight class="w-3.5 h-3.5" />}>
            <HiOutlineChevronDown class="w-3.5 h-3.5" />
          </Show>
          <HiOutlineCog6Tooth class="w-3.5 h-3.5" />
          Advanced
        </button>

        <Show when={props.showAdvanced()}>
          <div class="space-y-3 pl-5 pt-1">
            <SplitSizeSelector
              valueMb={props.segmentSize}
              setValueMb={props.setSegmentSize}
              label="Segment Size"
            />
          </div>
        </Show>
      </div>

      {/* File Filters */}
      <div class="space-y-2">
        <button
          class="flex items-center gap-1 text-xs text-txt-secondary hover:text-txt"
          onClick={() => setShowFilters(!showFilters())}
        >
          <Show when={showFilters()} fallback={<HiOutlineChevronRight class="w-3.5 h-3.5" />}>
            <HiOutlineChevronDown class="w-3.5 h-3.5" />
          </Show>
          <HiOutlineFunnel class="w-3.5 h-3.5" />
          File Filters
          <Show when={props.filterExtensions() || props.excludeExtensions() || props.minFileSize() != null || props.maxFileSize() != null}>
            <span class="ml-1 text-accent text-xs">(active)</span>
          </Show>
        </button>

        <Show when={showFilters()}>
          <div class="space-y-3 pl-5 pt-1">
            <div class="space-y-1">
              <label class="label text-xs">Include Only (extensions)</label>
              <input
                class="input-sm"
                type="text"
                value={props.filterExtensions()}
                onInput={(e) => props.setFilterExtensions(e.currentTarget.value)}
                placeholder="e.g. pdf, docx, xlsx"
              />
              <p class="text-xs text-txt-muted">Comma-separated. Leave empty to include all file types.</p>
            </div>
            <div class="space-y-1">
              <label class="label text-xs">Exclude (extensions)</label>
              <input
                class="input-sm"
                type="text"
                value={props.excludeExtensions()}
                onInput={(e) => props.setExcludeExtensions(e.currentTarget.value)}
                placeholder="e.g. tmp, log, cache"
              />
            </div>
            <div class="grid grid-cols-2 gap-2">
              <div class="space-y-1">
                <label class="label text-xs">Min File Size (bytes)</label>
                <input
                  class="input-sm"
                  type="number"
                  min="0"
                  value={props.minFileSize() ?? ""}
                  onInput={(e) => {
                    const v = e.currentTarget.value;
                    props.setMinFileSize(v ? parseInt(v, 10) : undefined);
                  }}
                  placeholder="0"
                />
              </div>
              <div class="space-y-1">
                <label class="label text-xs">Max File Size (bytes)</label>
                <input
                  class="input-sm"
                  type="number"
                  min="0"
                  value={props.maxFileSize() ?? ""}
                  onInput={(e) => {
                    const v = e.currentTarget.value;
                    props.setMaxFileSize(v ? parseInt(v, 10) : undefined);
                  }}
                  placeholder="No limit"
                />
              </div>
            </div>
          </div>
        </Show>
      </div>

      {/* Info Card */}
      <div class="callout">
        <HiOutlineInformationCircle class="w-4 h-4 text-info shrink-0 mt-0.5" />
        <div>
          <div class="text-xs font-medium text-txt mb-1">About L01 Containers</div>
          <p class="text-xs text-txt-muted">
            L01 is the logical evidence variant of the Expert Witness Format. Unlike E01
            (disk images), L01 stores individual files and directories with per-file
            MD5/SHA-1 hashes, timestamps, and full directory hierarchy. Compatible with
            EnCase, FTK Imager, AXIOM, and other forensic tools.
          </p>
        </div>
      </div>
    </div>
  );
};
