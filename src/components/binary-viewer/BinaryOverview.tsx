// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { For, Show } from "solid-js";
import { WarningIcon } from "../icons";
import { formatBytes } from "../../utils";
import type { BinaryInfo } from "./types";
import { formatHex, formatTimestamp } from "./helpers";

interface BinaryOverviewProps {
  data: BinaryInfo;
}

function PeInfoRow(props: { label: string; value: string | number | null | undefined; mono?: boolean }) {
  return (
    <Show when={props.value !== null && props.value !== undefined && props.value !== ""}>
      <div class="flex gap-2 text-xs py-0.5">
        <span class="text-txt-muted w-28 shrink-0">{props.label}</span>
        <span class={`${props.mono ? "font-mono" : ""} text-txt break-all`}>{props.value}</span>
      </div>
    </Show>
  );
}

const formatOptionalHex = (value: number | null) => value === null ? null : formatHex(value);
const formatOptionalBytes = (value: number | null) => value === null ? null : formatBytes(value);

export function BinaryOverview(props: BinaryOverviewProps) {
  const data = () => props.data;
  const versionInfoEntries = () => Object.entries(data().pe_version_info ?? {});
  const hasPeInformation = () =>
    data().pe_timestamp !== null ||
    data().pe_checksum !== null ||
    data().pe_subsystem ||
    data().pe_linker_version ||
    data().pe_os_version ||
    data().pe_image_version ||
    data().pe_subsystem_version ||
    data().pe_image_base !== null ||
    data().pe_section_alignment !== null ||
    data().pe_file_alignment !== null ||
    data().pe_size_of_image !== null ||
    data().pe_size_of_headers !== null ||
    data().pe_dll_characteristics ||
    data().pe_dll_characteristics_detail.length > 0 ||
    data().pe_certificate_table_size !== null;

  return (
    <>
      {/* Overview Grid */}
      <div class="grid grid-cols-2 gap-3">
        <div class="stat-box">
          <div class="text-txt-muted text-xs">Format</div>
          <div class="text-sm font-semibold text-txt">{data().format}</div>
        </div>
        <div class="stat-box">
          <div class="text-txt-muted text-xs">Architecture</div>
          <div class="text-sm font-semibold text-txt">
            {data().architecture} ({data().is_64bit ? "64-bit" : "32-bit"})
          </div>
        </div>
        <div class="stat-box">
          <div class="text-txt-muted text-xs">Entry Point</div>
          <div class="text-sm font-mono text-txt">
            {formatHex(data().entry_point)}
          </div>
        </div>
        <div class="stat-box">
          <div class="text-txt-muted text-xs">File Size</div>
          <div class="text-sm font-semibold text-txt">
            {formatBytes(data().file_size)}
          </div>
        </div>
      </div>

      {/* PE-specific info */}
      <Show when={hasPeInformation()}>
        <div class="card">
          <h3 class="text-xs font-semibold text-txt-secondary uppercase tracking-wider mb-2">
            PE Information
          </h3>
          <PeInfoRow label="Compile Time" value={data().pe_timestamp !== null ? formatTimestamp(data().pe_timestamp) : null} mono />
          <PeInfoRow label="Subsystem" value={data().pe_subsystem} />
          <PeInfoRow label="Checksum" value={formatOptionalHex(data().pe_checksum)} mono />
          <PeInfoRow label="Linker Version" value={data().pe_linker_version} />
          <PeInfoRow label="OS Version" value={data().pe_os_version} />
          <PeInfoRow label="Image Version" value={data().pe_image_version} />
          <PeInfoRow label="Subsystem Ver." value={data().pe_subsystem_version} />
          <PeInfoRow label="Image Base" value={formatOptionalHex(data().pe_image_base)} mono />
          <PeInfoRow label="Image Size" value={formatOptionalBytes(data().pe_size_of_image)} />
          <PeInfoRow label="Headers Size" value={formatOptionalBytes(data().pe_size_of_headers)} />
          <PeInfoRow label="Section Align" value={formatOptionalBytes(data().pe_section_alignment)} />
          <PeInfoRow label="File Align" value={formatOptionalBytes(data().pe_file_alignment)} />
          <PeInfoRow label="DLL Flags" value={data().pe_dll_characteristics} mono />
          <Show when={data().pe_dll_characteristics_detail.length > 0}>
            <div class="flex gap-2 text-xs py-0.5">
              <span class="text-txt-muted w-28 shrink-0">DLL Features</span>
              <div class="flex flex-wrap gap-1">
                {data().pe_dll_characteristics_detail.map((flag) => (
                  <span class="px-1.5 py-0.5 text-2xs bg-bg-hover text-txt-secondary rounded">
                    {flag}
                  </span>
                ))}
              </div>
            </div>
          </Show>
          <PeInfoRow label="Cert Table" value={formatOptionalBytes(data().pe_certificate_table_size)} />
        </div>
      </Show>

      {/* PE version resource strings */}
      <Show when={versionInfoEntries().length > 0}>
        <div class="card">
          <h3 class="text-xs font-semibold text-txt-secondary uppercase tracking-wider mb-2">
            Version Info
          </h3>
          <For each={versionInfoEntries()}>
            {([key, value]) => (
              <div class="flex gap-2 text-xs py-0.5">
                <span class="text-txt-muted w-28 shrink-0">{key}</span>
                <span class="text-txt break-all">{value}</span>
              </div>
            )}
          </For>
        </div>
      </Show>

      {/* Windows driver-specific info */}
      <Show when={data().pe_is_driver}>
        <div class="card">
          <h3 class="text-xs font-semibold text-txt-secondary uppercase tracking-wider mb-2">
            Driver Analysis
          </h3>
          <Show when={data().pe_driver_type}>
            <div class="flex gap-2 text-xs py-0.5">
              <span class="text-txt-muted w-24">Type</span>
              <span class="text-txt">{data().pe_driver_type}</span>
            </div>
          </Show>
          <Show when={data().pe_driver_indicators.length > 0}>
            <div class="flex gap-2 text-xs py-0.5">
              <span class="text-txt-muted w-24">Indicators</span>
              <div class="flex flex-wrap gap-1">
                {data().pe_driver_indicators.map((indicator) => (
                  <span class="px-1.5 py-0.5 text-2xs bg-bg-hover text-txt-secondary rounded">
                    {indicator}
                  </span>
                ))}
              </div>
            </div>
          </Show>
        </div>
      </Show>

      {/* Mach-O specific */}
      <Show when={data().macho_cpu_type || data().macho_filetype}>
        <div class="card">
          <h3 class="text-xs font-semibold text-txt-secondary uppercase tracking-wider mb-2">
            Mach-O Information
          </h3>
          <Show when={data().macho_cpu_type}>
            <div class="flex gap-2 text-xs py-0.5">
              <span class="text-txt-muted w-24">CPU Type</span>
              <span class="text-txt">{data().macho_cpu_type}</span>
            </div>
          </Show>
          <Show when={data().macho_filetype}>
            <div class="flex gap-2 text-xs py-0.5">
              <span class="text-txt-muted w-24">File Type</span>
              <span class="text-txt">{data().macho_filetype}</span>
            </div>
          </Show>
        </div>
      </Show>

      {/* Security Indicators */}
      <div class="card">
        <h3 class="text-xs font-semibold text-txt-secondary uppercase tracking-wider mb-2 flex items-center gap-1">
          <WarningIcon class="w-3 h-3" /> Security Indicators
        </h3>
        <div class="grid grid-cols-3 gap-2 text-xs">
          <div class="flex items-center gap-1.5">
            <span
              class={`w-2 h-2 rounded-full ${data().has_debug_info ? "bg-warning" : "bg-bg-hover"}`}
            />
            <span class="text-txt-secondary">Debug Info</span>
          </div>
          <div class="flex items-center gap-1.5">
            <span
              class={`w-2 h-2 rounded-full ${data().is_stripped ? "bg-error" : "bg-bg-hover"}`}
            />
            <span class="text-txt-secondary">Stripped</span>
          </div>
          <div class="flex items-center gap-1.5">
            <span
              class={`w-2 h-2 rounded-full ${data().has_code_signing ? "bg-success" : "bg-bg-hover"}`}
            />
            <span class="text-txt-secondary">Code Signed</span>
          </div>
        </div>
      </div>
    </>
  );
}
