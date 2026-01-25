// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ReportWizard - Multi-step wizard for forensic report generation
 * 
 * This is the main component that composes:
 * - WizardProvider for state management
 * - WizardLayout for navigation and step rendering
 * - Individual step components
 * 
 * Steps:
 * 1. Case Information - Enter case details
 * 2. Examiner - Enter examiner information
 * 3. Evidence Selection - Select which items to include
 * 4. Findings - Add/edit findings and narratives
 * 5. Preview - Review the report
 * 6. Export - Choose format and export
 */

import { onMount } from "solid-js";
import { makeEventListener } from "@solid-primitives/event-listener";
import { useFocusTrap } from "../../../hooks/useFocusTrap";
import type { ReportWizardProps } from "./types";
import { WizardProvider } from "./WizardContext";
import { WizardLayout } from "./WizardLayout";

// Re-export types for backward compatibility with old import paths
export type {
  Classification,
  Severity,
  EvidenceType,
  HashAlgorithmType,
  ReportMetadata,
  CaseInfo,
  ExaminerInfo,
  HashValue,
  EvidenceItem,
  Finding,
  CustodyRecord,
  SignatureRecord,
  OutputFormat,
  ForensicReport,
} from "../types";

export { EVIDENCE_TYPES } from "../constants";
export { REPORT_TEMPLATES, type ReportTemplateType, type ReportTemplate } from "../templates";

export function ReportWizard(props: ReportWizardProps) {
  // Focus trap for modal accessibility
  let modalRef: HTMLDivElement | undefined;
  useFocusTrap(() => modalRef, () => true);

  // Close on Escape key
  onMount(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        props.onClose();
      }
    };
    // makeEventListener auto-cleans up on component unmount
    makeEventListener(document, 'keydown', handleEscape);
  });

  return (
    <div class="modal-overlay">
      <div
        ref={modalRef}
        class="modal-content w-[900px] max-h-[85vh] flex flex-col"
        role="dialog"
        aria-modal="true"
        aria-labelledby="report-wizard-title"
      >
        <WizardProvider props={props}>
          <WizardLayout onClose={props.onClose} />
        </WizardProvider>
      </div>
    </div>
  );
}
