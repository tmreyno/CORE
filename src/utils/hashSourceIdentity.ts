// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { EvidenceSourceRef } from "../api/commands";

type HashSourceFields = {
  sourceId: string;
  sourceRefJson: string;
};

export function buildLocalFileHashSourceFields(
  filePath: string,
): HashSourceFields {
  const sourceRef: EvidenceSourceRef = {
    kind: "localFile",
    path: filePath,
  };

  return {
    sourceId: filePath,
    sourceRefJson: JSON.stringify(sourceRef),
  };
}
