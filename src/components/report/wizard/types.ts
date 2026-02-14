// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Wizard Types - Type definitions specific to the wizard component
 */

import type { DiscoveredFile, ContainerInfo } from "../../../types";
import type { ActivityLogEntry, ProjectSession } from "../../../types/project";

// =============================================================================
// WIZARD STEP TYPES
// =============================================================================

/** Available wizard steps */
export type WizardStep = "case" | "examiner" | "evidence" | "findings" | "preview" | "export";

/** Step configuration */
export interface WizardStepConfig {
  id: WizardStep;
  label: string;
  description?: string;
}

/** Wizard step definitions */
export const WIZARD_STEPS: WizardStepConfig[] = [
  { id: "case", label: "Case Info", description: "Enter case details" },
  { id: "examiner", label: "Examiner", description: "Your information" },
  { id: "evidence", label: "Evidence", description: "Select items" },
  { id: "findings", label: "Findings", description: "Document discoveries" },
  { id: "preview", label: "Preview", description: "Review report" },
  { id: "export", label: "Export", description: "Generate output" },
];

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
