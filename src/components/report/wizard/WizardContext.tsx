// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * WizardContext - Shared state management for the Report Wizard
 * 
 * Provides a context for all wizard state including:
 * - Navigation between steps
 * - Case, examiner, and evidence data
 * - Findings and chain of custody
 * - AI assistant integration
 * - Export configuration
 */

import { createContext, useContext, createSignal, createMemo, createEffect, on, onMount, type JSX, type Accessor, type Setter } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import type {
  ReportMetadata,
  CaseInfo,
  ExaminerInfo,
  Finding,
  CustodyRecord,
  TimelineEvent,
  OutputFormat,
  ForensicReport,
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
import { buildForensicReport } from "./utils/reportBuilder";
import { dbSync } from "../../../hooks/project/useProjectDbSync";
import {
  persistCocItemsToDb,
  loadCocItemsFromDb,
} from "./cocDbSync";
import { generateId, nowISO } from "../../../types/project";
import type { ProjectReportRecord } from "../../../types/project";
import { logger } from "../../../utils/logger";
const log = logger.scope("ReportWizard");

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

  const goToStep = (step: WizardStep) => setCurrentStep(step);

  const nextStep = () => {
    const steps = activeSteps();
    const idx = steps.findIndex((s) => s.id === currentStep());
    if (idx < steps.length - 1) {
      setCurrentStep(steps[idx + 1].id);
    }
  };

  const prevStep = () => {
    const steps = activeSteps();
    const idx = steps.findIndex((s) => s.id === currentStep());
    if (idx > 0) {
      setCurrentStep(steps[idx - 1].id);
    }
  };

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

  const applyPreset = (presetId: ReportPreset) => {
    setSelectedPreset(presetId);
    const preset = REPORT_PRESETS.find((p) => p.id === presetId);
    if (preset) {
      setMetadata((prev) => ({ ...prev, classification: preset.defaultClassification }));
      setEnabledSections(preset.defaultSections);
    }
  };

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
  // PROJECT DATA SEEDING
  // ==========================================================================

  onMount(() => {
    // --- Case Info: seed from project data ---
    const caseUpdates: Partial<CaseInfo> = {};

    // Project name → case_name (if not already set)
    if (props.projectName && !caseInfo().case_name) {
      caseUpdates.case_name = props.projectName;
    }

    // Project description → case description
    if (props.projectDescription && !caseInfo().description) {
      caseUpdates.description = props.projectDescription;
    }

    // Extract case number from case documents cache (first match wins)
    if (!caseInfo().case_number || caseInfo().case_number === (getPreference("caseNumberPrefix") || "")) {
      const caseDocs = props.caseDocumentsCache;
      if (caseDocs && caseDocs.length > 0) {
        const extracted = caseDocs.find(d => d.case_number)?.case_number;
        if (extracted) {
          caseUpdates.case_number = extracted;
        }
      }
    }

    // Agency from preferences
    const prefAgency = getPreference("defaultAgency");
    if (prefAgency && !caseInfo().agency) {
      caseUpdates.agency = prefAgency;
    }

    // Exam dates from sessions (earliest start, latest end)
    const sessions = props.sessions;
    if (sessions && sessions.length > 0) {
      if (!caseInfo().exam_start_date) {
        const earliest = sessions
          .map(s => s.started_at)
          .filter(Boolean)
          .sort()[0];
        if (earliest) {
          caseUpdates.exam_start_date = earliest.split("T")[0]; // date-only
        }
      }
      if (!caseInfo().exam_end_date) {
        const latestEnd = sessions
          .map(s => s.ended_at)
          .filter((d): d is string => !!d)
          .sort()
          .pop();
        if (latestEnd) {
          caseUpdates.exam_end_date = latestEnd.split("T")[0];
        }
      }
    }

    if (Object.keys(caseUpdates).length > 0) {
      setCaseInfo(prev => ({ ...prev, ...caseUpdates }));
    }

    // --- Metadata: seed generated_by from examiner name ---
    if (examiner().name) {
      setMetadata(prev => ({
        ...prev,
        generated_by: `${examiner().name} via FFX`,
      }));
    }

    // --- Evidence: auto-select all items ---
    if (props.files.length > 0 && selectedEvidence().size === 0) {
      const allPrimaries = groupedEvidence().map(g => g.primaryFile.path);
      setSelectedEvidence(new Set(allPrimaries));
    }

    // --- Chain of custody: seed from project sessions (skip for report types that don't use it) ---
    const rt = reportType();
    if (rt !== "chain_of_custody" &&
        sessions && sessions.length > 0 && chainOfCustody().length === 0) {
      const custodyFromSessions: CustodyRecord[] = sessions.map(session => ({
        timestamp: session.started_at,
        action: session.ended_at ? "Examination session" : "Active session",
        handler: session.user || examiner().name || "",
        location: session.hostname || undefined,
        notes: session.summary || 
          (session.duration_seconds 
            ? `Duration: ${Math.round(session.duration_seconds / 60)} min (${session.app_version})`
            : `App version: ${session.app_version}`),
      }));
      setChainOfCustody(custodyFromSessions);
    }

    // --- Findings: seed from bookmarks and notes ---
    if (findings().length === 0) {
      const autoFindings: Finding[] = [];

      // Bookmarks with notes become findings
      const bookmarks = props.bookmarks;
      if (bookmarks && bookmarks.length > 0) {
        for (const bm of bookmarks) {
          if (bm.notes || (bm.tags && bm.tags.length > 0)) {
            autoFindings.push({
              id: `finding-bm-${bm.id}`,
              title: bm.name,
              severity: "Informational",
              category: bm.tags?.[0] || "Bookmark",
              description: bm.notes || `Bookmarked: ${bm.name}`,
              artifact_paths: [bm.target_path],
              timestamps: [bm.created_at],
              evidence_refs: [],
              analysis: "",
            });
          }
        }
      }

      // Notes with high/critical priority become findings
      const notes = props.notes;
      if (notes && notes.length > 0) {
        for (const note of notes) {
          if (note.priority === "high" || note.priority === "critical") {
            autoFindings.push({
              id: `finding-note-${note.id}`,
              title: note.title,
              severity: note.priority === "critical" ? "Critical" : "High",
              category: note.tags?.[0] || "Note",
              description: note.content,
              artifact_paths: note.target_path ? [note.target_path] : [],
              timestamps: [note.created_at],
              evidence_refs: [],
              analysis: "",
            });
          }
        }
      }

      if (autoFindings.length > 0) {
        setFindings(autoFindings);
      }
    }

    log.info("Project data seeding complete", {
      caseUpdates: Object.keys(caseUpdates),
      evidenceAutoSelected: selectedEvidence().size,
      custodyRecords: chainOfCustody().length,
      findingsSeeded: findings().length,
    });
  });

  // Build timeline events from activity log for report
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

  const generatePreview = async () => {
    setPreviewLoading(true);
    try {
      const report = buildReport();
      const html = await invoke<string>("preview_report", { report });
      setPreviewHtml(html);
    } catch (e) {
      log.error("Preview failed:", e);
      setPreviewHtml(`<div style="color: red; padding: 20px;">Preview failed: ${e}</div>`);
    } finally {
      setPreviewLoading(false);
    }
  };

  // ==========================================================================
  // EXPORT STATE
  // ==========================================================================

  const [outputFormats, setOutputFormats] = createSignal<OutputFormat[]>([]);
  const [selectedFormat, setSelectedFormat] = createSignal<string>("Pdf");
  const [exporting, setExporting] = createSignal(false);
  const [exportError, setExportError] = createSignal<string | null>(null);

  // Initialize output formats
  onMount(async () => {
    try {
      const formats = await invoke<OutputFormat[]>("get_output_formats");
      setOutputFormats(formats);
    } catch (e) {
      log.error("Failed to get output formats:", e);
      // Set defaults
      setOutputFormats([
        { format: "Pdf", name: "PDF", description: "Portable Document Format", extension: "pdf", supported: true },
        { format: "Html", name: "HTML", description: "Web page format", extension: "html", supported: true },
        { format: "Markdown", name: "Markdown", description: "Markdown text format", extension: "md", supported: true },
      ]);
    }
  });

  const exportReport = async () => {
    setExporting(true);
    setExportError(null);

    try {
      const report = buildReport();
      const format = outputFormats().find((f) => f.format === selectedFormat());

      // Import save dialog dynamically
      const { save } = await import("@tauri-apps/plugin-dialog");

      // Open save dialog
      const path = await save({
        title: "Save Report",
        defaultPath: `${report.metadata.report_number}.${format?.extension || "pdf"}`,
        filters: format ? [{ name: format.name, extensions: [format.extension] }] : [],
      });

      if (!path) {
        setExporting(false);
        return;
      }

      // Generate report
      const outputPath = await invoke<string>("generate_report", {
        report,
        format: selectedFormat(),
        outputPath: path,
      });

      // Write-through to .ffxdb for report history tracking
      const reportRecord: ProjectReportRecord = {
        id: generateId(),
        title: report.metadata.title || "Forensic Report",
        report_type: "detailed",
        format: selectedFormat().toLowerCase() as ProjectReportRecord["format"],
        output_path: outputPath,
        generated_at: nowISO(),
        generated_by: report.examiner?.name || "unknown",
        status: "completed",
      };
      dbSync.insertReport(reportRecord);

      // Persist COC data to .ffxdb
      const rt = reportType();
      if (rt === "chain_of_custody" && cocItems().length > 0) {
        persistCocItemsToDb(cocItems());
      }

      props.onGenerated?.(outputPath, selectedFormat());
      props.onClose();
    } catch (e) {
      log.error("Export failed:", e);
      setExportError(String(e));

      // Record failed export in .ffxdb for audit trail
      const failedRecord: ProjectReportRecord = {
        id: generateId(),
        title: metadata().title || "Forensic Report",
        report_type: "detailed",
        format: selectedFormat().toLowerCase() as ProjectReportRecord["format"],
        generated_at: nowISO(),
        generated_by: examiner().name || "unknown",
        status: "failed",
        error: String(e),
      };
      dbSync.insertReport(failedRecord);
    } finally {
      setExporting(false);
    }
  };

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
  // LOAD COC FROM DB ON MOUNT
  // ==========================================================================

  onMount(async () => {
    const caseNum = caseInfo().case_number || undefined;
    try {
      const dbCocItems = await loadCocItemsFromDb(caseNum);
      if (dbCocItems.length > 0) {
        setCocItems(dbCocItems);
        log.info(`Loaded ${dbCocItems.length} COC items from .ffxdb`);
      }
    } catch (e) {
      log.warn("Could not load COC items from DB:", e);
    }
  });

  // ==========================================================================
  // REPORT BUILDING
  // ==========================================================================

  const buildReport = (): ForensicReport => {
    const base = buildForensicReport({
      metadata,
      caseInfo,
      examiner,
      executiveSummary,
      scope,
      methodology,
      conclusions,
      findings,
      chainOfCustody,
      groupedEvidence,
      selectedEvidence,
      examinerSignature,
      examinerSignedDate,
      supervisorName,
      supervisorSignature,
      supervisorSignedDate,
      digitalSignatureConfirmed,
      approvalNotes,
      fileInfoMap: props.fileInfoMap,
      fileHashMap: props.fileHashMap,
      projectTimeline,
    });

    // Attach report type and type-specific data
    base.report_type = reportType();
    const rt = reportType();
    if (rt === "chain_of_custody") {
      base.coc_items = cocItems();
    } else if (rt === "investigative_activity") {
      base.iar_data = { summary: iarSummary(), entries: iarEntries() };
    } else if (rt === "user_activity") {
      base.user_activity = userActivityData();
    } else if (rt === "timeline") {
      base.timeline_report = timelineReportData();
    }

    return base;
  };

  // ==========================================================================
  // CONTEXT VALUE
  // ==========================================================================

  const contextValue: WizardContextType = {
    props,

    // Navigation
    currentStep,
    setCurrentStep,
    goToStep,
    nextStep,
    prevStep,
    canGoNext,
    canGoPrev,

    // Report Type
    reportType,
    setReportType,
    activeSteps,

    // Preset
    selectedPreset,
    setSelectedPreset,
    applyPreset,
    currentPreset,

    // Case Info
    caseInfo,
    setCaseInfo,

    // Examiner
    examiner,
    setExaminer,
    newCert,
    setNewCert,
    addCertification,
    removeCertification,

    // Metadata
    metadata,
    setMetadata,
    setTitleManuallyEdited,
    setReportNumberManuallyEdited,

    // Evidence
    selectedEvidence,
    setSelectedEvidence,
    groupedEvidence,
    toggleEvidence,

    // Chain of Custody
    chainOfCustody,
    setChainOfCustody,
    addCustodyRecord,
    updateCustodyRecord,
    removeCustodyRecord,

    // Findings
    findings,
    setFindings,
    addFinding,
    updateFinding,
    removeFinding,

    // Narratives
    executiveSummary,
    setExecutiveSummary,
    scope,
    setScope,
    methodology,
    setMethodology,
    conclusions,
    setConclusions,

    // Section visibility
    enabledSections,
    setEnabledSections,

    // Preview
    previewHtml,
    previewLoading,
    generatePreview,

    // Export
    outputFormats,
    selectedFormat,
    setSelectedFormat,
    exporting,
    exportError,
    exportReport,

    // Signatures
    examinerSignature,
    setExaminerSignature,
    examinerSignedDate,
    setExaminerSignedDate,
    supervisorName,
    setSupervisorName,
    supervisorSignature,
    setSupervisorSignature,
    supervisorSignedDate,
    setSupervisorSignedDate,
    digitalSignatureConfirmed,
    setDigitalSignatureConfirmed,
    approvalNotes,
    setApprovalNotes,

    // AI Assistant
    aiState,
    aiActions,

    // Report building
    buildReport,

    // Report-type-specific data
    cocItems,
    setCocItems,
    iarSummary,
    setIarSummary,
    iarEntries,
    setIarEntries,
    userActivityData,
    setUserActivityData,
    timelineReportData,
    setTimelineReportData,
  };

  return (
    <WizardContext.Provider value={contextValue}>
      {providerProps.children}
    </WizardContext.Provider>
  );
}
