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

import { createContext, useContext, createSignal, createMemo, onMount, type JSX, type Accessor, type Setter } from "solid-js";
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
} from "../types";
import { REPORT_TEMPLATES, type ReportTemplateType } from "../templates";
import { getPreference } from "../../preferences";
import type { WizardStep, SectionVisibility, EvidenceGroup, ReportWizardProps } from "./types";
import { WIZARD_STEPS } from "./types";
import { useAiAssistant, type AiAssistantState, type AiAssistantActions } from "./hooks/useAiAssistant";
import { useExaminerState } from "./hooks/useExaminerState";
import { useEvidenceState } from "./hooks/useEvidenceState";
import { useCustodyState } from "./hooks/useCustodyState";
import { useFindingsState } from "./hooks/useFindingsState";
import { buildForensicReport } from "./utils/reportBuilder";
import { dbSync } from "../../../hooks/project/useProjectDbSync";
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

  // Navigation
  currentStep: Accessor<WizardStep>;
  setCurrentStep: Setter<WizardStep>;
  goToStep: (step: WizardStep) => void;
  nextStep: () => void;
  prevStep: () => void;
  canGoNext: Accessor<boolean>;
  canGoPrev: Accessor<boolean>;

  // Template
  selectedTemplate: Accessor<ReportTemplateType>;
  setSelectedTemplate: Setter<ReportTemplateType>;
  showTemplateSelector: Accessor<boolean>;
  setShowTemplateSelector: Setter<boolean>;
  applyTemplate: (templateId: ReportTemplateType) => void;
  currentTemplate: Accessor<(typeof REPORT_TEMPLATES)[number] | undefined>;

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
  // NAVIGATION STATE
  // ==========================================================================

  const [currentStep, setCurrentStep] = createSignal<WizardStep>("case");

  const goToStep = (step: WizardStep) => setCurrentStep(step);

  const nextStep = () => {
    const idx = WIZARD_STEPS.findIndex((s) => s.id === currentStep());
    if (idx < WIZARD_STEPS.length - 1) {
      setCurrentStep(WIZARD_STEPS[idx + 1].id);
    }
  };

  const prevStep = () => {
    const idx = WIZARD_STEPS.findIndex((s) => s.id === currentStep());
    if (idx > 0) {
      setCurrentStep(WIZARD_STEPS[idx - 1].id);
    }
  };

  const canGoNext = createMemo(() => {
    const idx = WIZARD_STEPS.findIndex((s) => s.id === currentStep());
    return idx < WIZARD_STEPS.length - 1;
  });

  const canGoPrev = createMemo(() => {
    const idx = WIZARD_STEPS.findIndex((s) => s.id === currentStep());
    return idx > 0;
  });

  // ==========================================================================
  // TEMPLATE STATE - Initialize from preferences
  // ==========================================================================

  const defaultTemplate = getPreference("defaultReportTemplate") || "law_enforcement";
  const [selectedTemplate, setSelectedTemplate] = createSignal<ReportTemplateType>(defaultTemplate as ReportTemplateType);
  const [showTemplateSelector, setShowTemplateSelector] = createSignal(true);

  const currentTemplate = createMemo(() => REPORT_TEMPLATES.find((t) => t.id === selectedTemplate()));

  const applyTemplate = (templateId: ReportTemplateType) => {
    setSelectedTemplate(templateId);
    const template = REPORT_TEMPLATES.find((t) => t.id === templateId);
    if (template) {
      setMetadata((prev) => ({ ...prev, classification: template.defaultClassification }));
      setEnabledSections(template.defaultSections);
    }
    setShowTemplateSelector(false);
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
  // METADATA STATE
  // ==========================================================================

  const [metadata, setMetadata] = createSignal<ReportMetadata>({
    title: "Digital Forensic Examination Report",
    report_number: `FR-${new Date().getFullYear()}-${String(Math.floor(Math.random() * 10000)).padStart(4, "0")}`,
    version: "1.0",
    classification: "LawEnforcementSensitive",
    generated_at: new Date().toISOString(),
    generated_by: "FFX - Forensic File Xplorer",
  });

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

    // --- Chain of custody: seed from project sessions ---
    if (sessions && sessions.length > 0 && chainOfCustody().length === 0) {
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
  // REPORT BUILDING
  // ==========================================================================

  const buildReport = (): ForensicReport => {
    return buildForensicReport({
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

    // Template
    selectedTemplate,
    setSelectedTemplate,
    showTemplateSelector,
    setShowTemplateSelector,
    applyTemplate,
    currentTemplate,

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
  };

  return (
    <WizardContext.Provider value={contextValue}>
      {providerProps.children}
    </WizardContext.Provider>
  );
}
