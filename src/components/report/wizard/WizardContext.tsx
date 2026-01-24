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
  OutputFormat,
  ForensicReport,
  EvidenceItem,
} from "../types";
import { REPORT_TEMPLATES, type ReportTemplateType } from "../templates";
import { getPreference } from "../../preferences";
import type { WizardStep, SectionVisibility, EvidenceGroup, ReportWizardProps } from "./types";
import { WIZARD_STEPS } from "./types";
import { groupEvidenceFiles, detectEvidenceType } from "./utils/evidenceUtils";
import { useAiAssistant, type AiAssistantState, type AiAssistantActions } from "./hooks/useAiAssistant";

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

  const [examiner, setExaminer] = createSignal<ExaminerInfo>({
    name: getPreference("examinerName") || "",
    organization: getPreference("organizationName") || undefined,
    certifications: [],
  });

  const [newCert, setNewCert] = createSignal("");

  const addCertification = () => {
    const cert = newCert().trim();
    if (cert && !examiner().certifications.includes(cert)) {
      setExaminer({
        ...examiner(),
        certifications: [...examiner().certifications, cert],
      });
      setNewCert("");
    }
  };

  const removeCertification = (cert: string) => {
    setExaminer({
      ...examiner(),
      certifications: examiner().certifications.filter((c) => c !== cert),
    });
  };

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

  const [selectedEvidence, setSelectedEvidence] = createSignal<Set<string>>(new Set());

  const groupedEvidence = createMemo(() => groupEvidenceFiles(props.files));

  const toggleEvidence = (path: string) => {
    const current = selectedEvidence();
    const next = new Set(current);
    if (next.has(path)) {
      next.delete(path);
    } else {
      next.add(path);
    }
    setSelectedEvidence(next);
  };

  // ==========================================================================
  // CHAIN OF CUSTODY STATE
  // ==========================================================================

  const [chainOfCustody, setChainOfCustody] = createSignal<CustodyRecord[]>([]);

  const addCustodyRecord = () => {
    setChainOfCustody([
      ...chainOfCustody(),
      {
        timestamp: new Date().toISOString(),
        action: "Received",
        handler: examiner().name || "",
        location: "",
        notes: "",
      },
    ]);
  };

  const updateCustodyRecord = (index: number, updates: Partial<CustodyRecord>) => {
    setChainOfCustody((prev) =>
      prev.map((record, i) => (i === index ? { ...record, ...updates } : record))
    );
  };

  const removeCustodyRecord = (index: number) => {
    setChainOfCustody((prev) => prev.filter((_, i) => i !== index));
  };

  // ==========================================================================
  // FINDINGS STATE
  // ==========================================================================

  const [findings, setFindings] = createSignal<Finding[]>([]);

  const addFinding = () => {
    const newFinding: Finding = {
      id: `F${String(findings().length + 1).padStart(3, "0")}`,
      title: "",
      severity: "Medium",
      category: "General",
      description: "",
      artifact_paths: [],
      timestamps: [],
      evidence_refs: [],
      analysis: "",
    };
    setFindings([...findings(), newFinding]);
  };

  const updateFinding = (index: number, updates: Partial<Finding>) => {
    const current = findings();
    const updated = [...current];
    updated[index] = { ...updated[index], ...updates };
    setFindings(updated);
  };

  const removeFinding = (index: number) => {
    setFindings(findings().filter((_, i) => i !== index));
  };

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
      console.error("Preview failed:", e);
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
      console.error("Failed to get output formats:", e);
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

      props.onGenerated?.(outputPath, selectedFormat());
      props.onClose();
    } catch (e) {
      console.error("Export failed:", e);
      setExportError(String(e));
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
    // Get report preferences
    const includeHashes = getPreference("includeHashesInReports");
    const includeTimestamps = getPreference("includeTimestampsInReports");
    const includeMetadata = getPreference("includeMetadataInReports");
    
    // Build evidence items from selected files
    const evidenceItems: EvidenceItem[] = [];
    const groups = groupedEvidence();

    for (const group of groups) {
      if (!selectedEvidence().has(group.primaryFile.path)) continue;

      const info = props.fileInfoMap.get(group.primaryFile.path);
      const hashInfo = props.fileHashMap.get(group.primaryFile.path);

      const ewfInfo = info?.e01 || info?.l01;
      const ad1Info = info?.ad1;

      evidenceItems.push({
        evidence_id: `EV${String(evidenceItems.length + 1).padStart(3, "0")}`,
        description: group.primaryFile.filename,
        evidence_type: detectEvidenceType(group.primaryFile, info),
        // Only include metadata if preference enabled
        make: includeMetadata ? undefined : undefined, // EWF format doesn't have manufacturer in current schema
        model: includeMetadata ? (ewfInfo?.model ?? undefined) : undefined,
        serial_number: includeMetadata ? (ewfInfo?.serial_number ?? undefined) : undefined,
        capacity: includeMetadata && group.totalSize > 0 ? String(group.totalSize) : undefined,
        // Only include timestamps if preference enabled
        acquisition_date: includeTimestamps ? (ewfInfo?.acquiry_date ?? ad1Info?.companion_log?.acquisition_date ?? undefined) : undefined,
        acquisition_method: includeMetadata ? (ewfInfo?.description ?? undefined) : undefined,
        acquisition_tool: undefined, // Not available in current EWF schema
        // Only include hashes if preference enabled
        acquisition_hashes: includeHashes && hashInfo
          ? [
              {
                item: group.primaryFile.filename,
                algorithm: hashInfo.algorithm as any,
                value: hashInfo.hash,
                verified: hashInfo.verified ?? undefined,
              },
            ]
          : [],
        verification_hashes: [],
        notes: group.segmentCount > 1 ? `Multi-segment container with ${group.segmentCount} segments` : undefined,
      });
    }

    // Build signatures array
    const signatures: any[] = [];
    if (examinerSignature()) {
      signatures.push({
        role: "examiner",
        name: examiner().name,
        signature: examinerSignature(),
        signed_date: examinerSignedDate() || undefined,
        certified: digitalSignatureConfirmed(),
      });
    }
    if (supervisorName() || supervisorSignature()) {
      signatures.push({
        role: "supervisor",
        name: supervisorName(),
        signature: supervisorSignature() || undefined,
        signed_date: supervisorSignedDate() || undefined,
        notes: approvalNotes() || undefined,
      });
    }

    return {
      metadata: metadata(),
      case_info: caseInfo(),
      examiner: examiner(),
      executive_summary: executiveSummary() || undefined,
      scope: scope() || undefined,
      methodology: methodology() || undefined,
      evidence_items: evidenceItems,
      chain_of_custody: chainOfCustody(),
      findings: findings(),
      timeline: [],
      hash_records: [],
      tools: [
        {
          name: "FFX - Forensic File Xplorer",
          version: "1.0.0",
          vendor: "FFX Team",
          purpose: "Forensic image analysis and report generation",
        },
      ],
      conclusions: conclusions() || undefined,
      appendices: [],
      signatures: signatures.length > 0 ? signatures : undefined,
      notes: undefined,
    };
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
