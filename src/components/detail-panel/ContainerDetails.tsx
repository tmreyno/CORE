// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ContainerDetails — renders an info card with normalized container
 * metadata fields for any supported container type.
 */

import { createMemo } from "solid-js";
import { HiOutlineClipboardDocument } from "../icons";
import type { ContainerInfo, StoredHash } from "../../types";
import { normalizeContainerFields } from "./normalizeContainerFields";
import { InfoRows } from "./InfoRow";

export function ContainerDetails(props: {
  info: ContainerInfo;
  storedHashes: StoredHash[];
}) {
  const fields = createMemo(() =>
    normalizeContainerFields(props.info, props.storedHashes),
  );

  return (
    <div class="info-card">
      <div class="info-card-title">
        <HiOutlineClipboardDocument class="w-4 h-4" /> Container Details
      </div>
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-0.5">
        <InfoRows fields={fields()} />
      </div>
    </div>
  );
}
