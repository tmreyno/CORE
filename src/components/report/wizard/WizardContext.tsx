// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * WizardContext — Context provider for the Report Wizard.
 *
 * Composes useWizardState (signals/memos) and useWizardActions (actions)
 * into a single context value consumed by all wizard sub-components.
 */

import { createContext, useContext, type JSX, type Accessor, type Setter } from "solid-js";
import type {
  ReportMetadata,
  CaseInfo,
  ExaminerInfo,
  Finding,
  CustodyRecord,
  OutputFormat,
  ForensicReport,
  ReportType,
  COCItem,
  IAREntry,
  IARSummary,
  UserActivityData,
  TimelineReportData,
} from "../types";
import type { ReportPreset, ReportPresetConfig } from "../constants";
import type { WizardStep, SectionVisibility, EvidenceGroup, ReportWizardProps, WizardStepConfig } from "./types";
import type { AiAssistantState, AiAssistantActions } from "./hooks/useAiAssistant";
import { useWizardState } from "./useWizardState";
import { useWizardActions } from "./useWizardActions";

// =============================================================================
// CONTEXT TYPE
// =============================================================================

export interface WizardContextType {
  // Props
  props: ReportWizardProps;

  // Report Type
  reportType: Accessor<ReportType>;
  setReportType: Setter<ReportType>;
  activeSteps: Accessor<WizardStepConfig[]>;

  // Navigation
  currentStep: Accessor<WizardStep>;
  setCurrentStep: Setter<WizardStep>;
  goToStep: (step: WizardStep) => void;
  nextStep: () => void;
  prevStep: () => void;
  canGoNext: Accessor<boolean>;
  canGoPrev: Accessor<boolean>;

  // Preset
  selectedPreset: Accessor<ReportPreset>;
  setSelectedPreset: Setter<ReportPreset>;
  applyPreset: (presetId: ReportPreset) => void;
  currentPreset: Accessor<ReportPresetConfig | undefined>;

  // Case Info
  caseInfo: Accessor<CaseInfo>;
  setCaseInfo: Setter<CaseInfo>;

  // Examiner
  examiner: Accessor<ExaminerInfo>;
  setExaminer: Setter<ExaminerInfo>;
  newCert: Accessor<string>;
  setNewCert: Setter<string>;
  addCertification: () => void;
  removeCertification: (cert: string) => void;

  // Metadata
  metadata: Accessor<ReportMetadata>;
  setMetadata: Setter<ReportMetadata>;
  /** Mark that the user has manually edited the report title */
  setTitleManuallyEdited: Setter<boolean>;
  /** Mark that the user has manually edited the report number */
  setReportNumberManuallyEdited: Setter<boolean>;

  // Evidence
  selectedEvidence: Accessor<Set<string>>;
  setSelectedEvidence: Setter<Set<string>>;
  groupedEvidence: Accessor<EvidenceGroup[]>;
  toggleEvidence: (path: string) => void;

  // Chain of Custody
  chainOfCustody: Accessor<CustodyRecord[]>;
  setChainOfCustody: Setter<CustodyRecord[]>;
  addCustodyRecord: () => void;
  updateCustodyRecord: (index: number, updates: Partial<CustodyRecord>) => void;
  removeCustodyRecord: (index: number) => void;

  // Findings
  findings: Accessor<Finding[]>;
  setFindings: Setter<Finding[]>;
  addFinding: () => void;
  updateFinding: (index: number, updates: Partial<Finding>) => void;
  removeFinding: (index: number) => void;

  // Narratives
  executiveSummary: Accessor<string>;
  setExecutiveSummary: Setter<string>;
  scope: Accessor<string>;
  setScope: Setter<string>;
  methodology: Accessor<string>;
  setMethodology: Setter<string>;
  conclusions: Accessor<string>;
  setConclusions: Setter<string>;

  // Section visibility
  enabledSections: Accessor<SectionVisibility>;
  setEnabledSections: Setter<SectionVisibility>;

  // Preview
  previewHtml: Accessor<string>;
  previewLoading: Accessor<boolean>;
  generatePreview: () => Promise<void>;

  // Export
  outputFormats: Accessor<OutputFormat[]>;
  selectedFormat: Accessor<string>;
  setSelectedFormat: Setter<string>;
  exporting: Accessor<boolean>;
  exportError: Accessor<string | null>;
  exportReport: () => Promise<void>;

  // Signatures
  examinerSignature: Accessor<string>;
  setExaminerSignature: Setter<string>;
  examinerSignedDate: Accessor<string>;
  setExaminerSignedDate: Setter<string>;
  supervisorName: Accessor<string>;
  setSupervisorName: Setter<string>;
  supervisorSignature: Accessor<string>;
  setSupervisorSignature: Setter<string>;
  supervisorSignedDate: Accessor<string>;
  setSupervisorSignedDate: Setter<string>;
  digitalSignatureConfirmed: Accessor<boolean>;
  setDigitalSignatureConfirmed: Setter<boolean>;
  approvalNotes: Accessor<string>;
  setApprovalNotes: Setter<string>;

  // AI Assistant
  aiState: Accessor<AiAssistantState>;
  aiActions: AiAssistantActions;

  // Report-type-specific data
  cocItems: Accessor<COCItem[]>;
  setCocItems: Setter<COCItem[]>;
  iarSummary: Accessor<IARSummary>;
  setIarSummary: Setter<IARSummary>;
  iarEntries: Accessor<IAREntry[]>;
  setIarEntries: Setter<IAREntry[]>;
  userActivityData: Accessor<UserActivityData>;
  setUserActivityData: Setter<UserActivityData>;
  timelineReportData: Accessor<TimelineReportData>;
  setTimelineReportData: Setter<TimelineReportData>;

