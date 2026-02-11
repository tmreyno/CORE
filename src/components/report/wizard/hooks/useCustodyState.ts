// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, type Accessor, type Setter } from "solid-js";
import type { CustodyRecord, ExaminerInfo } from "../../types";

export interface CustodyState {
  chainOfCustody: Accessor<CustodyRecord[]>;
  setChainOfCustody: Setter<CustodyRecord[]>;
  addCustodyRecord: () => void;
  updateCustodyRecord: (index: number, updates: Partial<CustodyRecord>) => void;
  removeCustodyRecord: (index: number) => void;
}

export function useCustodyState(examinerGetter: Accessor<ExaminerInfo>): CustodyState {
  const [chainOfCustody, setChainOfCustody] = createSignal<CustodyRecord[]>([]);

  const addCustodyRecord = () => {
    setChainOfCustody([
      ...chainOfCustody(),
      {
        timestamp: new Date().toISOString(),
        action: "Received",
        handler: examinerGetter().name || "",
        location: "",
        notes: "",
      },
    ]);
  };

  const updateCustodyRecord = (index: number, updates: Partial<CustodyRecord>) => {
    setChainOfCustody((prev) =>
      prev.map((record, i) => (i === index ? { ...record, ...updates } : record))
    );
  };

  const removeCustodyRecord = (index: number) => {
    setChainOfCustody((prev) => prev.filter((_, i) => i !== index));
  };

  return {
    chainOfCustody,
    setChainOfCustody,
    addCustodyRecord,
    updateCustodyRecord,
    removeCustodyRecord,
  };
}
