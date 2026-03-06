// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * BinaryViewer - PE/ELF/Mach-O binary analysis viewer
 *
 * Analyzes executable files and displays:
 * - Format, architecture, entry point
 * - Sections with sizes and characteristics
 * - Import libraries and functions
 * - Export symbols
 * - Forensic indicators (PE timestamps, debug info, code signing)
 */

import { Show } from "solid-js";
import { ChipIcon } from "../icons";
import { HiOutlineExclamationTriangle } from "../icons";
import { formatBytes } from "../../utils";
import type { BinaryViewerProps } from "./types";
import { useBinaryData } from "./useBinaryData";
import { BinaryOverview } from "./BinaryOverview";
import { SectionsPanel } from "./SectionsPanel";
import { ImportsPanel } from "./ImportsPanel";
import { ExportsPanel } from "./ExportsPanel";

export function BinaryViewer(props: BinaryViewerProps) {
  const bv = useBinaryData(props);

  return (
    <div class={`binary-viewer flex flex-col h-full ${props.class || ""}`}>
      {/* Toolbar */}
      <div class="flex items-center gap-2 p-2 border-b border-border bg-bg-secondary">
        <ChipIcon class="w-4 h-4 text-accent" />
        <span class="text-sm font-medium truncate" title={bv.filename()}>{bv.filename()}</span>
        <Show when={bv.badge()}>
          <span class={`text-xs px-1.5 py-0.5 rounded border ${bv.badge()!.color}`}>
            {bv.badge()!.label}
          </span>
        </Show>
        <Show when={bv.info()}>
          <span class="text-xs text-txt-muted">{bv.info()!.architecture}</span>
          <span class="text-xs text-txt-muted">{formatBytes(bv.info()!.file_size)}</span>
        </Show>
      </div>

      {/* Content */}
      <div class="flex-1 overflow-y-auto">
        <Show
          when={!bv.loading()}
          fallback={
            <div class="flex flex-col items-center justify-center h-full gap-2">
              <div class="animate-spin w-8 h-8 border-2 border-accent border-t-transparent rounded-full" />
              <span class="text-txt-muted">Analyzing binary...</span>
            </div>
          }
        >
          <Show
            when={!bv.error()}
            fallback={
              <div class="flex flex-col items-center gap-3 text-txt-muted p-6 max-w-md mx-auto text-center">
                <HiOutlineExclamationTriangle class="w-10 h-10 text-warning" />
                <span class="font-medium text-txt">Not a recognized executable</span>
                <p class="text-sm leading-relaxed">
                  <span class="font-mono text-xs bg-bg-secondary px-1.5 py-0.5 rounded">{bv.filename()}</span>{" "}
                  has a binary file extension but is not a PE, ELF, or Mach-O executable.
                  Use the <span class="font-semibold text-txt">Hex</span> button in the toolbar to inspect the raw bytes.
                </p>
                <button onClick={bv.loadBinary} class="btn btn-secondary btn-sm mt-1">Retry Analysis</button>
              </div>
            }
          >
            <Show when={bv.info()}>
              {(data) => (
                <div class="p-3 space-y-3">
                  <BinaryOverview data={data()} />
                  <SectionsPanel
                    sections={data().sections}
                    open={bv.showSections}
                    onToggle={() => bv.setShowSections(!bv.showSections())}
                  />
                  <ImportsPanel
                    imports={data().imports}
                    totalFunctions={bv.totalImportFunctions}
                    open={bv.showImports}
                    onToggle={() => bv.setShowImports(!bv.showImports())}
                    filteredImports={bv.filteredImports}
                    importFilter={bv.importFilter}
                    setImportFilter={bv.setImportFilter}
                  />
                  <ExportsPanel
                    exports={data().exports}
                    open={bv.showExports}
                    onToggle={() => bv.setShowExports(!bv.showExports())}
                  />
                </div>
              )}
            </Show>
          </Show>
        </Show>
      </div>
    </div>
  );
}
