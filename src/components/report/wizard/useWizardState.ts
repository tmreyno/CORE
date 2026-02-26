// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useWizardState — Reactive state signals and derived computations
 * for the Report Wizard.
 *
 * Creates all SolidJS signals, memos, and reactive effects.
 * Must be called within a component's reactive scope (WizardProvider).
 */

import { createSignal, createMemo, createEffect, on, type Accessor, type Setter } from "solid-js";
import type {
  ReportMetadata,
  CaseInfo,
  ExaminerInfo,
  Finding,
  CustodyRecord,
  TimelineEvent,
  OutputFormat,
  ReportType,
  COCItem,
  IAREntry,
  IARSummary,
  UserActivityData,
  TimelineReportData,
} from "../types";
import { REPORT_PRESETS, REPORT_TYPE_DEFAULTS, type ReportPreset, type ReportPresetConfig } from "../constants";
import { getPreference } from "../../preferences";
import { generateReportNumber } from "./utils/reportNumbering";
import type { WizardStep, SectionVisibility, EvidenceGroup, ReportWizardProps, WizardStepConfig } from "./types";
import { getStepsForReportType } from "./types";
import { useAiAssistant, type AiAssistantState, type AiAssistantActions } from "./hooks/useAiAssistant";
import { useExaminerState } from "./hooks/useExaminerState";
import { useEvidenceState } from "./hooks/useEvidenceState";
import { useCustodyState } from "./hooks/useCustodyState";
import { useFindingsState } from "./hooks/useFindingsState";

// =============================================================================
// STATE INTERFACE
// =============================================================================

/**
 * Internal state bag shared between useWizardState and useWizardActions.
 * Exposes all signal getters AND setters so actions can mutate state.
 */
export interface WizardState {
  // Report Type
  reportType: Accessor<ReportType>;
  setReportType: Setter<ReportType>;
  activeSteps: Accessor<WizardStepConfig[]>;

  // Navigation
  currentStep: Accessor<WizardStep>;
  setCurrentStep: Setter<WizardStep>;
  canGoNext: Accessor<boolean>;
  canGoPrev: Accessor<boolean>;

  // Preset
  selectedPreset: Accessor<ReportPreset>;
  setSelectedPreset: Setter<ReportPreset>;
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
  titleManuallyEdited: Accessor<boolean>;
  setTitleManuallyEdited: Setter<boolean>;
  reportNumberManuallyEdited: Accessor<boolean>;
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
  setPreviewHtml: Setter<string>;
  previewLoading: Accessor<boolean>;
  setPreviewLoading: Setter<boolean>;

  // Export
  outputFormats: Accessor<OutputFormat[]>;
  setOutputFormats: Setter<OutputFormat[]>;
  selectedFormat: Accessor<string>;
  setSelectedFormat: Setter<string>;
  exporting: Accessor<boolean>;
  setExporting: Setter<boolean>;
  exportError: Accessor<string | null>;
  setExportError: Setter<string | null>;

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

  // Derived
  projectTimeline: Accessor<TimelineEvent[]>;
}

// =============================================================================
// STATE CREATION
// =============================================================================

