// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, createMemo, type Accessor, type Setter } from "solid-js";
import type { EvidenceGroup } from "../types";
import type { DiscoveredFile } from "../../../../types";
import { groupEvidenceFiles } from "../utils/evidenceUtils";

export interface EvidenceState {
  selectedEvidence: Accessor<Set<string>>;
  setSelectedEvidence: Setter<Set<string>>;
  groupedEvidence: Accessor<EvidenceGroup[]>;
  toggleEvidence: (path: string) => void;
}

export function useEvidenceState(files: DiscoveredFile[]): EvidenceState {
  const [selectedEvidence, setSelectedEvidence] = createSignal<Set<string>>(new Set());

  const groupedEvidence = createMemo(() => groupEvidenceFiles(files));

  const toggleEvidence = (path: string) => {
    const current = selectedEvidence();
    const next = new Set(current);
    if (next.has(path)) {
      next.delete(path);
    } else {
      next.add(path);
    }
    setSelectedEvidence(next);
  };

  return {
    selectedEvidence,
    setSelectedEvidence,
    groupedEvidence,
    toggleEvidence,
  };
}
