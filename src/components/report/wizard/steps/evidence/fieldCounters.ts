// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Metadata field counter helpers for the evidence info badge.
 */

import type { ContainerInfo } from "../../../../../types/containerInfo";

export function countEwfFields(ci: ContainerInfo): number {
  const ewf = ci.e01 || ci.l01;
  if (!ewf) return 0;
  let count = 3; // format, compression, size always present
  if (ewf.case_number) count++;
  if (ewf.examiner_name) count++;
  if (ewf.evidence_number) count++;
  if (ewf.description) count++;
  if (ewf.model) count++;
  if (ewf.serial_number) count++;
  if (ewf.stored_hashes?.length) count++;
  return count;
}

export function countAd1Fields(ci: ContainerInfo): number {
  const log = ci.ad1?.companion_log;
  if (!log) return 0;
  let count = 0;
  if (log.case_number) count++;
  if (log.examiner) count++;
  if (log.evidence_number) count++;
  if (log.acquisition_date) count++;
  if (log.source_device) count++;
  if (log.acquisition_tool) count++;
  return count;
}

export function countUfedFields(ci: ContainerInfo): number {
  const ufed = ci.ufed;
  if (!ufed) return 0;
  let count = 2; // format, size always present
  if (ufed.device_info) count += 3;
  if (ufed.case_info) count += 2;
  if (ufed.extraction_info) count += 3;
  return count;
}
