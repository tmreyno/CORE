// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

// CaseMetadataSection - Shared collapsible case metadata for E01, L01, and 7z exports

import { Component, Show, Accessor, Setter } from "solid-js";
import {
  HiOutlineChevronDown,
  HiOutlineChevronRight,
  HiOutlineDocumentCheck,
} from "../icons";

export interface CaseMetadataSectionProps {
  isOpen: Accessor<boolean>;
  setIsOpen: (v: boolean) => void;
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

export const CaseMetadataSection: Component<CaseMetadataSectionProps> = (props) => (
  <div class="space-y-2">
    <button
      class="flex items-center gap-1 text-xs text-txt-secondary hover:text-txt"
      onClick={() => props.setIsOpen(!props.isOpen())}
    >
      <Show when={props.isOpen()} fallback={<HiOutlineChevronRight class="w-4 h-4" />}>
        <HiOutlineChevronDown class="w-4 h-4" />
      </Show>
      <HiOutlineDocumentCheck class="w-4 h-4" />
      Case Metadata
    </button>

    <Show when={props.isOpen()}>
      <div class="space-y-2 pl-5 pt-1">
        <div class="grid grid-cols-2 gap-2">
          <div class="space-y-1">
            <label class="label text-xs">Case Number</label>
            <input
              class="input-sm"
              type="text"
              value={props.caseNumber()}
              onInput={(e) => props.setCaseNumber(e.currentTarget.value)}
              placeholder="Case #"
            />
          </div>
          <div class="space-y-1">
            <label class="label text-xs">Evidence Number</label>
            <input
              class="input-sm"
              type="text"
              value={props.evidenceNumber()}
              onInput={(e) => props.setEvidenceNumber(e.currentTarget.value)}
              placeholder="Evidence #"
            />
          </div>
        </div>

        <div class="space-y-1">
          <label class="label text-xs">Examiner</label>
          <input
            class="input-sm"
            type="text"
            value={props.examinerName()}
            onInput={(e) => props.setExaminerName(e.currentTarget.value)}
            placeholder="Examiner name"
          />
        </div>

        <div class="space-y-1">
          <label class="label text-xs">Description</label>
          <input
            class="input-sm"
            type="text"
            value={props.description()}
            onInput={(e) => props.setDescription(e.currentTarget.value)}
            placeholder="Evidence description"
          />
        </div>

        <div class="space-y-1">
          <label class="label text-xs">Notes</label>
          <textarea
            class="input-sm min-h-[60px] resize-y"
            value={props.notes()}
            onInput={(e) => props.setNotes(e.currentTarget.value)}
            placeholder="Additional notes..."
            rows={2}
          />
        </div>
      </div>
    </Show>
  </div>
);
