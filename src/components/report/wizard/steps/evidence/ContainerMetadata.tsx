// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ContainerMetadata — renders container-specific metadata panels (EWF, AD1, UFED, Archive, Raw).
 */

import { Show } from "solid-js";
import { formatBytes } from "../../../../../utils";
import {
  HiOutlineCpuChip,
  HiOutlineDocumentText,
  HiOutlineDevicePhoneMobile,
  HiOutlineServer,
} from "../../../../icons";
import type { ContainerInfo } from "../../../../../types/containerInfo";
import { MetaRow } from "./MetaRow";

export function ContainerMetadata(props: { info: ContainerInfo | undefined }) {
  return (
    <Show when={props.info}>
      {(ci) => (
        <>
          {/* EWF (E01/L01) Metadata */}
          <Show when={ci().e01 || ci().l01}>
            {(_ewf) => {
              const ewf = () => ci().e01 || ci().l01;
              return (
                <div class="space-y-1.5 p-2.5 bg-bg/50 rounded-lg border border-border/20">
                  <div class="flex items-center gap-1.5 text-xs font-medium text-accent/80 mb-1.5">
                    <HiOutlineCpuChip class="w-3.5 h-3.5" />
                    <span>EWF Container Details</span>
                  </div>
                  <div class="grid grid-cols-2 gap-x-4 gap-y-1">
                    <MetaRow label="Format" value={ewf()!.format_version} />
                    <MetaRow label="Segments" value={ewf()!.segment_count} />
                    <MetaRow label="Compression" value={ewf()!.compression} />
                    <MetaRow label="Sector Size" value={ewf()!.bytes_per_sector ? `${ewf()!.bytes_per_sector} bytes` : undefined} />
                    <MetaRow label="Total Size" value={ewf()!.total_size ? formatBytes(ewf()!.total_size) : undefined} />
                    <MetaRow label="Chunk Count" value={ewf()!.chunk_count} />
                    <MetaRow label="Case Number" value={ewf()!.case_number} />
                    <MetaRow label="Evidence #" value={ewf()!.evidence_number} />
                    <MetaRow label="Examiner" value={ewf()!.examiner_name} />
                    <MetaRow label="Acquiry Date" value={ewf()!.acquiry_date} />
                    <MetaRow label="Model" value={ewf()!.model} />
                    <MetaRow label="Serial" value={ewf()!.serial_number} />
                  </div>
                  <Show when={ewf()!.description}>
                    <div class="mt-1">
                      <MetaRow label="Description" value={ewf()!.description} />
                    </div>
                  </Show>
                  <Show when={ewf()!.notes}>
                    <div class="mt-1">
                      <MetaRow label="Notes" value={ewf()!.notes} />
                    </div>
                  </Show>
                </div>
              );
            }}
          </Show>

          {/* AD1 Companion Log Metadata */}
          <Show when={ci().ad1?.companion_log}>
            {(log) => (
              <div class="space-y-1.5 p-2.5 bg-bg/50 rounded-lg border border-border/20">
                <div class="flex items-center gap-1.5 text-xs font-medium text-accent/80 mb-1.5">
                  <HiOutlineDocumentText class="w-3.5 h-3.5" />
                  <span>AD1 Companion Log</span>
                </div>
                <div class="grid grid-cols-2 gap-x-4 gap-y-1">
                  <MetaRow label="Case Number" value={log().case_number} />
                  <MetaRow label="Evidence #" value={log().evidence_number} />
                  <MetaRow label="Examiner" value={log().examiner} />
                  <MetaRow label="Organization" value={log().organization} />
                  <MetaRow label="Acq. Date" value={log().acquisition_date} />
                  <MetaRow label="Acq. Tool" value={log().acquisition_tool} />
                  <MetaRow label="Acq. Method" value={log().acquisition_method} />
                  <MetaRow label="Source Device" value={log().source_device} />
                  <MetaRow label="Source Path" value={log().source_path} />
                  <MetaRow label="Total Items" value={log().total_items} />
                  <MetaRow label="Total Size" value={log().total_size ? formatBytes(log().total_size!) : undefined} />
                </div>
                <Show when={log().notes}>
                  <div class="mt-1">
                    <MetaRow label="Notes" value={log().notes} />
                  </div>
                </Show>
              </div>
            )}
          </Show>

          {/* AD1 Segment Info */}
          <Show when={ci().ad1 && !ci().ad1!.companion_log}>
            {(_ad1) => {
              const ad1 = () => ci().ad1!;
              return (
                <div class="space-y-1.5 p-2.5 bg-bg/50 rounded-lg border border-border/20">
                  <div class="flex items-center gap-1.5 text-xs font-medium text-accent/80 mb-1.5">
                    <HiOutlineDocumentText class="w-3.5 h-3.5" />
                    <span>AD1 Container</span>
                  </div>
                  <div class="grid grid-cols-2 gap-x-4 gap-y-1">
                    <MetaRow label="Item Count" value={ad1().item_count} />
                    <MetaRow label="Total Size" value={ad1().total_size ? formatBytes(ad1().total_size!) : undefined} />
                    <Show when={ad1().segment_files}>
                      <MetaRow label="Segments" value={ad1().segment_files!.length} />
                    </Show>
                    <Show when={ad1().missing_segments && ad1().missing_segments!.length > 0}>
                      <MetaRow label="Missing" value={ad1().missing_segments!.join(", ")} />
                    </Show>
                  </div>
                </div>
              );
            }}
          </Show>

          {/* UFED Metadata */}
          <Show when={ci().ufed}>
            {(ufed) => (
              <div class="space-y-1.5 p-2.5 bg-bg/50 rounded-lg border border-border/20">
                <div class="flex items-center gap-1.5 text-xs font-medium text-accent/80 mb-1.5">
                  <HiOutlineDevicePhoneMobile class="w-3.5 h-3.5" />
                  <span>UFED / Cellebrite</span>
                </div>
                <div class="grid grid-cols-2 gap-x-4 gap-y-1">
                  <MetaRow label="Format" value={ufed().format} />
                  <MetaRow label="Size" value={formatBytes(ufed().size)} />
                  <Show when={ufed().device_info}>
                    <MetaRow label="Device" value={ufed().device_info!.full_name || `${ufed().device_info!.vendor || ""} ${ufed().device_info!.model || ""}`.trim()} />
                    <MetaRow label="IMEI" value={ufed().device_info!.imei} />
                    <MetaRow label="OS Version" value={ufed().device_info!.os_version} />
                    <MetaRow label="Serial" value={ufed().device_info!.serial_number} />
                  </Show>
                  <Show when={ufed().case_info}>
                    <MetaRow label="Case ID" value={ufed().case_info!.case_identifier} />
                    <MetaRow label="Crime Type" value={ufed().case_info!.crime_type} />
                    <MetaRow label="Department" value={ufed().case_info!.department} />
                    <MetaRow label="Examiner" value={ufed().case_info!.examiner_name} />
                  </Show>
                  <Show when={ufed().extraction_info}>
                    <MetaRow label="Acq. Tool" value={ufed().extraction_info!.acquisition_tool} />
                    <MetaRow label="Tool Version" value={ufed().extraction_info!.tool_version} />
                    <MetaRow label="Extraction" value={ufed().extraction_info!.extraction_type} />
                    <MetaRow label="Started" value={ufed().extraction_info!.start_time} />
                    <MetaRow label="Ended" value={ufed().extraction_info!.end_time} />
                  </Show>
                </div>
              </div>
            )}
          </Show>

          {/* Archive Metadata */}
          <Show when={ci().archive}>
            {(arch) => (
              <div class="space-y-1.5 p-2.5 bg-bg/50 rounded-lg border border-border/20">
                <div class="flex items-center gap-1.5 text-xs font-medium text-accent/80 mb-1.5">
                  <HiOutlineServer class="w-3.5 h-3.5" />
                  <span>Archive Details</span>
                </div>
                <div class="grid grid-cols-2 gap-x-4 gap-y-1">
                  <MetaRow label="Format" value={arch().format} />
                  <MetaRow label="Total Size" value={formatBytes(arch().total_size)} />
                  <MetaRow label="Entries" value={arch().entry_count} />
                  <MetaRow label="Encrypted" value={arch().aes_encrypted ? "Yes (AES)" : arch().encrypted_headers ? "Yes (Headers)" : "No"} />
                  <Show when={arch().is_multipart}>
                    <MetaRow label="Segments" value={arch().segment_count} />
                  </Show>
                  <Show when={arch().version}>
                    <MetaRow label="Version" value={arch().version} />
                  </Show>
                  <Show when={arch().cellebrite_detected}>
                    <MetaRow label="Cellebrite" value={`Detected (${arch().cellebrite_files?.length || 0} files)`} />
                  </Show>
                </div>
              </div>
            )}
          </Show>

          {/* Raw Image Metadata */}
          <Show when={ci().raw}>
            {(raw) => (
              <div class="space-y-1.5 p-2.5 bg-bg/50 rounded-lg border border-border/20">
                <div class="flex items-center gap-1.5 text-xs font-medium text-accent/80 mb-1.5">
                  <HiOutlineServer class="w-3.5 h-3.5" />
                  <span>Raw Image</span>
                </div>
                <div class="grid grid-cols-2 gap-x-4 gap-y-1">
                  <MetaRow label="Segments" value={raw().segment_count} />
                  <MetaRow label="Total Size" value={formatBytes(raw().total_size)} />
                  <MetaRow label="First" value={raw().first_segment} />
                  <MetaRow label="Last" value={raw().last_segment} />
                </div>
              </div>
            )}
          </Show>

          {/* Companion Log (E01/other) */}
          <Show when={ci().companion_log}>
            {(log) => (
              <div class="space-y-1.5 p-2.5 bg-bg/50 rounded-lg border border-border/20">
                <div class="flex items-center gap-1.5 text-xs font-medium text-accent/80 mb-1.5">
                  <HiOutlineDocumentText class="w-3.5 h-3.5" />
                  <span>Companion Log</span>
                </div>
                <div class="grid grid-cols-2 gap-x-4 gap-y-1">
                  <MetaRow label="Case Number" value={log().case_number} />
                  <MetaRow label="Evidence #" value={log().evidence_number} />
                  <MetaRow label="Examiner" value={log().examiner} />
                  <MetaRow label="Description" value={log().unique_description} />
                  <MetaRow label="Acq. Started" value={log().acquisition_started} />
                  <MetaRow label="Acq. Finished" value={log().acquisition_finished} />
                </div>
              </div>
            )}
          </Show>
        </>
      )}
    </Show>
  );
}
