// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

// Aff4ImageMode - AFF4 forensic container creation UI

import { Component, For, Accessor, Setter, createSignal } from "solid-js";
import {
  HiOutlineFingerPrint,
  HiOutlineInformationCircle,
} from "../icons";
import { CaseMetadataSection } from "./CaseMetadataSection";

// --- Props ---

export interface Aff4ImageModeProps {
  // Image name
  imageName: Accessor<string>;
  setImageName: Setter<string>;
  // Compression
  compression: Accessor<string>;
  setCompression: Setter<string>;
  // Hash algorithms (multi-select)
  hashAlgorithms: Accessor<string[]>;
  setHashAlgorithms: Setter<string[]>;
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
}

// Available hash algorithms for AFF4
const AFF4_HASH_OPTIONS = [
  { value: "md5", label: "MD5" },
  { value: "sha1", label: "SHA-1" },
  { value: "sha256", label: "SHA-256" },
  { value: "sha512", label: "SHA-512" },
  { value: "blake2b", label: "Blake2b" },
] as const;

// --- Component ---

export const Aff4ImageMode: Component<Aff4ImageModeProps> = (props) => {
  const [showCaseMetadata, setShowCaseMetadata] = createSignal(false);

  const toggleHash = (algo: string) => {
    const current = props.hashAlgorithms();
    if (current.includes(algo)) {
      // Don't allow removing the last algorithm
      if (current.length > 1) {
        props.setHashAlgorithms(current.filter((h) => h !== algo));
      }
    } else {
      props.setHashAlgorithms([...current, algo]);
    }
  };

  return (
    <div class="space-y-3">
      {/* Compact info callout */}
      <div class="flex items-start gap-2 bg-bg-secondary border border-border rounded-lg p-2.5">
        <HiOutlineInformationCircle class="w-4 h-4 text-info mt-0.5 flex-shrink-0" />
        <p class="text-xs text-txt-muted leading-relaxed">
          Creates an <span class="text-txt-secondary font-medium">AFF4 forensic container</span> using
          the Advanced Forensic Framework 4 format — an open standard based on ZIP with RDF metadata,
          content-addressable storage, and multiple compression/hash options.
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
            <span class="text-sm text-txt-muted font-mono">.aff4</span>
          </div>
        </div>
        <div class="space-y-1">
          <label class="label text-xs">Compression</label>
          <select
            class="input-sm"
            value={props.compression()}
            onChange={(e) => props.setCompression(e.currentTarget.value)}
          >
            <option value="deflate">Deflate (default)</option>
            <option value="lz4">LZ4 (fast)</option>
            <option value="snappy">Snappy (fast)</option>
            <option value="stored">None (Store)</option>
          </select>
        </div>
      </div>

      {/* Hash Algorithms - multi-select checkboxes */}
      <div class="space-y-2">
        <label class="label flex items-center gap-1">
          <HiOutlineFingerPrint class="w-3.5 h-3.5" />
          Hash Algorithms
        </label>
        <div class="flex gap-4 pl-1 flex-wrap">
          <For each={AFF4_HASH_OPTIONS}>
            {(opt) => (
              <label class="flex items-center gap-2 text-xs cursor-pointer">
                <input
                  type="checkbox"
                  checked={props.hashAlgorithms().includes(opt.value)}
                  onChange={() => toggleHash(opt.value)}
                  class="accent-accent"
                />
                <span class="text-txt">{opt.label}</span>
              </label>
            )}
          </For>
        </div>
        <div class="text-xs text-txt-muted pl-1">
          Select one or more hash algorithms for content verification
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

      {/* Info Card */}
      <div class="bg-bg-secondary border border-border rounded-lg p-3 space-y-2">
        <div class="text-xs font-medium text-txt-secondary">About AFF4 Format</div>
        <ul class="text-xs text-txt-muted space-y-1 list-disc pl-4">
          <li>Open forensic container standard (Advanced Forensic Framework 4)</li>
          <li>Content-addressable storage with configurable chunk sizes</li>
          <li>Multiple compression algorithms: Deflate, LZ4, Snappy, or uncompressed</li>
          <li>Multiple hash algorithms: MD5, SHA-1, SHA-256, SHA-512, Blake2b</li>
          <li>ZIP-based container with RDF metadata for interoperability</li>
        </ul>
      </div>
    </div>
  );
};
