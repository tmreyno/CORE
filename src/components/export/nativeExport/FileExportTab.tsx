// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show } from "solid-js";
import {
  HiOutlineArrowUpTray,
  HiOutlineInformationCircle,
} from "../../icons";
import type { NativeExportModeProps } from "./types";

/** Sanitize export name: strip invalid filename characters, trim whitespace */
function sanitizeExportName(value: string): string {
  // Remove characters not allowed in filenames (Windows + macOS + Linux)
  return value.replace(/[<>:"/\\|?*\x00-\x1f]/g, "").trim();
}

/** File export sub-tab UI: export name, hash/verify/manifest checkboxes */
export const FileExportTab: Component<
  Pick<
    NativeExportModeProps,
    | "exportName"
    | "setExportName"
    | "computeHashes"
    | "setComputeHashes"
    | "verifyAfterCopy"
    | "setVerifyAfterCopy"
    | "generateJsonManifest"
    | "setGenerateJsonManifest"
    | "generateTxtReport"
    | "setGenerateTxtReport"
  >
> = (props) => {
  return (
    <div class="space-y-3 p-3 bg-bg-secondary rounded-lg border border-border">
      <h3 class="text-sm font-semibold text-txt flex items-center gap-2">
        <HiOutlineArrowUpTray class="w-4 h-4" />
        Export Options
      </h3>

      {/* Export Name */}
      <div class="space-y-1">
        <label class="label text-xs">Export Name</label>
        <input
          class="input input-sm"
          type="text"
          value={props.exportName()}
          onInput={(e) => {
            const sanitized = sanitizeExportName(e.currentTarget.value);
            props.setExportName(sanitized);
          }}
          onBlur={() => {
            // Default to "forensic_export" if empty after blur
            if (!props.exportName().trim()) {
              props.setExportName("forensic_export");
            }
          }}
          placeholder="forensic_export"
        />
        <p class="text-2xs text-txt-muted leading-tight">
          Used for manifest and report filenames. Invalid characters are removed automatically.
        </p>
      </div>

      {/* Checkbox Options */}
      <div class="space-y-2">
        <label class="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            checked={props.computeHashes()}
            onChange={(e) => props.setComputeHashes(e.currentTarget.checked)}
            class="w-4 h-4"
          />
          <span class="text-xs text-txt">Compute SHA-256 hashes</span>
        </label>

        <Show when={props.computeHashes()}>
          <label class="flex items-center gap-2 cursor-pointer ml-6">
            <input
              type="checkbox"
              checked={props.verifyAfterCopy()}
              onChange={(e) => props.setVerifyAfterCopy(e.currentTarget.checked)}
              class="w-4 h-4"
            />
            <span class="text-xs text-txt">Verify copied files</span>
          </label>

          <label class="flex items-center gap-2 cursor-pointer ml-6">
            <input
              type="checkbox"
              checked={props.generateJsonManifest()}
              onChange={(e) => props.setGenerateJsonManifest(e.currentTarget.checked)}
              class="w-4 h-4"
            />
            <span class="text-xs text-txt">Generate JSON manifest</span>
          </label>

          <label class="flex items-center gap-2 cursor-pointer ml-6">
            <input
              type="checkbox"
              checked={props.generateTxtReport()}
              onChange={(e) => props.setGenerateTxtReport(e.currentTarget.checked)}
              class="w-4 h-4"
            />
            <span class="text-xs text-txt">Generate TXT report</span>
          </label>
        </Show>
      </div>

      <div class="p-2 bg-bg-panel rounded border border-info/20 text-xs text-info">
        <HiOutlineInformationCircle class="w-3 h-3 inline mr-1" />
        Forensic export includes timestamps, hashes, and manifests for chain-of-custody
      </div>
    </div>
  );
};
