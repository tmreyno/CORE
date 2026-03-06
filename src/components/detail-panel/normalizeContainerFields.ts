// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * normalizeContainerFields — maps container info (AD1, E01, L01, Raw,
 * Archive, UFED, companion log) to a uniform InfoField[] array for display.
 */

import type { ContainerInfo, StoredHash } from "../../types";
import type { InfoField } from "./types";
import { formatOffsetLabel } from "../../utils";
import { getBasename } from "../../utils/pathUtils";

export function normalizeContainerFields(
  info: ContainerInfo,
  storedHashes: StoredHash[],
): InfoField[] {
  const fields: InfoField[] = [];

  // AD1
  if (info.ad1) {
    const ad1 = info.ad1;
    const log = ad1.companion_log;
    const vol = ad1.volume;

    // Show warning if segments are missing
    if (ad1.missing_segments && ad1.missing_segments.length > 0) {
      fields.push({
        label: "⚠ Incomplete",
        value: `Missing ${ad1.missing_segments.length} segment(s): ${ad1.missing_segments.join(", ")}`,
        type: "full-width",
        format: "warning",
      });
    }

    fields.push(
      { label: "Format", value: `AD1 (${ad1.logical.signature})` },
      { label: "Version", value: ad1.logical.image_version },
      {
        label: "Segments",
        value: `${ad1.segment_files?.length ?? 0} / ${ad1.segment.segment_number}${ad1.missing_segments?.length ? " (incomplete)" : ""}`,
      },
      { label: "Total Size", value: ad1.total_size, format: "bytes" },
      { label: "Items", value: ad1.item_count },
      // Case metadata from companion log
      { label: "Case #", value: log?.case_number, type: "highlight" },
      { label: "Evidence #", value: log?.evidence_number, type: "highlight" },
      { label: "Examiner", value: log?.examiner },
      { label: "Acquired", value: log?.acquisition_date },
      // Volume/system info from header
      { label: "Volume", value: vol?.volume_label },
      { label: "Filesystem", value: vol?.filesystem },
      { label: "OS", value: vol?.os_info },
      { label: "Block Size", value: vol?.block_size, format: "bytes" },
      // Technical details
      { label: "Chunk Size", value: ad1.logical.zlib_chunk_size, format: "bytes" },
      { label: "Source", value: ad1.logical.data_source_name, type: "full-width" },
      // Notes
      { label: "Notes", value: log?.notes, type: "full-width", format: "notes" },
    );
  }

  // E01
  if (info.e01) {
    const e01 = info.e01;
    fields.push(
      { label: "Format", value: e01.format_version },
      { label: "Segments", value: e01.segment_count },
      { label: "Total Size", value: e01.total_size, format: "bytes" },
      { label: "Compression", value: e01.compression },
      { label: "Bytes/Sector", value: e01.bytes_per_sector },
      { label: "Sectors/Chunk", value: e01.sectors_per_chunk },
      { label: "Case #", value: e01.case_number, type: "highlight" },
      { label: "Evidence #", value: e01.evidence_number, type: "highlight" },
      { label: "Examiner", value: e01.examiner_name },
      { label: "Acquired", value: e01.acquiry_date },
      { label: "System Date", value: e01.system_date },
      { label: "Model", value: e01.model, type: "device" },
      { label: "Serial #", value: e01.serial_number, type: "device" },
      { label: "Description", value: e01.description, type: "full-width" },
      { label: "Notes", value: e01.notes, type: "full-width", format: "notes" },
    );
  }

  // L01 (Logical Evidence - uses same EwfInfo type as E01)
  if (info.l01) {
    const l01 = info.l01;
    fields.push(
      { label: "Format", value: l01.format_version },
      { label: "Segments", value: l01.segment_count },
      { label: "Total Size", value: l01.total_size, format: "bytes" },
      { label: "Compression", value: l01.compression },
      { label: "Bytes/Sector", value: l01.bytes_per_sector },
      { label: "Sectors/Chunk", value: l01.sectors_per_chunk },
      { label: "Case #", value: l01.case_number, type: "highlight" },
      { label: "Evidence #", value: l01.evidence_number, type: "highlight" },
      { label: "Examiner", value: l01.examiner_name },
      { label: "Acquired", value: l01.acquiry_date },
      { label: "System Date", value: l01.system_date },
      { label: "Model", value: l01.model, type: "device" },
      { label: "Serial #", value: l01.serial_number, type: "device" },
      { label: "Description", value: l01.description, type: "full-width" },
      { label: "Notes", value: l01.notes, type: "full-width", format: "notes" },
    );
  }

  // Raw
  if (info.raw) {
    const raw = info.raw;
    fields.push(
      { label: "Format", value: "Raw Image" },
      { label: "Segments", value: raw.segment_count },
      { label: "Total Size", value: raw.total_size, format: "bytes" },
    );
    if (raw.segment_count > 1) {
      const segList =
        raw.segment_names.slice(0, 5).join(", ") +
        (raw.segment_count > 5 ? ` (+${raw.segment_count - 5} more)` : "");
      fields.push({
        label: "Segment Files",
        value: segList,
        type: "full-width",
        format: "list",
      });
    }
  }

  // Archive (ZIP/7z)
  if (info.archive) {
    const archive = info.archive;
    fields.push(
      {
        label: "Format",
        value: `${archive.format}${archive.version ? ` v${archive.version}` : ""}`,
      },
      { label: "Segments", value: archive.segment_count },
      { label: "Total Size", value: archive.total_size, format: "bytes" },
      { label: "Entries", value: archive.entry_count },
      {
        label: "AES Encrypted",
        value: archive.aes_encrypted ? "Yes" : undefined,
        type: "highlight",
      },
      {
        label: "Encrypted Headers",
        value: archive.encrypted_headers ? "Filenames Hidden" : undefined,
        type: "highlight",
      },
    );
    if (
      archive.start_header_crc_valid !== undefined &&
      archive.start_header_crc_valid !== null
    ) {
      fields.push({
        label: "Header CRC",
        value: archive.start_header_crc_valid ? "✓ Valid" : "✗ Invalid",
        type: archive.start_header_crc_valid ? "normal" : "highlight",
        condition: true,
      });
    }
    fields.push(
      {
        label: "Central Dir",
        value: archive.central_dir_offset
          ? `@ ${archive.central_dir_offset.toLocaleString()}`
          : undefined,
      },
      {
        label: "Next Header",
        value: archive.next_header_offset
          ? formatOffsetLabel(archive.next_header_offset)
          : undefined,
      },
    );
    if (archive.segment_count > 1) {
      const segList =
        archive.segment_names.slice(0, 5).join(", ") +
        (archive.segment_count > 5
          ? ` (+${archive.segment_count - 5} more)`
          : "");
      fields.push({
        label: "Segment Files",
        value: segList,
        type: "full-width",
        format: "list",
      });
    }
  }

  // UFED (Cellebrite)
  if (info.ufed) {
    const ufed = info.ufed;
    const allFiles: string[] = [...ufed.associated_files.map((f) => f.filename)];
    if (ufed.collection_info?.ufdx_path) {
      const ufdxName = getBasename(ufed.collection_info.ufdx_path);
      if (ufdxName && !allFiles.includes(ufdxName)) allFiles.push(ufdxName);
    }

    fields.push(
      { label: "Format", value: `UFED (${ufed.format})` },
      { label: "Total Size", value: ufed.size, format: "bytes" },
      { label: "Extraction", value: ufed.extraction_info?.extraction_type },
      {
        label: "Tool",
        value: ufed.extraction_info?.acquisition_tool
          ? `${ufed.extraction_info.acquisition_tool}${ufed.extraction_info.tool_version ? ` v${ufed.extraction_info.tool_version}` : ""}`
          : undefined,
      },
      {
        label: "Case #",
        value: ufed.case_info?.case_identifier,
        type: "highlight",
      },
      {
        label: "Evidence #",
        value: ufed.case_info?.device_name || ufed.evidence_number,
        type: "highlight",
      },
      { label: "Examiner", value: ufed.case_info?.examiner_name },
      { label: "Acquired", value: ufed.extraction_info?.start_time },
      { label: "Completed", value: ufed.extraction_info?.end_time },
      {
        label: "Device",
        value: ufed.device_info?.full_name || ufed.device_hint,
        type: "device",
      },
      { label: "Model", value: ufed.device_info?.model, type: "device" },
      {
        label: "Serial #",
        value: ufed.device_info?.serial_number,
        type: "device",
      },
      {
        label: "IMEI",
        value: ufed.device_info?.imei
          ? `${ufed.device_info.imei}${ufed.device_info.imei2 ? ` / ${ufed.device_info.imei2}` : ""}`
          : undefined,
        type: "device",
      },
      {
        label: "OS",
        value: ufed.device_info?.os_version
          ? `${ufed.device_info.vendor ? `${ufed.device_info.vendor} ` : ""}${ufed.device_info.os_version}`
          : undefined,
        type: "device",
      },
      {
        label: "Connection",
        value: ufed.extraction_info?.connection_type,
        type: "full-width",
      },
      {
        label: "Location",
        value: ufed.case_info?.location,
        type: "full-width",
      },
      {
        label: "GUID",
        value: ufed.extraction_info?.guid,
        type: "full-width",
        format: "mono",
      },
      {
        label: "Files",
        value: allFiles.length > 0 ? allFiles.join(", ") : undefined,
        type: "full-width",
      },
    );
  }

  // Companion log
  if (info.companion_log) {
    const clog = info.companion_log;
    fields.push(
      { label: "Created By", value: clog.created_by },
      { label: "Case #", value: clog.case_number, type: "highlight" },
      { label: "Evidence #", value: clog.evidence_number, type: "highlight" },
      { label: "Examiner", value: clog.examiner },
      { label: "Acquired", value: clog.acquisition_started },
      {
        label: "Source",
        value: clog.unique_description,
        type: "full-width",
      },
      {
        label: "Notes",
        value: clog.notes,
        type: "full-width",
        format: "notes",
      },
    );
  }

  // Add all stored hashes (unified display for all container types)
  if (storedHashes && storedHashes.length > 0) {
    for (const sh of storedHashes) {
      const algo = sh.algorithm?.toUpperCase() || "HASH";
      const hash = sh.hash || "";
      const sourceLabel =
        sh.source === "container"
          ? "◆"
          : sh.source === "companion"
            ? "◇"
            : "▣";
      const verifyIcon =
        sh.verified === true ? " ✓" : sh.verified === false ? " ✗" : "";
      // Show filename if available (UFED has per-file hashes)
      const filenameLabel = sh.filename ? ` (${sh.filename})` : "";
      fields.push({
        label: `${sourceLabel} ${algo}${verifyIcon}${filenameLabel}`,
        value: hash,
        type: "hash",
      });
    }
  }

  return fields;
}
