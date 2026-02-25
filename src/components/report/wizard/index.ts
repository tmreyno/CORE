// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Report Wizard Module
 * 
 * This module provides a multi-step wizard for forensic report generation.
 * The wizard is split into separate components for maintainability:
 * 
 * - WizardContext: Shared state and context provider
 * - WizardLayout: Main layout with step navigation
 * - Steps: Individual step components (CaseInfo, Examiner, Evidence, Findings, Preview, Export)
 * - Hooks: AI assistant and other reusable logic
 * - Utils: Helper functions for evidence grouping, detection, etc.
 */

// Main wizard component
export { ReportWizard } from "./ReportWizard";

// Context and types
export { WizardProvider, useWizard, type WizardContextType } from "./WizardContext";
export { type WizardStep, WIZARD_STEPS } from "./types";

// Individual steps (for potential standalone use)
export { CaseInfoStep } from "./steps/CaseInfoStep";
export { CaseInfoSchemaStep } from "./steps/CaseInfoSchemaStep";
export { ExaminerStep } from "./steps/ExaminerStep";
export { ExaminerSchemaStep } from "./steps/ExaminerSchemaStep";
export { EvidenceStep } from "./steps/EvidenceStep";
export { FindingsStep } from "./steps/FindingsStep";
export { PreviewStep } from "./steps/PreviewStep";
export { ExportStep } from "./steps/ExportStep";

// Hooks
export { useAiAssistant } from "./hooks/useAiAssistant";

// Utils
export { groupEvidenceFiles, detectEvidenceType } from "./utils/evidenceUtils";
