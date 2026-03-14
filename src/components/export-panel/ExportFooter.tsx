// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show } from "solid-js";
import { HiOutlinePlay, HiOutlineStop } from "../icons";
import type { ExportMode } from "../../hooks/useExportState";
import type { Accessor } from "solid-js";

interface ExportFooterProps {
  mode: Accessor<ExportMode>;
  sources: Accessor<string[]>;
  destination: Accessor<string>;
  isProcessing: Accessor<boolean>;
  isAcquiring: Accessor<boolean>;
  nativeExportTab: Accessor<string>;
  toolsTab: Accessor<string>;
  testArchivePath: Accessor<string>;
  repairCorruptedPath: Accessor<string>;
  repairOutputPath: Accessor<string>;
  validateArchivePath: Accessor<string>;
  extractFirstVolume: Accessor<string>;
  extractOutputDir: Accessor<string>;
  lzmaInputPath: Accessor<string>;
  lzmaOutputPath: Accessor<string>;
  lzmaDecompressInput: Accessor<string>;
  lzmaDecompressOutput: Accessor<string>;
  activeExportOperationId?: Accessor<string | null>;
  onStart: () => void;
  onToolAction: () => void;
  onCancelExport?: () => void;
}

export function ExportFooter(props: ExportFooterProps) {
  return (
    <div class="p-3 border-t border-border flex justify-between items-center">
      <Show when={props.mode() !== "tools"}>
        <div class="text-xs text-txt-muted">
          {props.sources().length} item{props.sources().length !== 1 ? "s" : ""} selected
        </div>

        <div class="flex items-center gap-2">
          <Show when={props.activeExportOperationId?.() && props.onCancelExport}>
            <button
              class="btn-sm btn-secondary"
              onClick={props.onCancelExport}
            >
              <HiOutlineStop class="w-4 h-4" />
              Cancel Export
            </button>
          </Show>

          <button
            class="btn-sm-primary"
            onClick={props.onStart}
            disabled={props.isProcessing() || props.isAcquiring() || props.sources().length === 0 || !props.destination()}
          >
            <Show when={!props.isProcessing() && !props.isAcquiring()} fallback={<span>{props.isAcquiring() ? "Acquisition in progress..." : "Processing..."}</span>}>
              <HiOutlinePlay class="w-4 h-4" />
              Start{" "}
              {props.mode() === "physical"
                ? "E01 Image"
                : props.mode() === "logical"
                  ? "L01 Image"
                  : props.nativeExportTab() === "archive"
                    ? "Archive"
                    : "Export"}
            </Show>
          </button>
        </div>
      </Show>

      <Show when={props.mode() === "tools"}>
        <div class="flex-1" />
        <button
          class="btn-sm-primary"
          onClick={props.onToolAction}
          disabled={
            props.isProcessing() ||
            (props.toolsTab() === "test" && !props.testArchivePath()) ||
            (props.toolsTab() === "repair" && (!props.repairCorruptedPath() || !props.repairOutputPath())) ||
            (props.toolsTab() === "validate" && !props.validateArchivePath()) ||
            (props.toolsTab() === "extract" && (!props.extractFirstVolume() || !props.extractOutputDir())) ||
            (props.toolsTab() === "compress" && (!props.lzmaInputPath() || !props.lzmaOutputPath())) ||
            (props.toolsTab() === "decompress" && (!props.lzmaDecompressInput() || !props.lzmaDecompressOutput()))
          }
        >
          <Show when={!props.isProcessing()} fallback={<span>Processing...</span>}>
            <HiOutlinePlay class="w-4 h-4" />
            {props.toolsTab() === "test" && "Test Archive"}
            {props.toolsTab() === "repair" && "Repair Archive"}
            {props.toolsTab() === "validate" && "Validate Archive"}
            {props.toolsTab() === "extract" && "Extract Archive"}
            {props.toolsTab() === "compress" && "Compress"}
            {props.toolsTab() === "decompress" && "Decompress"}
          </Show>
        </button>
      </Show>
    </div>
  );
}