export function useWizardState(props: ReportWizardProps): WizardState {
  // ==========================================================================
  // REPORT TYPE STATE
  // ==========================================================================

  const [reportType, setReportType] = createSignal<ReportType>(
    props.initialReportType || "forensic_examination"
  );

  /** Steps filtered by the currently selected report type */
  const activeSteps = createMemo(() => getStepsForReportType(reportType()));

  // ==========================================================================
  // NAVIGATION STATE
  // ==========================================================================

  // If an initial report type was provided, skip the type selection step
  const [currentStep, setCurrentStep] = createSignal<WizardStep>(
    props.initialReportType ? "case" : "report_type"
  );

  const canGoNext = createMemo(() => {
    const steps = activeSteps();
    const idx = steps.findIndex((s) => s.id === currentStep());
    return idx < steps.length - 1;
  });

  const canGoPrev = createMemo(() => {
    const steps = activeSteps();
    const idx = steps.findIndex((s) => s.id === currentStep());
    return idx > 0;
  });

  // ==========================================================================
  // PRESET STATE - Initialize from preferences
  // ==========================================================================

  const defaultPreset = getPreference("defaultReportPreset") || "law_enforcement";
  const [selectedPreset, setSelectedPreset] = createSignal<ReportPreset>(defaultPreset as ReportPreset);

  const currentPreset = createMemo(() => REPORT_PRESETS.find((p) => p.id === selectedPreset()));

  // ==========================================================================
  // CASE INFO STATE - Initialize with prefix from preferences
  // ==========================================================================

  const [caseInfo, setCaseInfo] = createSignal<CaseInfo>({
    case_number: getPreference("caseNumberPrefix") || "",
  });

  // ==========================================================================
  // EXAMINER STATE - Initialize from preferences
  // ==========================================================================

  const {
    examiner,
    setExaminer,
    newCert,
    setNewCert,
    addCertification,
    removeCertification,
  } = useExaminerState();

  // ==========================================================================
  // METADATA STATE - Initialized from report type defaults + numbering prefs
  // ==========================================================================

  const initialType = reportType();
  const initialDefaults = REPORT_TYPE_DEFAULTS[initialType];

  const [metadata, setMetadata] = createSignal<ReportMetadata>({
    title: initialDefaults.title,
    report_number: generateReportNumber(initialType),
    version: "1.0",
    classification: initialDefaults.classification,
    generated_at: new Date().toISOString(),
    generated_by: "FFX - Forensic File Xplorer",
  });

  // Track whether the user has manually edited the title / report number
  // so we don't clobber their changes when report type changes
  const [titleManuallyEdited, setTitleManuallyEdited] = createSignal(false);
  const [reportNumberManuallyEdited, setReportNumberManuallyEdited] = createSignal(false);

  // When report type changes, auto-update title, classification, and
  // generate a new unique report number (unless user manually edited)
  createEffect(on(reportType, (rt, prevRt) => {
    if (prevRt === undefined) return; // skip initial
    const defaults = REPORT_TYPE_DEFAULTS[rt];
    setMetadata((prev) => ({
      ...prev,
      classification: defaults.classification,
      ...(titleManuallyEdited() ? {} : { title: defaults.title }),
      ...(reportNumberManuallyEdited() ? {} : { report_number: generateReportNumber(rt) }),
    }));
  }));

  // ==========================================================================
  // EVIDENCE STATE
  // ==========================================================================

  const {
    selectedEvidence,
    setSelectedEvidence,
    groupedEvidence,
    toggleEvidence,
  } = useEvidenceState(props.files);

  // ==========================================================================
  // CHAIN OF CUSTODY STATE
  // ==========================================================================

  const {
    chainOfCustody,
    setChainOfCustody,
    addCustodyRecord,
    updateCustodyRecord,
    removeCustodyRecord,
  } = useCustodyState(examiner);

  // ==========================================================================
  // FINDINGS STATE
  // ==========================================================================

  const {
    findings,
    setFindings,
    addFinding,
    updateFinding,
    removeFinding,
  } = useFindingsState();

  // ==========================================================================
  // PROJECT TIMELINE (DERIVED)
  // ==========================================================================

  const projectTimeline = createMemo((): TimelineEvent[] => {
    const activityLog = props.activityLog;
    if (!activityLog || activityLog.length === 0) return [];
    return activityLog.map(entry => ({
      timestamp: entry.timestamp,
      event_type: entry.category,
      description: `[${entry.action}] ${entry.description}`,
      source: entry.user || "system",
      evidence_ref: undefined,
      artifact_path: entry.file_path || undefined,
    }));
  });

  // ==========================================================================
  // NARRATIVE STATE
  // ==========================================================================

  const [executiveSummary, setExecutiveSummary] = createSignal("");
  const [scope, setScope] = createSignal("");
  const [methodology, setMethodology] = createSignal("");
  const [conclusions, setConclusions] = createSignal("");

  // ==========================================================================
  // SECTION VISIBILITY STATE
  // ==========================================================================

  const [enabledSections, setEnabledSections] = createSignal<SectionVisibility>({
    executiveSummary: true,
    scope: true,
    methodology: true,
    chainOfCustody: true,
    timeline: true,
    conclusions: true,
    appendices: true,
  });

  // ==========================================================================
  // PREVIEW STATE
  // ==========================================================================

  const [previewHtml, setPreviewHtml] = createSignal("");
  const [previewLoading, setPreviewLoading] = createSignal(false);

  // ==========================================================================
  // EXPORT STATE
  // ==========================================================================

  const [outputFormats, setOutputFormats] = createSignal<OutputFormat[]>([]);
  const [selectedFormat, setSelectedFormat] = createSignal<string>("Pdf");
  const [exporting, setExporting] = createSignal(false);
  const [exportError, setExportError] = createSignal<string | null>(null);

  // ==========================================================================
  // SIGNATURE STATE
  // ==========================================================================

  const [examinerSignature, setExaminerSignature] = createSignal<string>("");
  const [examinerSignedDate, setExaminerSignedDate] = createSignal<string>("");
  const [supervisorName, setSupervisorName] = createSignal<string>("");
  const [supervisorSignature, setSupervisorSignature] = createSignal<string>("");
  const [supervisorSignedDate, setSupervisorSignedDate] = createSignal<string>("");
  const [digitalSignatureConfirmed, setDigitalSignatureConfirmed] = createSignal(false);
  const [approvalNotes, setApprovalNotes] = createSignal<string>("");

  // ==========================================================================
  // AI ASSISTANT
  // ==========================================================================

  const [aiState, aiActions] = useAiAssistant();

  // ==========================================================================
  // REPORT-TYPE-SPECIFIC STATE
  // ==========================================================================

  const [cocItems, setCocItems] = createSignal<COCItem[]>([]);
  const [iarSummary, setIarSummary] = createSignal<IARSummary>({
    investigation_start: "",
    lead_examiner: "",
    synopsis: "",
    authorization: "",
    personnel_list: [],
  });
  const [iarEntries, setIarEntries] = createSignal<IAREntry[]>([]);
  const [userActivityData, setUserActivityData] = createSignal<UserActivityData>({
    target_user: "",
    activity_entries: [],
  });
  const [timelineReportData, setTimelineReportData] = createSignal<TimelineReportData>({
    included_categories: [],
    events: [],
    key_events: [],
  });

  // ==========================================================================
  // RETURN STATE
  // ==========================================================================

  return {
    // Report Type
    reportType, setReportType, activeSteps,

    // Navigation
    currentStep, setCurrentStep, canGoNext, canGoPrev,

    // Preset
    selectedPreset, setSelectedPreset, currentPreset,

    // Case Info
    caseInfo, setCaseInfo,

    // Examiner
    examiner, setExaminer, newCert, setNewCert,
    addCertification, removeCertification,

    // Metadata
    metadata, setMetadata,
    titleManuallyEdited, setTitleManuallyEdited,
    reportNumberManuallyEdited, setReportNumberManuallyEdited,

    // Evidence
    selectedEvidence, setSelectedEvidence, groupedEvidence, toggleEvidence,

    // Chain of Custody
    chainOfCustody, setChainOfCustody,
    addCustodyRecord, updateCustodyRecord, removeCustodyRecord,

    // Findings
    findings, setFindings,
    addFinding, updateFinding, removeFinding,

    // Narratives
    executiveSummary, setExecutiveSummary,
    scope, setScope,
    methodology, setMethodology,
    conclusions, setConclusions,

    // Section visibility
    enabledSections, setEnabledSections,

    // Preview
    previewHtml, setPreviewHtml,
    previewLoading, setPreviewLoading,

    // Export
    outputFormats, setOutputFormats,
    selectedFormat, setSelectedFormat,
    exporting, setExporting,
    exportError, setExportError,

    // Signatures
    examinerSignature, setExaminerSignature,
    examinerSignedDate, setExaminerSignedDate,
    supervisorName, setSupervisorName,
    supervisorSignature, setSupervisorSignature,
    supervisorSignedDate, setSupervisorSignedDate,
    digitalSignatureConfirmed, setDigitalSignatureConfirmed,
    approvalNotes, setApprovalNotes,

    // AI Assistant
    aiState, aiActions,

    // Report-type-specific data
    cocItems, setCocItems,
    iarSummary, setIarSummary,
    iarEntries, setIarEntries,
    userActivityData, setUserActivityData,
    timelineReportData, setTimelineReportData,

    // Derived
    projectTimeline,
  };
}
