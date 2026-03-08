// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

export { EvidenceCollectionPanel } from "./EvidenceCollectionPanel";
export type { EvidenceCollectionPanelProps, CollectionStatus } from "./types";
export { generateId, evidenceToFormData, formDataToEvidence } from "./formDataConversion";
export { buildLinkedDataTree } from "./linkedDataBuilder";
export {
  extractItemFieldsFromEvidence,
  extractHeaderFieldsFromEvidence,
  buildCollectedItemsFromEvidence,
  getAutoFillSummaries,
} from "./evidenceAutoFill";
export type { EvidenceFileSummary, HeaderAutoFillResult } from "./evidenceAutoFill";
