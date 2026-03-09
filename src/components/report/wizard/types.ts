// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Wizard Types - Type definitions specific to the wizard component
 */

import type { DiscoveredFile, ContainerInfo } from "../../../types";
import type { ActivityLogEntry, ProjectSession, CachedCaseDocument, ProjectBookmark, ProjectNote } from "../../../types/project";
import type { ReportType } from "../types";

// =============================================================================
// WIZARD STEP TYPES
// =============================================================================

/** Available wizard steps */
export type WizardStep = "report_type" | "case" | "examiner" | "evidence" | "findings" | "report_data" | "preview" | "export";

/** Step configuration */
export interface WizardStepConfig {
  id: WizardStep;
  label: string;
  description?: string;
  /** Only show for certain report types (undefined = always show) */
  forReportTypes?: ReportType[];
  /** Exclude from certain report types (checked after forReportTypes) */
  excludeForReportTypes?: ReportType[];
}

/** Base wizard steps - always shown */
export const WIZARD_STEPS: WizardStepConfig[] = [
  { id: "report_type", label: "Report Type", description: "Select report type" },
  { id: "case", label: "Case Info", description: "Enter case details" },
  { id: "examiner", label: "Examiner", description: "Examiner / collecting officer" },
  { id: "evidence", label: "Evidence", description: "Select items", excludeForReportTypes: ["chain_of_custody"] },
  { id: "report_data", label: "Report Data", description: "Enter report details", forReportTypes: ["chain_of_custody", "investigative_activity", "user_activity", "timeline"] },
  { id: "findings", label: "Findings", description: "Document discoveries", forReportTypes: ["forensic_examination"] },
  { id: "preview", label: "Preview", description: "Review report" },
  { id: "export", label: "Export", description: "Generate output" },
];

/** Get steps filtered by report type */
export function getStepsForReportType(reportType: ReportType): WizardStepConfig[] {
  return WIZARD_STEPS.filter(step => {
    // If the step has an exclude list and this type is in it, skip
    if (step.excludeForReportTypes?.includes(reportType)) return false;
    // If the step has an include list, only show for those types
    if (step.forReportTypes) return step.forReportTypes.includes(reportType);
    // No restrictions — always show
    return true;
  });
}

// =============================================================================
// EVIDENCE GROUPING TYPES
// =============================================================================

/** Grouped evidence for multi-segment containers */
export interface EvidenceGroup {
  /** Primary file (first segment) */
  primaryFile: DiscoveredFile;
  /** All segment files */
  segments: DiscoveredFile[];
  /** Number of segments */
  segmentCount: number;
  /** Total size across all segments */
  totalSize: number;
  /** Base name without segment extension */
  baseName: string;
}

// =============================================================================
// SECTION VISIBILITY
// =============================================================================

/** Report section visibility settings */
export interface SectionVisibility {
  executiveSummary: boolean;
  scope: boolean;
  methodology: boolean;
  chainOfCustody: boolean;
  timeline: boolean;
  conclusions: boolean;
  appendices: boolean;
}

// =============================================================================
// WIZARD PROPS
// =============================================================================

/** Props for the main ReportWizard component */
export interface ReportWizardProps {
  /** Files discovered in the workspace */
  files: DiscoveredFile[];
  /** Map of file path to container info */
  fileInfoMap: Map<string, ContainerInfo>;
  /** Map of file path to hash info */
  fileHashMap: Map<string, { algorithm: string; hash: string; verified?: boolean | null }>;
  /** Project activity log entries (for auto-populating timeline) */
  activityLog?: ActivityLogEntry[];
  /** Project sessions (for auto-populating chain of custody) */
  sessions?: ProjectSession[];
  /** Project name (for auto-populating case name) */
  projectName?: string;
  /** Project description (for auto-populating case description) */
  projectDescription?: string;
  /** Project-level case number (pre-fills CaseInfo.case_number when set) */
  caseNumber?: string;
  /** Project-level case name (pre-fills CaseInfo.case_name when set) */
  caseName?: string;
  /** Cached case documents (for extracting case number) */
  caseDocumentsCache?: CachedCaseDocument[];
  /** Project bookmarks (for auto-populating findings) */
  bookmarks?: ProjectBookmark[];
  /** Project notes (for auto-populating findings) */
  notes?: ProjectNote[];
  /** Pre-selected report type (skips type selection step) */
  initialReportType?: ReportType;
  /** Called when wizard is closed */
  onClose: () => void;
  /** Called when report is generated */
  onGenerated?: (path: string, format: string) => void;
}

/** Props passed to step components from context */
export interface WizardStepProps {
  /** Whether this step is currently active */
  isActive?: boolean;
}

// =============================================================================
// AI ASSISTANT TYPES
// =============================================================================

/** AI generation state */
export interface AiGenerationState {
  generating: string | null;
  error: string | null;
  available: boolean;
}

/** AI settings configuration */
export interface AiSettings {
  provider: string;
  model: string;
  apiKey: string;
  ollamaConnected: boolean;
}