  // Report building
  buildReport: () => ForensicReport;
}

// =============================================================================
// CONTEXT
// =============================================================================

const WizardContext = createContext<WizardContextType>();

export function useWizard(): WizardContextType {
  const context = useContext(WizardContext);
  if (!context) {
    throw new Error("useWizard must be used within a WizardProvider");
  }
  return context;
}

// =============================================================================
// PROVIDER
// =============================================================================

interface WizardProviderProps {
  props: ReportWizardProps;
  children: JSX.Element;
}

export function WizardProvider(providerProps: WizardProviderProps) {
  const { props } = providerProps;

  // Create all reactive state
  const state = useWizardState(props);

  // Create all action functions (also registers lifecycle onMount hooks)
  const actions = useWizardActions(state, props);

  // ==========================================================================
  // CONTEXT VALUE
  // ==========================================================================

  const contextValue: WizardContextType = {
    props,

    // Navigation
    currentStep: state.currentStep,
    setCurrentStep: state.setCurrentStep,
    goToStep: actions.goToStep,
    nextStep: actions.nextStep,
    prevStep: actions.prevStep,
    canGoNext: state.canGoNext,
    canGoPrev: state.canGoPrev,

    // Report Type
    reportType: state.reportType,
    setReportType: state.setReportType,
    activeSteps: state.activeSteps,

    // Preset
    selectedPreset: state.selectedPreset,
    setSelectedPreset: state.setSelectedPreset,
    applyPreset: actions.applyPreset,
    currentPreset: state.currentPreset,

    // Case Info
    caseInfo: state.caseInfo,
    setCaseInfo: state.setCaseInfo,

    // Examiner
    examiner: state.examiner,
    setExaminer: state.setExaminer,
    newCert: state.newCert,
    setNewCert: state.setNewCert,
    addCertification: state.addCertification,
    removeCertification: state.removeCertification,

    // Metadata
    metadata: state.metadata,
    setMetadata: state.setMetadata,
    setTitleManuallyEdited: state.setTitleManuallyEdited,
    setReportNumberManuallyEdited: state.setReportNumberManuallyEdited,

    // Evidence
    selectedEvidence: state.selectedEvidence,
    setSelectedEvidence: state.setSelectedEvidence,
    groupedEvidence: state.groupedEvidence,
    toggleEvidence: state.toggleEvidence,

    // Chain of Custody
    chainOfCustody: state.chainOfCustody,
    setChainOfCustody: state.setChainOfCustody,
    addCustodyRecord: state.addCustodyRecord,
    updateCustodyRecord: state.updateCustodyRecord,
    removeCustodyRecord: state.removeCustodyRecord,

    // Findings
    findings: state.findings,
    setFindings: state.setFindings,
    addFinding: state.addFinding,
    updateFinding: state.updateFinding,
    removeFinding: state.removeFinding,

    // Narratives
    executiveSummary: state.executiveSummary,
    setExecutiveSummary: state.setExecutiveSummary,
    scope: state.scope,
    setScope: state.setScope,
    methodology: state.methodology,
    setMethodology: state.setMethodology,
    conclusions: state.conclusions,
    setConclusions: state.setConclusions,

    // Section visibility
    enabledSections: state.enabledSections,
    setEnabledSections: state.setEnabledSections,

    // Preview
    previewHtml: state.previewHtml,
    previewLoading: state.previewLoading,
    generatePreview: actions.generatePreview,

    // Export
    outputFormats: state.outputFormats,
    selectedFormat: state.selectedFormat,
    setSelectedFormat: state.setSelectedFormat,
    exporting: state.exporting,
    exportError: state.exportError,
    exportReport: actions.exportReport,

    // Signatures
    examinerSignature: state.examinerSignature,
    setExaminerSignature: state.setExaminerSignature,
    examinerSignedDate: state.examinerSignedDate,
    setExaminerSignedDate: state.setExaminerSignedDate,
    supervisorName: state.supervisorName,
    setSupervisorName: state.setSupervisorName,
    supervisorSignature: state.supervisorSignature,
    setSupervisorSignature: state.setSupervisorSignature,
    supervisorSignedDate: state.supervisorSignedDate,
    setSupervisorSignedDate: state.setSupervisorSignedDate,
    digitalSignatureConfirmed: state.digitalSignatureConfirmed,
    setDigitalSignatureConfirmed: state.setDigitalSignatureConfirmed,
    approvalNotes: state.approvalNotes,
    setApprovalNotes: state.setApprovalNotes,

    // AI Assistant
    aiState: state.aiState,
    aiActions: state.aiActions,

    // Report building
    buildReport: actions.buildReport,

    // Report-type-specific data
    cocItems: state.cocItems,
    setCocItems: state.setCocItems,
    iarSummary: state.iarSummary,
    setIarSummary: state.setIarSummary,
    iarEntries: state.iarEntries,
    setIarEntries: state.setIarEntries,
    userActivityData: state.userActivityData,
    setUserActivityData: state.setUserActivityData,
    timelineReportData: state.timelineReportData,
    setTimelineReportData: state.setTimelineReportData,
  };

  return (
    <WizardContext.Provider value={contextValue}>
      {providerProps.children}
    </WizardContext.Provider>
  );
}
