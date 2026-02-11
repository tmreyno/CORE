// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, type Accessor, type Setter } from "solid-js";
import type { Finding } from "../../types";

export interface FindingsState {
  findings: Accessor<Finding[]>;
  setFindings: Setter<Finding[]>;
  addFinding: () => void;
  updateFinding: (index: number, updates: Partial<Finding>) => void;
  removeFinding: (index: number) => void;
}

export function useFindingsState(): FindingsState {
  const [findings, setFindings] = createSignal<Finding[]>([]);

  const addFinding = () => {
    const newFinding: Finding = {
      id: `F${String(findings().length + 1).padStart(3, "0")}`,
      title: "",
      severity: "Medium",
      category: "General",
      description: "",
      artifact_paths: [],
      timestamps: [],
      evidence_refs: [],
      analysis: "",
    };
    setFindings([...findings(), newFinding]);
  };

  const updateFinding = (index: number, updates: Partial<Finding>) => {
    const current = findings();
    const updated = [...current];
    updated[index] = { ...updated[index], ...updates };
    setFindings(updated);
  };

  const removeFinding = (index: number) => {
    setFindings(findings().filter((_, i) => i !== index));
  };

  return {
    findings,
    setFindings,
    addFinding,
    updateFinding,
    removeFinding,
  };
}
