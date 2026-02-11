// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Document Type Configuration
 * 
 * Styling and icon configuration for different case document types.
 */

import {
  HiOutlineClipboardDocumentList,
  HiOutlineDocumentCheck,
  HiOutlineDocumentText,
  HiOutlineClipboard,
  HiOutlineMagnifyingGlass,
} from "../icons";
import type { CaseDocumentType } from "../../types";

export const documentTypeColors: Record<CaseDocumentType, string> = {
  ChainOfCustody: "text-accent bg-accent/10 border-accent/30",
  EvidenceIntake: "text-green-400 bg-green-500/10 border-green-500/30",
  CaseNotes: "text-yellow-400 bg-yellow-500/10 border-yellow-500/30",
  EvidenceReceipt: "text-purple-400 bg-purple-500/10 border-purple-500/30",
  LabRequest: "text-blue-400 bg-blue-500/10 border-blue-500/30",
  ExternalReport: "text-orange-400 bg-orange-500/10 border-orange-500/30",
  Other: "text-txt-secondary bg-bg-muted/10 border-border-subtle/30",
};

export const documentTypeIcons: Record<CaseDocumentType, typeof HiOutlineClipboard> = {
  ChainOfCustody: HiOutlineClipboardDocumentList,
  EvidenceIntake: HiOutlineDocumentCheck,
  CaseNotes: HiOutlineDocumentText,
  EvidenceReceipt: HiOutlineClipboard,
  LabRequest: HiOutlineMagnifyingGlass,
  ExternalReport: HiOutlineDocumentText,
  Other: HiOutlineDocumentText,
};
