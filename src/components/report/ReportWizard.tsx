// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ReportWizard - Multi-step wizard for forensic report generation
 * 
 * Steps:
 * 1. Case Information - Enter case details
 * 2. Evidence Selection - Select which items to include
 * 3. Findings - Add/edit findings
 * 4. Preview - Review the report
 * 5. Export - Choose format and export
 */

import { createSignal, For, Show, createEffect, onMount, onCleanup, createMemo } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import DOMPurify from "dompurify";
import { formatBytes } from "../../utils";
import {
  HiOutlineClipboardDocument,
  HiOutlineUser,
  HiOutlineCircleStack,
  HiOutlineArrowUpTray,
  HiOutlineCpuChip,
  HiOutlineExclamationTriangle,
  HiOutlineArrowPath,
  HiOutlineDocument,
  HiOutlineDocumentText,
  HiOutlineDocumentCheck,
  HiOutlineCheckCircle,
  HiOutlineXCircle,
  HiOutlineUserGroup,
  HiOutlineXMark,
  HiOutlineServer,
  HiOutlineCalendarDays,
} from "../icons";
import type { DiscoveredFile, ContainerInfo } from "../../types";
import { useFocusTrap } from "../../hooks/useFocusTrap";
import { 
  isAiAvailable, 
  getAiProviders, 
  checkOllamaConnection, 
  generateAiNarrative,
  buildEvidenceContext,
  type AiProviderInfo,
  type NarrativeType 
} from "../../report/api";

// Import types, constants, and templates from separate modules
import type {
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
} from "./types";

import {
  CLASSIFICATIONS,
  SEVERITIES,
  EVIDENCE_TYPES,
} from "./constants";

import {
  REPORT_TEMPLATES,
  type ReportTemplateType,
  type ReportTemplate,
} from "./templates";

// Re-export types for backward compatibility
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
  ReportTemplateType,
  ReportTemplate,
};

// Re-export constants for backward compatibility
export { EVIDENCE_TYPES, REPORT_TEMPLATES };

interface ReportWizardProps {
  /** Files discovered in the workspace */
  files: DiscoveredFile[];
  /** Map of file path to container info */
  fileInfoMap: Map<string, ContainerInfo>;
  /** Map of file path to hash info */
  fileHashMap: Map<string, { algorithm: string; hash: string; verified?: boolean | null }>;
  /** Called when wizard is closed */
  onClose: () => void;
  /** Called when report is generated */
  onGenerated?: (path: string, format: string) => void;
}

// Wizard steps
type WizardStep = "case" | "examiner" | "evidence" | "findings" | "preview" | "export";

const STEPS: { id: WizardStep; label: string }[] = [
  { id: "case", label: "Case Info" },
  { id: "examiner", label: "Examiner" },
  { id: "evidence", label: "Evidence" },
  { id: "findings", label: "Findings" },
  { id: "preview", label: "Preview" },
  { id: "export", label: "Export" },
];

export function ReportWizard(props: ReportWizardProps) {
  // Focus trap for modal accessibility
  let modalRef: HTMLDivElement | undefined;
  useFocusTrap(() => modalRef, () => true);
  
  // Current step
  const [currentStep, setCurrentStep] = createSignal<WizardStep>("case");
  
  // Template selection
  const [selectedTemplate, setSelectedTemplate] = createSignal<ReportTemplateType>("law_enforcement");
  const [showTemplateSelector, setShowTemplateSelector] = createSignal(true);
  
  // Report data
  const [caseInfo, setCaseInfo] = createSignal<CaseInfo>({
    case_number: "",
  });
  
  const [examiner, setExaminer] = createSignal<ExaminerInfo>({
    name: "",
    certifications: [],
  });
  
  const [metadata, setMetadata] = createSignal<ReportMetadata>({
    title: "Digital Forensic Examination Report",
    report_number: `FR-${new Date().getFullYear()}-${String(Math.floor(Math.random() * 10000)).padStart(4, '0')}`,
    version: "1.0",
    classification: "LawEnforcementSensitive",
    generated_at: new Date().toISOString(),
    generated_by: "FFX - Forensic File Xplorer",
  });
  
  const [selectedEvidence, setSelectedEvidence] = createSignal<Set<string>>(new Set());
  const [findings, setFindings] = createSignal<Finding[]>([]);
  const [executiveSummary, setExecutiveSummary] = createSignal("");
  const [scope, setScope] = createSignal("");
  const [methodology, setMethodology] = createSignal("");
  const [conclusions, setConclusions] = createSignal("");
  
  // Chain of Custody state
  const [chainOfCustody, setChainOfCustody] = createSignal<CustodyRecord[]>([]);
  
  // Section visibility (from template)
  const [enabledSections, setEnabledSections] = createSignal({
    executiveSummary: true,
    scope: true,
    methodology: true,
    chainOfCustody: true,
    timeline: true,
    conclusions: true,
    appendices: true,
  });
  
  // Preview HTML
  const [previewHtml, setPreviewHtml] = createSignal("");
  const [previewLoading, setPreviewLoading] = createSignal(false);
  
  // Export state
  const [outputFormats, setOutputFormats] = createSignal<OutputFormat[]>([]);
  const [selectedFormat, setSelectedFormat] = createSignal<string>("Pdf");
  const [exporting, setExporting] = createSignal(false);
  const [exportError, setExportError] = createSignal<string | null>(null);
  
  // Signature/Approval state
  const [examinerSignature, setExaminerSignature] = createSignal<string>("");
  const [examinerSignedDate, setExaminerSignedDate] = createSignal<string>("");
  const [supervisorName, setSupervisorName] = createSignal<string>("");
  const [supervisorSignature, setSupervisorSignature] = createSignal<string>("");
  const [supervisorSignedDate, setSupervisorSignedDate] = createSignal<string>("");
  const [digitalSignatureConfirmed, setDigitalSignatureConfirmed] = createSignal(false);
  const [approvalNotes, setApprovalNotes] = createSignal<string>("");
  
  // New certification input
  const [newCert, setNewCert] = createSignal("");
  
  // AI Assistant state
  const [aiAvailable, setAiAvailable] = createSignal(false);
  const [aiProviders, setAiProviders] = createSignal<AiProviderInfo[]>([]);
  const [selectedProvider, setSelectedProvider] = createSignal<string>("ollama");
  const [selectedModel, setSelectedModel] = createSignal<string>("llama3.2");
  const [apiKey, setApiKey] = createSignal<string>("");
  const [ollamaConnected, setOllamaConnected] = createSignal(false);
  const [aiGenerating, setAiGenerating] = createSignal<string | null>(null); // Which section is generating
  const [aiError, setAiError] = createSignal<string | null>(null);
  const [showAiSettings, setShowAiSettings] = createSignal(false);
  
  // Auto-populate case info from container metadata
  const autoPopulateCaseInfo = () => {
    for (const file of props.files) {
      const info = props.fileInfoMap.get(file.path);
      if (!info) continue;
      
      const ewfInfo = info.e01 || info.l01;
      const ad1Info = info.ad1;
      const ufedInfo = info.ufed;
      
      // Extract case info from first container that has it
      const extractedCaseNumber = ewfInfo?.case_number ?? ad1Info?.companion_log?.case_number ?? ufedInfo?.case_info?.case_identifier;
      const extractedEvidenceNumber = ewfInfo?.evidence_number ?? ad1Info?.companion_log?.evidence_number ?? ufedInfo?.evidence_number;
      const extractedExaminer = ewfInfo?.examiner_name ?? ad1Info?.companion_log?.examiner ?? ufedInfo?.case_info?.examiner_name;
      const extractedAgency = ufedInfo?.case_info?.department;
      
      if (extractedCaseNumber || extractedEvidenceNumber || extractedExaminer) {
        // Update case info
        setCaseInfo(prev => ({
          ...prev,
          case_number: prev.case_number || extractedCaseNumber || "",
          agency: prev.agency || extractedAgency || undefined,
        }));
        
        // Update examiner
        if (extractedExaminer && !examiner().name) {
          setExaminer(prev => ({
            ...prev,
            name: extractedExaminer,
          }));
        }
        
        break; // Only use first container with info
      }
    }
  };
  
  // Apply template settings
  const applyTemplate = (templateId: ReportTemplateType) => {
    const template = REPORT_TEMPLATES.find(t => t.id === templateId);
    if (!template) return;
    
    setSelectedTemplate(templateId);
    setEnabledSections(template.defaultSections);
    setMetadata(prev => ({
      ...prev,
      classification: template.defaultClassification,
    }));
    
    // Apply default methodology and scope if provided
    if (template.defaultMethodology && !methodology()) {
      setMethodology(template.defaultMethodology);
    }
    if (template.defaultScope && !scope()) {
      setScope(template.defaultScope);
    }
    
    setShowTemplateSelector(false);
  };
  
  // Chain of custody management
  const addCustodyRecord = () => {
    const newRecord: CustodyRecord = {
      timestamp: new Date().toISOString(),
      action: "Received",
      handler: examiner().name || "",
      location: "",
      notes: "",
    };
    setChainOfCustody([...chainOfCustody(), newRecord]);
  };
  
  const updateCustodyRecord = (index: number, updates: Partial<CustodyRecord>) => {
    const current = chainOfCustody();
    const updated = [...current];
    updated[index] = { ...updated[index], ...updates };
    setChainOfCustody(updated);
  };
  
  const removeCustodyRecord = (index: number) => {
    setChainOfCustody(chainOfCustody().filter((_, i) => i !== index));
  };
  
  // Get current template
  const currentTemplate = () => REPORT_TEMPLATES.find(t => t.id === selectedTemplate());
  
  // Helper: Group segmented containers (E01/E02, ad1/ad2, etc.) into logical evidence items
  // This ensures multi-segment forensic images appear as a single evidence item with combined metadata
  const getGroupedEvidenceFiles = () => {
    const files = props.files;
    const grouped = new Map<string, {
      primaryFile: typeof files[0];
      segments: typeof files[0][];
      totalSize: number;
      segmentCount: number;
    }>();
    
    // Regex to match segment extensions (E01-E99, ad1-ad99, etc.)
    const segmentPattern = /^(.+)\.(E|L|Ex|Lx|ad|s)(\d{2,})$/i;
    
    for (const file of files) {
      const match = file.filename.match(segmentPattern);
      
      if (match) {
        // This is a segment file - group by base name
        const baseName = match[1];
        const prefix = match[2].toLowerCase();
        const segNum = parseInt(match[3], 10);
        
        // Get or create group
        const key = `${file.path.substring(0, file.path.lastIndexOf('/'))}/${baseName}.${prefix}`;
        
        if (!grouped.has(key)) {
          grouped.set(key, {
            primaryFile: file,
            segments: [file],
            totalSize: file.size,
            segmentCount: 1,
          });
        } else {
          const group = grouped.get(key)!;
          group.segments.push(file);
          group.totalSize += file.size;
          group.segmentCount++;
          
          // Keep the lowest segment number as primary (E01, ad1, etc.)
          const primaryMatch = group.primaryFile.filename.match(segmentPattern);
          if (primaryMatch && segNum < parseInt(primaryMatch[3], 10)) {
            group.primaryFile = file;
          }
        }
      } else {
        // Single file container - use path as key
        grouped.set(file.path, {
          primaryFile: file,
          segments: [file],
          totalSize: file.size,
          segmentCount: 1,
        });
      }
    }
    
    return Array.from(grouped.values());
  };
  
  // Create a memo for grouped evidence
  const groupedEvidence = createMemo(() => getGroupedEvidenceFiles());
  
  // Load output formats and AI settings on mount
  onMount(async () => {
    try {
      const formats = await invoke<OutputFormat[]>("get_output_formats");
      setOutputFormats(formats);
    } catch (e) {
      console.warn("Failed to load output formats:", e);
      // Fallback formats
      setOutputFormats([
        { format: "Pdf", name: "PDF", description: "Portable Document Format", extension: "pdf", supported: true },
        { format: "Docx", name: "Word", description: "Microsoft Word", extension: "docx", supported: true },
        { format: "Html", name: "HTML", description: "Web page", extension: "html", supported: true },
        { format: "Markdown", name: "Markdown", description: "Plain text", extension: "md", supported: true },
      ]);
    }
    
    // Load AI settings
    try {
      const available = await isAiAvailable();
      setAiAvailable(available);
      
      if (available) {
        const providers = await getAiProviders();
        setAiProviders(providers);
        
        // Set default provider and model
        if (providers.length > 0) {
          setSelectedProvider(providers[0].id);
          setSelectedModel(providers[0].default_model);
        }
        
        // Check Ollama connection
        const ollamaOk = await checkOllamaConnection();
        setOllamaConnected(ollamaOk);
      }
    } catch (e) {
      console.warn("Failed to load AI settings:", e);
    }
    
    // Auto-select all grouped evidence items (by primary file path)
    const grouped = getGroupedEvidenceFiles();
    const allPaths = new Set(grouped.map(g => g.primaryFile.path));
    setSelectedEvidence(allPaths);
    
    // Auto-populate case info from containers
    autoPopulateCaseInfo();
  });
  
  // Build the report object from current state
  const buildReport = (): ForensicReport => {
    const evidenceItems: EvidenceItem[] = [];
    const grouped = groupedEvidence();
    
    grouped.forEach((group, index) => {
      const file = group.primaryFile;
      if (!selectedEvidence().has(file.path)) return;
      
      const info = props.fileInfoMap.get(file.path);
      const hashInfo = props.fileHashMap.get(file.path);
      
      const hashes: HashValue[] = [];
      if (hashInfo) {
        // Map algorithm string to enum type
        const algoMap: Record<string, HashAlgorithmType> = {
          "md5": "MD5",
          "sha1": "SHA1", 
          "sha256": "SHA256",
          "sha512": "SHA512",
          "blake2b": "Blake2b",
          "blake3": "Blake3",
          "xxh3": "XXH3",
          "xxh64": "XXH64",
        };
        const algo = algoMap[hashInfo.algorithm.toLowerCase()] || "SHA256";
        
        hashes.push({
          item: file.filename,
          algorithm: algo,
          value: hashInfo.hash,
          verified: hashInfo.verified ?? undefined,
        });
      }
      
      // Extract info from container metadata (dig into the nested structure)
      const ewfInfo = info?.e01 || info?.l01;
      const ad1Info = info?.ad1;
      const ufedInfo = info?.ufed;
      
      // For multi-segment containers, use the metadata's total_size (actual image size)
      // or fall back to the group's combined file size
      const totalSize = ewfInfo?.total_size ?? ad1Info?.total_size ?? group.totalSize;
      
      // Get acquisition date from various sources
      const acquisitionDate = ewfInfo?.acquiry_date ?? ad1Info?.companion_log?.acquisition_date ?? ufedInfo?.extraction_info?.start_time;
      
      // Get serial number from various sources
      const serialNumber = ewfInfo?.serial_number ?? ufedInfo?.device_info?.serial_number;
      
      // Get acquisition tool from various sources
      const acquisitionTool = ewfInfo?.notes ?? ufedInfo?.extraction_info?.acquisition_tool ?? ad1Info?.companion_log?.notes;
      
      // Get notes from various sources, and add segment info for multi-segment containers
      let notes = ewfInfo?.notes ?? ad1Info?.companion_log?.notes ?? info?.note;
      if (group.segmentCount > 1) {
        const segmentNote = `Multi-segment image: ${group.segmentCount} segments`;
        notes = notes ? `${notes}\n${segmentNote}` : segmentNote;
      }
      
      // Generate display name (normalize segment extension for multi-segment containers)
      let displayName = file.filename;
      if (group.segmentCount > 1) {
        const match = file.filename.match(/^(.+)\.(E|L|Ex|Lx|ad|s)\d{2,}$/i);
        if (match) {
          displayName = `${match[1]}.${match[2].toUpperCase()}01`;
        }
      }
      
      const evidenceItem: EvidenceItem = {
        evidence_id: `E${String(index + 1).padStart(3, '0')}`,
        description: displayName,
        evidence_type: detectEvidenceType(file, info),
        serial_number: serialNumber ?? undefined,
        capacity: totalSize ? formatBytes(totalSize) : undefined,
        acquisition_date: acquisitionDate ?? undefined,
        acquisition_tool: acquisitionTool ?? undefined,
        acquisition_hashes: hashes,
        verification_hashes: [],
        notes: notes ?? undefined,
      };
      
      evidenceItems.push(evidenceItem);
    });
    
    // Build signatures array
    const signatures: SignatureRecord[] = [];
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
  
  // Generate preview
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
  
  // Effect to generate preview when entering preview step
  createEffect(() => {
    if (currentStep() === "preview") {
      generatePreview();
    }
  });
  
  // Export report
  const exportReport = async () => {
    setExporting(true);
    setExportError(null);
    
    try {
      const report = buildReport();
      const format = outputFormats().find(f => f.format === selectedFormat());
      
      // Open save dialog
      const path = await save({
        title: "Save Report",
        defaultPath: `${report.metadata.report_number}.${format?.extension || 'pdf'}`,
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
  
  // Navigate between steps
  const goToStep = (step: WizardStep) => {
    setCurrentStep(step);
  };
  
  const nextStep = () => {
    const idx = STEPS.findIndex(s => s.id === currentStep());
    if (idx < STEPS.length - 1) {
      setCurrentStep(STEPS[idx + 1].id);
    }
  };
  
  const prevStep = () => {
    const idx = STEPS.findIndex(s => s.id === currentStep());
    if (idx > 0) {
      setCurrentStep(STEPS[idx - 1].id);
    }
  };
  
  // Toggle evidence selection
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
  
  // Add finding
  const addFinding = () => {
    const newFinding: Finding = {
      id: `F${String(findings().length + 1).padStart(3, '0')}`,
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
  
  // Update finding
  const updateFinding = (index: number, updates: Partial<Finding>) => {
    const current = findings();
    const updated = [...current];
    updated[index] = { ...updated[index], ...updates };
    setFindings(updated);
  };
  
  // Remove finding
  const removeFinding = (index: number) => {
    setFindings(findings().filter((_, i) => i !== index));
  };
  
  // Add certification
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
  
  // Remove certification
  const removeCertification = (cert: string) => {
    setExaminer({
      ...examiner(),
      certifications: examiner().certifications.filter(c => c !== cert),
    });
  };
  
  // Get current provider info
  const currentProviderInfo = () => aiProviders().find(p => p.id === selectedProvider());
  
  // Build evidence context for AI
  const buildAiContext = () => {
    const report = buildReport();
    const evidenceContext = buildEvidenceContext(
      report.evidence_items.map(item => ({
        evidence_id: item.evidence_id,
        description: item.description,
        evidence_type: item.evidence_type,
        model: item.model,
        serial_number: item.serial_number,
        capacity: item.capacity,
        acquisition_hashes: item.acquisition_hashes.map(h => ({
          item: h.algorithm,
          algorithm: h.algorithm,
          value: h.value,
          verified: h.verified,
        })),
        image_info: item.acquisition_tool ? {
          format: "",
          file_names: [],
          total_size: 0,
          acquisition_tool: item.acquisition_tool,
        } : undefined,
        notes: item.notes,
      }))
    );
    
    // Add case context
    let context = `=== CASE INFORMATION ===\n`;
    context += `Case Number: ${caseInfo().case_number || "Not specified"}\n`;
    context += `Case Name: ${caseInfo().case_name || "Not specified"}\n`;
    context += `Agency: ${caseInfo().agency || "Not specified"}\n`;
    context += `Investigation Type: ${caseInfo().investigation_type || "Not specified"}\n`;
    context += `\n${evidenceContext}`;
    
    // Add existing findings summary
    if (findings().length > 0) {
      context += `\n=== FINDINGS ===\n`;
      for (const finding of findings()) {
        context += `- ${finding.title} (${finding.severity}): ${finding.description}\n`;
      }
    }
    
    return context;
  };
  
  // Generate AI narrative for a section
  const generateNarrative = async (type: NarrativeType, setter: (value: string) => void) => {
    if (!aiAvailable() || aiGenerating()) return;
    
    // Check Ollama connection for Ollama provider
    if (selectedProvider() === "ollama" && !ollamaConnected()) {
      setAiError("Ollama is not running. Please start Ollama first (run 'ollama serve' in terminal).");
      return;
    }
    
    // Check API key for OpenAI
    if (selectedProvider() === "openai" && !apiKey()) {
      setAiError("OpenAI API key is required. Please enter your API key in AI settings.");
      setShowAiSettings(true);
      return;
    }
    
    setAiGenerating(type);
    setAiError(null);
    
    try {
      const context = buildAiContext();
      const result = await generateAiNarrative(
        context,
        type,
        selectedProvider(),
        selectedModel(),
        selectedProvider() === "openai" ? apiKey() : undefined
      );
      setter(result);
    } catch (e) {
      console.error("AI generation failed:", e);
      setAiError(String(e));
    } finally {
      setAiGenerating(null);
    }
  };
  
  // Refresh Ollama connection status
  const refreshOllamaStatus = async () => {
    try {
      const connected = await checkOllamaConnection();
      setOllamaConnected(connected);
    } catch (e) {
      setOllamaConnected(false);
    }
  };

  // Close on Escape key
  onMount(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        props.onClose();
      }
    };
    document.addEventListener('keydown', handleEscape);
    onCleanup(() => document.removeEventListener('keydown', handleEscape));
  });

  return (
    <div class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50">
      <div 
        ref={modalRef}
        class="bg-bg-panel rounded-xl shadow-2xl w-[900px] max-h-[85vh] flex flex-col border border-border/50"
        role="dialog"
        aria-modal="true"
        aria-labelledby="report-wizard-title"
      >
        {/* Header */}
        <div class="flex items-center justify-between px-5 py-4 border-b border-border/50">
          <h2 id="report-wizard-title" class="text-lg font-semibold flex items-center gap-2.5">
            <div class="w-8 h-8 rounded-lg bg-accent/10 flex items-center justify-center">
              <HiOutlineDocumentText class="w-5 h-5 text-accent" />
            </div>
            Generate Forensic Report
          </h2>
          <button 
            class="text-txt-muted hover:text-txt hover:bg-bg-hover p-1.5 rounded-lg transition-colors"
            onClick={props.onClose}
            aria-label="Close report wizard"
          >
            <HiOutlineXMark class="w-5 h-5" />
          </button>
        </div>
        
        {/* Step indicators - cleaner horizontal stepper */}
        <div class="px-5 py-3 border-b border-border/50 bg-surface/30">
          <div class="flex items-center justify-between">
            <For each={STEPS}>
              {(step, index) => {
                const stepIndex = () => STEPS.findIndex(s => s.id === currentStep());
                const isActive = () => currentStep() === step.id;
                const isCompleted = () => index() < stepIndex();
                const isClickable = () => index() <= stepIndex() + 1;
                
                return (
                  <>
                    <button
                      class={`flex items-center gap-2 px-3 py-2 rounded-lg text-sm font-medium transition-all ${
                        isActive() 
                          ? 'bg-accent text-white shadow-sm shadow-accent/25' 
                          : isCompleted()
                            ? 'text-accent hover:bg-accent/10'
                            : 'text-txt-muted hover:bg-bg-hover'
                      } ${!isClickable() ? 'opacity-50 cursor-not-allowed' : 'cursor-pointer'}`}
                      onClick={() => isClickable() && goToStep(step.id)}
                      disabled={!isClickable()}
                    >
                      <div class={`w-6 h-6 rounded-full flex items-center justify-center text-xs font-bold ${
                        isActive() 
                          ? 'bg-accent/20 text-accent' 
                          : isCompleted()
                            ? 'bg-accent/20 text-accent'
                            : 'bg-bg-hover'
                      }`}>
                        {isCompleted() ? '✓' : index() + 1}
                      </div>
                      <span class="hidden sm:inline">{step.label}</span>
                    </button>
                    <Show when={index() < STEPS.length - 1}>
                      <div class={`flex-1 h-0.5 mx-2 rounded ${
                        isCompleted() ? 'bg-accent' : 'bg-border'
                      }`} />
                    </Show>
                  </>
                );
              }}
            </For>
          </div>
        </div>
        
        {/* Content area */}
        <div class="flex-1 overflow-y-auto p-5">
          {/* Step 1: Case Information */}
          <Show when={currentStep() === "case"}>
            <div class="space-y-5">
              {/* Template Selector */}
              <Show when={showTemplateSelector()}>
                <div class="mb-6">
                  <div class="flex items-center gap-3 mb-4">
                    <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-accent/20 to-accent/5 flex items-center justify-center">
                      <span class="text-xl">📋</span>
                    </div>
                    <div>
                      <h3 class="text-base font-semibold">Choose Report Template</h3>
                      <p class="text-sm text-txt/60">Pre-configure settings based on investigation type</p>
                    </div>
                  </div>
                  <div class="grid grid-cols-2 lg:grid-cols-3 gap-3">
                    <For each={REPORT_TEMPLATES}>
                      {(template) => (
                        <button
                          type="button"
                          class={`group relative p-4 rounded-xl border-2 text-left transition-all duration-200 ${
                            selectedTemplate() === template.id
                              ? "border-accent bg-accent/5 ring-2 ring-accent/20"
                              : "border-border/50 bg-surface/50 hover:border-accent/50 hover:bg-surface"
                          }`}
                          onClick={() => applyTemplate(template.id)}
                        >
                          <div class="flex items-start gap-3">
                            <span class="text-2xl">{template.icon}</span>
                            <div class="flex-1 min-w-0">
                              <span class="font-medium text-sm block">{template.name}</span>
                              <p class="text-xs text-txt/50 mt-0.5 line-clamp-2">{template.description}</p>
                            </div>
                          </div>
                          <Show when={selectedTemplate() === template.id}>
                            <div class="absolute top-2 right-2 w-5 h-5 rounded-full bg-accent flex items-center justify-center">
                              <span class="text-white text-xs">✓</span>
                            </div>
                          </Show>
                        </button>
                      )}
                    </For>
                  </div>
                  <div class="mt-4 flex justify-end">
                    <button
                      type="button"
                      class="text-sm text-txt/50 hover:text-accent transition-colors"
                      onClick={() => setShowTemplateSelector(false)}
                    >
                      Continue with {currentTemplate()?.name || "Custom"} →
                    </button>
                  </div>
                </div>
                <div class="border-t border-border/30 mb-5" />
              </Show>
              
              {/* Template Badge (when collapsed) */}
              <Show when={!showTemplateSelector()}>
                <div class="flex items-center justify-between p-3 bg-surface/50 rounded-xl border border-border/30">
                  <div class="flex items-center gap-3">
                    <span class="text-xl">{currentTemplate()?.icon || "📋"}</span>
                    <div>
                      <span class="text-sm font-medium">{currentTemplate()?.name || "Custom"} Template</span>
                      <p class="text-xs text-txt/50">Pre-configured report settings</p>
                    </div>
                  </div>
                  <button
                    type="button"
                    class="text-sm text-accent hover:underline px-3 py-1.5 rounded-lg hover:bg-accent/10 transition-colors"
                    onClick={() => setShowTemplateSelector(true)}
                  >
                    Change
                  </button>
                </div>
              </Show>
              
              {/* Section Header */}
              <div class="flex items-center gap-2">
                <HiOutlineClipboardDocument class="w-5 h-5 text-accent" />
                <h3 class="text-base font-semibold">Case Information</h3>
              </div>
              
              <div class="grid grid-cols-2 gap-4">
                <div>
                  <label class="label">Case Number *</label>
                  <input
                    type="text"
                    class="input"
                    value={caseInfo().case_number}
                    onInput={(e) => setCaseInfo({ ...caseInfo(), case_number: e.currentTarget.value })}
                    placeholder="e.g., 2026-CF-00123"
                  />
                </div>
                
                <div>
                  <label class="label">Case Name</label>
                  <input
                    type="text"
                    class="input"
                    value={caseInfo().case_name || ""}
                    onInput={(e) => setCaseInfo({ ...caseInfo(), case_name: e.currentTarget.value || undefined })}
                    placeholder="e.g., State v. John Doe"
                  />
                </div>
                
                <div>
                  <label class="label">Agency/Department</label>
                  <input
                    type="text"
                    class="input"
                    value={caseInfo().agency || ""}
                    onInput={(e) => setCaseInfo({ ...caseInfo(), agency: e.currentTarget.value || undefined })}
                    placeholder="e.g., Metro Police Department"
                  />
                </div>
                
                <div>
                  <label class="label">Requestor</label>
                  <input
                    type="text"
                    class="input"
                    value={caseInfo().requestor || ""}
                    onInput={(e) => setCaseInfo({ ...caseInfo(), requestor: e.currentTarget.value || undefined })}
                    placeholder="e.g., Det. Jane Smith"
                  />
                </div>
                
                <div>
                  <label class="label">Investigation Type</label>
                  <input
                    type="text"
                    class="input"
                    value={caseInfo().investigation_type || ""}
                    onInput={(e) => setCaseInfo({ ...caseInfo(), investigation_type: e.currentTarget.value || undefined })}
                    placeholder="e.g., Fraud Investigation"
                  />
                </div>
                
                <div>
                  <label class="label">Classification</label>
                  <select
                    class="input"
                    value={metadata().classification}
                    onChange={(e) => setMetadata({ ...metadata(), classification: e.currentTarget.value as Classification })}
                  >
                    <For each={CLASSIFICATIONS}>
                      {(c) => <option value={c.value}>{c.label}</option>}
                    </For>
                  </select>
                </div>
              </div>
              
              <div>
                <label class="label">Case Description</label>
                <textarea
                  class="textarea h-24"
                  value={caseInfo().description || ""}
                  onInput={(e) => setCaseInfo({ ...caseInfo(), description: e.currentTarget.value || undefined })}
                  placeholder="Brief description of the case and examination request..."
                />
              </div>
              
              <div class="grid grid-cols-2 gap-4">
                <div>
                  <label class="label">Report Title</label>
                  <input
                    type="text"
                    class="input"
                    value={metadata().title}
                    onInput={(e) => setMetadata({ ...metadata(), title: e.currentTarget.value })}
                  />
                </div>
                
                <div>
                  <label class="label">Report Number</label>
                  <input
                    type="text"
                    class="input"
                    value={metadata().report_number}
                    onInput={(e) => setMetadata({ ...metadata(), report_number: e.currentTarget.value })}
                  />
                </div>
              </div>
            </div>
          </Show>
          
          {/* Step 2: Examiner Information */}
          <Show when={currentStep() === "examiner"}>
            <div class="space-y-5">
              <div class="flex items-center gap-2">
                <HiOutlineUser class="w-5 h-5 text-accent" />
                <h3 class="text-base font-semibold">Examiner Information</h3>
              </div>
              
              <div class="grid grid-cols-2 gap-4">
                <div>
                  <label class="label">Full Name *</label>
                  <input
                    type="text"
                    class="input"
                    value={examiner().name}
                    onInput={(e) => setExaminer({ ...examiner(), name: e.currentTarget.value })}
                    placeholder="e.g., John Smith"
                  />
                </div>
                
                <div>
                  <label class="label">Title</label>
                  <input
                    type="text"
                    class="input"
                    value={examiner().title || ""}
                    onInput={(e) => setExaminer({ ...examiner(), title: e.currentTarget.value || undefined })}
                    placeholder="e.g., Senior Digital Forensic Examiner"
                  />
                </div>
                
                <div>
                  <label class="label">Organization</label>
                  <input
                    type="text"
                    class="input"
                    value={examiner().organization || ""}
                    onInput={(e) => setExaminer({ ...examiner(), organization: e.currentTarget.value || undefined })}
                    placeholder="e.g., Metro Police Forensic Lab"
                  />
                </div>
                
                <div>
                  <label class="label">Badge/ID Number</label>
                  <input
                    type="text"
                    class="input"
                    value={examiner().badge_number || ""}
                    onInput={(e) => setExaminer({ ...examiner(), badge_number: e.currentTarget.value || undefined })}
                    placeholder="e.g., F-1234"
                  />
                </div>
                
                <div>
                  <label class="label">Email</label>
                  <input
                    type="email"
                    class="input"
                    value={examiner().email || ""}
                    onInput={(e) => setExaminer({ ...examiner(), email: e.currentTarget.value || undefined })}
                    placeholder="e.g., jsmith@agency.gov"
                  />
                </div>
                
                <div>
                  <label class="label">Phone</label>
                  <input
                    type="tel"
                    class="input"
                    value={examiner().phone || ""}
                    onInput={(e) => setExaminer({ ...examiner(), phone: e.currentTarget.value || undefined })}
                    placeholder="e.g., (555) 123-4567"
                  />
                </div>
              </div>
              
              {/* Certifications with improved UI */}
              <div class="card">
                <label class="label">Certifications</label>
                <div class="flex gap-2 mb-3">
                  <input
                    type="text"
                    class="input-sm flex-1"
                    value={newCert()}
                    onInput={(e) => setNewCert(e.currentTarget.value)}
                    onKeyDown={(e) => e.key === "Enter" && addCertification()}
                    placeholder="e.g., EnCE, GCFE, ACE..."
                  />
                  <button
                    class="px-4 py-2 rounded-lg text-sm font-medium bg-accent/10 text-accent hover:bg-accent/20 transition-colors"
                    onClick={addCertification}
                  >
                    + Add
                  </button>
                </div>
                <div class="flex flex-wrap gap-2">
                  <For each={examiner().certifications}>
                    {(cert) => (
                      <span class="inline-flex items-center gap-1.5 px-3 py-1.5 bg-accent/10 text-accent rounded-full text-sm font-medium">
                        {cert}
                        <button
                          class="w-4 h-4 rounded-full bg-accent/20 hover:bg-accent/30 flex items-center justify-center text-xs transition-colors"
                          onClick={() => removeCertification(cert)}
                        >
                          <HiOutlineXMark class="w-3 h-3" />
                        </button>
                      </span>
                    )}
                  </For>
                  <Show when={examiner().certifications.length === 0}>
                    <span class="text-sm text-txt/40 italic">No certifications added</span>
                  </Show>
                </div>
              </div>
            </div>
          </Show>
          
          {/* Step 3: Evidence Selection */}
          <Show when={currentStep() === "evidence"}>
            <div class="space-y-5">
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-2">
                  <HiOutlineCircleStack class="w-5 h-5 text-accent" />
                  <h3 class="text-base font-semibold">Evidence Items</h3>
                </div>
                <div class="flex items-center gap-3">
                  <span class="text-sm text-txt/60">
                    <span class="font-medium text-accent">{selectedEvidence().size}</span> of {groupedEvidence().length} selected
                  </span>
                  <Show when={groupedEvidence().length > 0}>
                    <button
                      class="text-xs text-accent hover:underline"
                      onClick={() => {
                        if (selectedEvidence().size === groupedEvidence().length) {
                          setSelectedEvidence(new Set<string>());
                        } else {
                          setSelectedEvidence(new Set<string>(groupedEvidence().map(g => g.primaryFile.path)));
                        }
                      }}
                    >
                      {selectedEvidence().size === groupedEvidence().length ? 'Deselect All' : 'Select All'}
                    </button>
                  </Show>
                </div>
              </div>
              
              <Show when={groupedEvidence().length === 0}>
                <div class="text-center py-12 bg-surface/30 rounded-xl border border-border/30">
                  <div class="w-16 h-16 mx-auto mb-4 rounded-2xl bg-accent/10 flex items-center justify-center">
                    <span class="text-3xl">📂</span>
                  </div>
                  <p class="font-medium text-txt/80">No evidence files discovered</p>
                  <p class="text-sm text-txt/50 mt-1">Scan a directory first to discover forensic images</p>
                </div>
              </Show>
              
              <div class="space-y-2 max-h-[300px] overflow-y-auto pr-1">
                <For each={groupedEvidence()}>
                  {(group) => {
                    const file = group.primaryFile;
                    const info = () => props.fileInfoMap.get(file.path);
                    const hashInfo = () => props.fileHashMap.get(file.path);
                    const isSelected = () => selectedEvidence().has(file.path);
                    
                    // Extract display info from container - use group's total size for segments
                    const displayInfo = () => {
                      const i = info();
                      if (!i) return { totalSize: group.totalSize, acqDate: null };
                      const ewfInfo = i.e01 || i.l01;
                      const ad1Info = i.ad1;
                      // For segmented containers, use the metadata's total_size (image size) or fallback to group total
                      const totalSize = ewfInfo?.total_size ?? ad1Info?.total_size ?? group.totalSize;
                      const acqDate = ewfInfo?.acquiry_date ?? ad1Info?.companion_log?.acquisition_date;
                      return { totalSize, acqDate };
                    };
                    
                    // Get base name without segment extension for multi-segment containers
                    const displayName = () => {
                      if (group.segmentCount > 1) {
                        // Remove segment extension to show clean base name
                        const match = file.filename.match(/^(.+)\.(E|L|Ex|Lx|ad|s)\d{2,}$/i);
                        if (match) {
                          return `${match[1]}.${match[2].toUpperCase()}01`;
                        }
                      }
                      return file.filename;
                    };
                    
                    return (
                      <div 
                        class={`flex items-start gap-3 p-3.5 rounded-xl border-2 cursor-pointer transition-all duration-200 ${
                          isSelected() 
                            ? 'border-accent bg-accent/5 shadow-sm shadow-accent/10' 
                            : 'border-border/30 bg-surface/30 hover:border-accent/30 hover:bg-surface/50'
                        }`}
                        onClick={() => toggleEvidence(file.path)}
                      >
                        <div class={`w-5 h-5 rounded-md border-2 flex items-center justify-center mt-0.5 transition-colors ${
                          isSelected() ? 'bg-accent border-accent' : 'border-border/50'
                        }`}>
                          <Show when={isSelected()}>
                            <span class="text-white text-xs font-bold">✓</span>
                          </Show>
                        </div>
                        <div class="flex-1 min-w-0">
                          <div class="flex items-center gap-2 flex-wrap">
                            <span class="font-medium text-sm truncate">{displayName()}</span>
                            <span class="text-xs px-2 py-0.5 bg-accent/10 text-accent rounded-full font-medium">
                              {file.container_type}
                            </span>
                            <Show when={group.segmentCount > 1}>
                              <span class="text-xs px-2 py-0.5 bg-blue-500/10 text-blue-400 rounded-full font-medium">
                                {group.segmentCount} segments
                              </span>
                            </Show>
                          </div>
                          <div class="text-xs text-txt/50 truncate mt-0.5">
                            {file.path.substring(0, file.path.lastIndexOf('/'))}
                          </div>
                          <div class="flex items-center gap-3 mt-2 flex-wrap">
                            <Show when={displayInfo()?.totalSize}>
                              <span class="text-xs text-txt/60 flex items-center gap-1">
                                <HiOutlineServer class="w-3 h-3" /> {formatBytes(displayInfo()!.totalSize!)}
                                <Show when={group.segmentCount > 1}>
                                  <span class="text-txt/40">(total)</span>
                                </Show>
                              </span>
                            </Show>
                            <Show when={displayInfo()?.acqDate}>
                              <span class="text-xs text-txt/60 flex items-center gap-1">
                                <HiOutlineCalendarDays class="w-3 h-3" /> {displayInfo()!.acqDate}
                              </span>
                            </Show>
                            <Show when={hashInfo()}>
                              <span class={`text-xs font-mono flex items-center gap-1 ${
                                hashInfo()!.verified === true ? 'text-success' : 
                                hashInfo()!.verified === false ? 'text-error' : 'text-txt/60'
                              }`}>
                                <HiOutlineCheckCircle class="w-3 h-3" /> {hashInfo()!.algorithm}
                                {hashInfo()!.verified === true && " ✓"}
                                {hashInfo()!.verified === false && " ✗"}
                              </span>
                            </Show>
                          </div>
                        </div>
                      </div>
                    );
                  }}
                </For>
              </div>
              
              {/* Chain of Custody Section */}
              <Show when={enabledSections().chainOfCustody}>
                <div class="mt-6 pt-5 border-t border-border/30">
                  <div class="flex items-center justify-between mb-4">
                    <div class="flex items-center gap-2">
                      <span class="text-lg">🔗</span>
                      <h4 class="text-sm font-semibold">Chain of Custody</h4>
                      <span class="text-xs px-2 py-0.5 bg-accent/10 text-accent rounded-full font-medium">
                        {chainOfCustody().length} records
                      </span>
                    </div>
                    <button
                      type="button"
                      class="px-3 py-1.5 rounded-lg text-sm font-medium bg-accent/10 text-accent hover:bg-accent/20 transition-colors"
                      onClick={addCustodyRecord}
                    >
                      + Add Record
                    </button>
                  </div>
                  
                  <Show when={chainOfCustody().length === 0}>
                    <div class="text-center py-8 bg-surface/30 rounded-xl border-2 border-dashed border-border/30">
                      <div class="w-12 h-12 mx-auto mb-3 rounded-xl bg-accent/10 flex items-center justify-center">
                        <span class="text-2xl">📋</span>
                      </div>
                      <p class="text-sm font-medium text-txt/70">No chain of custody records</p>
                      <p class="text-xs text-txt/50 mt-1">Add records to document evidence handling</p>
                    </div>
                  </Show>
                  
                  <div class="space-y-3">
                    <For each={chainOfCustody()}>
                      {(record, index) => (
                        <div class="p-4 bg-surface/50 border border-border/30 rounded-xl">
                          <div class="grid grid-cols-4 gap-3">
                            <div>
                              <label class="block text-xs text-txt/50 mb-1.5">Date/Time</label>
                              <input
                                type="datetime-local"
                                class="w-full px-2.5 py-2 bg-bg-panel border border-border/50 rounded-lg text-sm focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/20 transition-all"
                                value={record.timestamp.slice(0, 16)}
                                onInput={(e) => updateCustodyRecord(index(), { timestamp: new Date(e.currentTarget.value).toISOString() })}
                              />
                            </div>
                            <div>
                              <label class="block text-xs text-txt/50 mb-1.5">Action</label>
                              <select
                                class="w-full px-2.5 py-2 bg-bg-panel border border-border/50 rounded-lg text-sm focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/20 transition-all"
                                value={record.action}
                                onChange={(e) => updateCustodyRecord(index(), { action: e.currentTarget.value })}
                              >
                                <option value="Received">Received</option>
                                <option value="Transferred">Transferred</option>
                                <option value="Imaged">Imaged</option>
                                <option value="Analyzed">Analyzed</option>
                                <option value="Stored">Stored</option>
                                <option value="Released">Released</option>
                                <option value="Returned">Returned</option>
                                <option value="Destroyed">Destroyed</option>
                              </select>
                            </div>
                            <div>
                              <label class="block text-xs text-txt/50 mb-1.5">Handler</label>
                              <input
                                type="text"
                                class="w-full px-2.5 py-2 bg-bg-panel border border-border/50 rounded-lg text-sm focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/20 transition-all"
                                value={record.handler}
                                onInput={(e) => updateCustodyRecord(index(), { handler: e.currentTarget.value })}
                                placeholder="Name of handler"
                              />
                            </div>
                            <div>
                              <label class="block text-xs text-txt/50 mb-1.5">Location</label>
                              <input
                                type="text"
                                class="w-full px-2.5 py-2 bg-bg-panel border border-border/50 rounded-lg text-sm focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/20 transition-all"
                                value={record.location || ""}
                                onInput={(e) => updateCustodyRecord(index(), { location: e.currentTarget.value || undefined })}
                                placeholder="Storage location"
                              />
                            </div>
                          </div>
                          <div class="mt-3 flex gap-2">
                            <input
                              type="text"
                              class="flex-1 px-2.5 py-2 bg-bg-panel border border-border/50 rounded-lg text-sm focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/20 transition-all"
                              value={record.notes || ""}
                              onInput={(e) => updateCustodyRecord(index(), { notes: e.currentTarget.value || undefined })}
                              placeholder="Additional notes..."
                            />
                            <button
                              type="button"
                              class="p-2 text-error/70 hover:text-error hover:bg-error/10 rounded-lg transition-colors"
                              onClick={() => removeCustodyRecord(index())}
                              title="Remove custody record"
                            >
                              <HiOutlineXMark class="w-4 h-4" />
                            </button>
                          </div>
                        </div>
                      )}
                    </For>
                  </div>
                </div>
              </Show>
            </div>
          </Show>
          
          {/* Step 4: Findings */}
          <Show when={currentStep() === "findings"}>
            <div class="space-y-5">
              {/* AI Settings Panel */}
              <Show when={aiAvailable()}>
                <div class="border border-accent/30 rounded-xl p-4 bg-accent/5">
                  <div class="flex items-center justify-between mb-3">
                    <div class="flex items-center gap-2">
                      <div class="w-8 h-8 rounded-lg bg-accent/20 flex items-center justify-center">
                        <HiOutlineCpuChip class="w-5 h-5 text-accent" />
                      </div>
                      <div>
                        <span class="font-medium text-sm">AI Report Assistant</span>
                        <Show when={selectedProvider() === "ollama"}>
                          <span class={`ml-2 text-xs px-1.5 py-0.5 rounded ${ollamaConnected() ? 'bg-success/20 text-success' : 'bg-error/20 text-error'}`}>
                            {ollamaConnected() ? "Connected" : "Disconnected"}
                          </span>
                        </Show>
                      </div>
                    </div>
                    <button 
                      class="text-sm text-accent hover:underline"
                      onClick={() => setShowAiSettings(!showAiSettings())}
                    >
                      {showAiSettings() ? "Hide Settings" : "Settings"}
                    </button>
                  </div>
                  
                  <Show when={showAiSettings()}>
                    <div class="grid grid-cols-3 gap-3 mt-3 pt-3 border-t border-border">
                      <div>
                        <label class="block text-xs text-txt-muted mb-1">Provider</label>
                        <select
                          class="w-full px-2 py-1.5 bg-bg border border-border rounded text-sm"
                          value={selectedProvider()}
                          onChange={(e) => {
                            const provider = e.currentTarget.value;
                            setSelectedProvider(provider);
                            const info = aiProviders().find(p => p.id === provider);
                            if (info) setSelectedModel(info.default_model);
                            if (provider === "ollama") refreshOllamaStatus();
                          }}
                        >
                          <For each={aiProviders()}>
                            {(p) => <option value={p.id}>{p.name}</option>}
                          </For>
                        </select>
                      </div>
                      
                      <div>
                        <label class="block text-xs text-txt-muted mb-1">Model</label>
                        <select
                          class="w-full px-2 py-1.5 bg-bg border border-border rounded text-sm"
                          value={selectedModel()}
                          onChange={(e) => setSelectedModel(e.currentTarget.value)}
                        >
                          <For each={currentProviderInfo()?.available_models || []}>
                            {(m) => <option value={m}>{m}</option>}
                          </For>
                        </select>
                      </div>
                      
                      <Show when={currentProviderInfo()?.requires_api_key}>
                        <div>
                          <label class="block text-xs text-txt-muted mb-1">API Key</label>
                          <input
                            type="password"
                            class="w-full px-2 py-1.5 bg-bg border border-border rounded text-sm"
                            value={apiKey()}
                            onInput={(e) => setApiKey(e.currentTarget.value)}
                            placeholder="sk-..."
                          />
                        </div>
                      </Show>
                      
                      <Show when={selectedProvider() === "ollama" && !ollamaConnected()}>
                        <div class="col-span-3 text-sm text-error flex items-center gap-2">
                          <HiOutlineExclamationTriangle class="w-4 h-4" />
                          <span>Ollama not running.</span>
                          <button class="text-accent hover:underline" onClick={refreshOllamaStatus}>
                            Retry
                          </button>
                          <span class="text-txt-muted">| Run: <code class="bg-bg px-1 rounded">ollama serve</code></span>
                        </div>
                      </Show>
                    </div>
                  </Show>
                  
                  <Show when={aiError()}>
                    <div class="mt-2 text-sm text-error bg-error/10 rounded p-2">
                      {aiError()}
                    </div>
                  </Show>
                </div>
              </Show>
              
              <div class="flex items-center justify-between">
                <h3 class="text-lg font-medium">Findings</h3>
                <button class="btn-action-primary" onClick={addFinding}>
                  + Add Finding
                </button>
              </div>
              
              <div class="grid grid-cols-2 gap-4">
                <div>
                  <div class="flex items-center justify-between mb-1">
                    <label class="text-sm font-medium">Executive Summary</label>
                    <Show when={aiAvailable()}>
                      <button
                        class="text-xs text-accent hover:underline disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1"
                        onClick={() => generateNarrative("executive_summary", setExecutiveSummary)}
                        disabled={!!aiGenerating()}
                      >
                        {aiGenerating() === "executive_summary" 
                          ? <><HiOutlineArrowPath class="w-3 h-3 animate-spin" /> Generating...</>
                          : <><HiOutlineCpuChip class="w-3 h-3" /> Generate with AI</>
                        }
                      </button>
                    </Show>
                  </div>
                  <textarea
                    class="textarea h-32"
                    value={executiveSummary()}
                    onInput={(e) => setExecutiveSummary(e.currentTarget.value)}
                    placeholder="Brief summary for non-technical readers..."
                  />
                </div>
                
                <div>
                  <label class="label">Scope</label>
                  <textarea
                    class="textarea h-32"
                    value={scope()}
                    onInput={(e) => setScope(e.currentTarget.value)}
                    placeholder="Scope of the examination..."
                  />
                </div>
                
                <div>
                  <div class="flex items-center justify-between mb-1">
                    <label class="label">Methodology</label>
                    <Show when={aiAvailable()}>
                      <button
                        class="text-xs text-accent hover:underline disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1"
                        onClick={() => generateNarrative("methodology", setMethodology)}
                        disabled={!!aiGenerating()}
                      >
                        {aiGenerating() === "methodology" 
                          ? <><HiOutlineArrowPath class="w-3 h-3 animate-spin" /> Generating...</>
                          : <><HiOutlineCpuChip class="w-3 h-3" /> Generate with AI</>
                        }
                      </button>
                    </Show>
                  </div>
                  <textarea
                    class="textarea h-32"
                    value={methodology()}
                    onInput={(e) => setMethodology(e.currentTarget.value)}
                    placeholder="Examination methodology employed..."
                  />
                </div>
                
                <div>
                  <div class="flex items-center justify-between mb-1">
                    <label class="label">Conclusions</label>
                    <Show when={aiAvailable()}>
                      <button
                        class="text-xs text-accent hover:underline disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1"
                        onClick={() => generateNarrative("conclusion", setConclusions)}
                        disabled={!!aiGenerating()}
                      >
                        {aiGenerating() === "conclusion" 
                          ? <><HiOutlineArrowPath class="w-3 h-3 animate-spin" /> Generating...</>
                          : <><HiOutlineCpuChip class="w-3 h-3" /> Generate with AI</>
                        }
                      </button>
                    </Show>
                  </div>
                  <textarea
                    class="textarea h-32"
                    value={conclusions()}
                    onInput={(e) => setConclusions(e.currentTarget.value)}
                    placeholder="Final conclusions..."
                  />
                </div>
              </div>
              
              <Show when={findings().length === 0}>
                <div class="text-center py-6 text-txt-muted border border-dashed border-border rounded">
                  <p>No findings added yet.</p>
                  <p class="text-sm">Click "Add Finding" to document discoveries.</p>
                </div>
              </Show>
              
              <For each={findings()}>
                {(finding, index) => (
                  <div class="border border-border rounded p-3 space-y-3">
                    <div class="flex items-center justify-between">
                      <span class="text-sm font-mono text-txt-muted">{finding.id}</span>
                      <button
                        class="text-error hover:text-error/80 text-sm"
                        onClick={() => removeFinding(index())}
                      >
                        Remove
                      </button>
                    </div>
                    
                    <div class="grid grid-cols-3 gap-3">
                      <div class="col-span-2">
                        <input
                          type="text"
                          class="w-full px-2 py-1.5 bg-bg border border-border rounded text-sm focus:outline-none focus:border-accent"
                          value={finding.title}
                          onInput={(e) => updateFinding(index(), { title: e.currentTarget.value })}
                          placeholder="Finding title..."
                        />
                      </div>
                      
                      <select
                        class="px-2 py-1.5 bg-bg border border-border rounded text-sm focus:outline-none focus:border-accent"
                        value={finding.severity}
                        onChange={(e) => updateFinding(index(), { severity: e.currentTarget.value as Severity })}
                      >
                        <For each={SEVERITIES}>
                          {(s) => <option value={s.value}>{s.label}</option>}
                        </For>
                      </select>
                    </div>
                    
                    <textarea
                      class="w-full px-2 py-1.5 bg-bg border border-border rounded text-sm focus:outline-none focus:border-accent h-20 resize-none"
                      value={finding.description}
                      onInput={(e) => updateFinding(index(), { description: e.currentTarget.value })}
                      placeholder="Detailed description of the finding..."
                    />
                  </div>
                )}
              </For>
            </div>
          </Show>
          
          {/* Step 5: Preview */}
          <Show when={currentStep() === "preview"}>
            <div class="space-y-4">
              <div class="flex items-center justify-between">
                <h3 class="text-lg font-medium">Report Preview</h3>
                <button 
                  class="btn-action-secondary"
                  onClick={generatePreview}
                  disabled={previewLoading()}
                >
                  {previewLoading() ? "Generating..." : "🔄 Refresh Preview"}
                </button>
              </div>
              
              {/* Report Statistics Summary */}
              <div class="grid grid-cols-4 gap-3">
                <div class="p-3 bg-surface border border-border rounded-lg text-center">
                  <div class="text-2xl font-bold text-accent">{selectedEvidence().size}</div>
                  <div class="text-xs text-txt-muted">Evidence Items</div>
                </div>
                <div class="p-3 bg-surface border border-border rounded-lg text-center">
                  <div class="text-2xl font-bold text-accent">{findings().length}</div>
                  <div class="text-xs text-txt-muted">Findings</div>
                </div>
                <div class="p-3 bg-surface border border-border rounded-lg text-center">
                  <div class="text-2xl font-bold text-accent">{chainOfCustody().length}</div>
                  <div class="text-xs text-txt-muted">Custody Records</div>
                </div>
                <div class="p-3 bg-surface border border-border rounded-lg text-center">
                  <div class="text-2xl font-bold text-accent">
                    {findings().filter(f => f.severity === "Critical" || f.severity === "High").length}
                  </div>
                  <div class="text-xs text-txt-muted">Critical/High</div>
                </div>
              </div>
              
              {/* Severity Breakdown */}
              <Show when={findings().length > 0}>
                <div class="p-3 bg-surface border border-border rounded-lg">
                  <h4 class="text-sm font-medium mb-2">Finding Severity Breakdown</h4>
                  <div class="flex gap-4">
                    <For each={SEVERITIES}>
                      {(sev) => {
                        const count = findings().filter(f => f.severity === sev.value).length;
                        return (
                          <Show when={count > 0}>
                            <div class="flex items-center gap-2">
                              <div class="w-3 h-3 rounded-full" style={{ "background-color": sev.color }} />
                              <span class="text-sm">{sev.label}: {count}</span>
                            </div>
                          </Show>
                        );
                      }}
                    </For>
                  </div>
                </div>
              </Show>
              
              {/* Report Completeness Check */}
              <div class="p-3 bg-surface border border-border rounded-lg">
                <h4 class="text-sm font-medium mb-2">Report Completeness</h4>
                <div class="grid grid-cols-3 gap-2 text-sm">
                  <div class="flex items-center gap-2">
                    <span class={caseInfo().case_number ? "text-success" : "text-error"}>
                      {caseInfo().case_number ? "✓" : "✗"}
                    </span>
                    Case Number
                  </div>
                  <div class="flex items-center gap-2">
                    <span class={examiner().name ? "text-success" : "text-error"}>
                      {examiner().name ? "✓" : "✗"}
                    </span>
                    Examiner Name
                  </div>
                  <div class="flex items-center gap-2">
                    <span class={selectedEvidence().size > 0 ? "text-success" : "text-warning"}>
                      {selectedEvidence().size > 0 ? "✓" : "⚠"}
                    </span>
                    Evidence Selected
                  </div>
                  <div class="flex items-center gap-2">
                    <span class={executiveSummary() ? "text-success" : "text-txt-muted"}>
                      {executiveSummary() ? "✓" : "○"}
                    </span>
                    Executive Summary
                  </div>
                  <div class="flex items-center gap-2">
                    <span class={methodology() ? "text-success" : "text-txt-muted"}>
                      {methodology() ? "✓" : "○"}
                    </span>
                    Methodology
                  </div>
                  <div class="flex items-center gap-2">
                    <span class={conclusions() ? "text-success" : "text-txt-muted"}>
                      {conclusions() ? "✓" : "○"}
                    </span>
                    Conclusions
                  </div>
                </div>
              </div>
              
              <Show when={previewLoading()}>
                <div class="flex items-center justify-center py-12">
                  <HiOutlineArrowPath class="w-8 h-8 animate-spin text-accent" />
                </div>
              </Show>
              
              <Show when={!previewLoading() && previewHtml()}>
                {/* Print preview uses white background for accurate paper representation */}
                <div 
                  class="border border-border rounded bg-[#ffffff] text-[#1a1a1a] p-4 max-h-[50vh] overflow-auto"
                  innerHTML={DOMPurify.sanitize(previewHtml() || "")}
                />
              </Show>
            </div>
          </Show>
          
          {/* Step 6: Export */}
          <Show when={currentStep() === "export"}>
            <div class="space-y-4">
              <h3 class="text-lg font-medium">Export Report</h3>
              
              <div class="grid grid-cols-2 gap-3">
                <For each={outputFormats()}>
                  {(format) => (
                    <button
                      class={`p-4 rounded border text-left transition-colors ${
                        selectedFormat() === format.format
                          ? 'border-accent bg-accent/10'
                          : format.supported
                            ? 'border-border hover:border-accent/50'
                            : 'border-border/30 hover:border-accent/30 opacity-50 cursor-not-allowed'
                      }`}
                      onClick={() => format.supported && setSelectedFormat(format.format)}
                      disabled={!format.supported}
                    >
                      <div class="flex items-center gap-2 mb-1">
                        <HiOutlineDocument class="w-5 h-5" />
                        <span class="font-medium">{format.name}</span>
                        <span class="text-xs text-txt/50">.{format.extension}</span>
                      </div>
                      <p class="text-xs text-txt/50">{format.description}</p>
                      <Show when={!format.supported}>
                        <span class="text-xs text-warning mt-1 block">Coming soon</span>
                      </Show>
                    </button>
                  )}
                </For>
              </div>
              
              <Show when={exportError()}>
                <div class="p-3 bg-error/10 border border-error/30 rounded-lg text-error text-sm">
                  Export failed: {exportError()}
                </div>
              </Show>
              
              {/* Signature & Approval Section */}
              <div class="space-y-4 pt-5 border-t border-border/30">
                <div class="flex items-center gap-2">
                  <HiOutlineDocumentCheck class="w-5 h-5 text-accent" />
                  <h4 class="text-sm font-semibold">Signature & Approval</h4>
                </div>
                
                {/* Examiner Signature */}
                <div class="p-4 bg-surface/50 rounded-xl border border-border/30 space-y-3">
                  <div class="flex items-center justify-between">
                    <div class="flex items-center gap-2">
                      <HiOutlineUser class="w-4 h-4 text-accent" />
                      <span class="text-sm font-medium">Examiner Signature</span>
                    </div>
                    <Show when={examiner().name && !examinerSignature()}>
                      <button
                        class="text-xs px-2 py-1 text-accent bg-accent/10 rounded-md hover:bg-accent/20 transition-colors"
                        onClick={() => {
                          setExaminerSignature(examiner().name);
                          setExaminerSignedDate(new Date().toISOString().slice(0, 16));
                        }}
                      >
                        Auto-fill
                      </button>
                    </Show>
                  </div>
                  <div class="grid grid-cols-2 gap-3">
                    <div>
                      <label class="label">Digital Signature</label>
                      <input
                        type="text"
                        class="input-sm italic"
                        placeholder="Type your full name"
                        value={examinerSignature()}
                        onInput={(e) => setExaminerSignature(e.currentTarget.value)}
                      />
                    </div>
                    <div>
                      <label class="label">Date Signed</label>
                      <input
                        type="datetime-local"
                        class="input-sm"
                        value={examinerSignedDate()}
                        onInput={(e) => setExaminerSignedDate(e.currentTarget.value)}
                      />
                    </div>
                  </div>
                </div>
                
                {/* Supervisor Approval */}
                <div class="card space-y-3">
                  <div class="flex items-center gap-2">
                    <HiOutlineUserGroup class="w-4 h-4 text-warning" />
                    <span class="text-sm font-medium">Supervisor Approval</span>
                    <span class="badge badge-accent/30">Optional</span>
                  </div>
                  <div>
                    <label class="label">Supervisor Name</label>
                    <input
                      type="text"
                      class="input-sm"
                      placeholder="Supervisor's full name"
                      value={supervisorName()}
                      onInput={(e) => setSupervisorName(e.currentTarget.value)}
                    />
                  </div>
                  <div class="grid grid-cols-2 gap-3">
                    <div>
                      <label class="label">Supervisor Signature</label>
                      <input
                        type="text"
                        class="input-sm italic"
                        placeholder="Type name as signature"
                        value={supervisorSignature()}
                        onInput={(e) => setSupervisorSignature(e.currentTarget.value)}
                      />
                    </div>
                    <div>
                      <label class="label">Date Approved</label>
                      <input
                        type="datetime-local"
                        class="input-sm"
                        value={supervisorSignedDate()}
                        onInput={(e) => setSupervisorSignedDate(e.currentTarget.value)}
                      />
                    </div>
                  </div>
                  <div>
                    <label class="label">Approval Notes</label>
                    <textarea
                      class="textarea text-sm"
                      rows={2}
                      placeholder="Any notes regarding approval..."
                      value={approvalNotes()}
                      onInput={(e) => setApprovalNotes(e.currentTarget.value)}
                    />
                  </div>
                </div>
                
                {/* Digital Signature Confirmation */}
                <label class="flex items-start gap-3 p-4 bg-accent/5 border-2 border-accent/20 rounded-xl cursor-pointer hover:bg-accent/10 hover:border-accent/30 transition-all">
                  <div class={`w-5 h-5 rounded-md border-2 flex items-center justify-center mt-0.5 transition-colors ${
                    digitalSignatureConfirmed() ? 'bg-accent border-accent' : 'border-accent/50'
                  }`}>
                    <Show when={digitalSignatureConfirmed()}>
                      <span class="text-white text-xs font-bold">✓</span>
                    </Show>
                  </div>
                  <input
                    type="checkbox"
                    class="sr-only"
                    checked={digitalSignatureConfirmed()}
                    onChange={(e) => setDigitalSignatureConfirmed(e.currentTarget.checked)}
                  />
                  <div>
                    <span class="text-sm font-medium text-txt">I confirm this report is accurate and complete</span>
                    <p class="text-xs text-txt/50 mt-1">
                      By checking this box, I certify that all information contained in this forensic report 
                      is true and accurate to the best of my knowledge.
                    </p>
                  </div>
                </label>
                
                {/* Signature Status */}
                <div class="flex items-center justify-center gap-6 py-3 px-4 bg-surface/30 rounded-xl">
                  <div class={`flex items-center gap-2 text-sm ${examinerSignature() ? 'text-success' : 'text-txt/30'}`}>
                    {examinerSignature() ? <HiOutlineCheckCircle class="w-5 h-5" /> : <HiOutlineXCircle class="w-5 h-5" />}
                    <span>Examiner</span>
                  </div>
                  <div class="w-px h-4 bg-border/50" />
                  <div class={`flex items-center gap-2 text-sm ${supervisorSignature() ? 'text-success' : 'text-txt/30'}`}>
                    {supervisorSignature() ? <HiOutlineCheckCircle class="w-5 h-5" /> : <HiOutlineXCircle class="w-5 h-5" />}
                    <span>Supervisor</span>
                  </div>
                  <div class="w-px h-4 bg-border/50" />
                  <div class={`flex items-center gap-2 text-sm ${digitalSignatureConfirmed() ? 'text-success' : 'text-txt/30'}`}>
                    {digitalSignatureConfirmed() ? <HiOutlineCheckCircle class="w-5 h-5" /> : <HiOutlineXCircle class="w-5 h-5" />}
                    <span>Certified</span>
                  </div>
                </div>
              </div>
              
              {/* Export Button */}
              <div class="pt-5 border-t border-border/30">
                <button
                  class="w-full flex items-center justify-center gap-2 px-4 py-3 rounded-xl text-sm font-semibold transition-all duration-200 bg-accent text-white hover:bg-accent/90 shadow-lg shadow-accent/25 disabled:opacity-50 disabled:cursor-not-allowed disabled:shadow-none"
                  onClick={exportReport}
                  disabled={exporting() || !caseInfo().case_number || !examiner().name}
                >
                  <HiOutlineArrowUpTray class="w-5 h-5" />
                  {exporting() ? "Exporting..." : `Export as ${outputFormats().find(f => f.format === selectedFormat())?.name || selectedFormat()}`}
                </button>
                
                <Show when={!caseInfo().case_number || !examiner().name}>
                  <p class="text-sm text-warning flex items-center justify-center gap-2 mt-3">
                    <HiOutlineExclamationTriangle class="w-4 h-4" />
                    Please fill in required fields: Case Number and Examiner Name
                  </p>
                </Show>
              </div>
            </div>
          </Show>
        </div>
        
        {/* Footer navigation - cleaner design */}
        <div class="flex items-center justify-between px-5 py-4 border-t border-border/50 bg-surface/30">
          <button
            class="inline-flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 bg-surface border border-border/50 text-txt/70 hover:text-txt hover:bg-bg-hover disabled:opacity-40 disabled:cursor-not-allowed"
            onClick={prevStep}
            disabled={currentStep() === "case"}
          >
            <span>←</span>
            <span>Previous</span>
          </button>
          
          <div class="flex items-center gap-2">
            <For each={STEPS}>
              {(_step, index) => {
                const stepIndex = () => STEPS.findIndex(s => s.id === currentStep());
                return (
                  <div class={`w-2 h-2 rounded-full transition-colors ${
                    index() === stepIndex() ? 'bg-accent' :
                    index() < stepIndex() ? 'bg-accent/40' : 'bg-border'
                  }`} />
                );
              }}
            </For>
          </div>
          
          <Show when={currentStep() !== "export"}>
            <button
              class="inline-flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 bg-accent text-white hover:bg-accent/90 shadow-sm shadow-accent/25"
              onClick={nextStep}
            >
              <span>Next</span>
              <span>→</span>
            </button>
          </Show>
          
          <Show when={currentStep() === "export"}>
            <button
              class="inline-flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 bg-surface border border-border/50 text-txt/70 hover:text-txt hover:bg-bg-hover"
              onClick={props.onClose}
            >
              Close
            </button>
          </Show>
        </div>
      </div>
    </div>
  );
}

// Helper functions
function detectEvidenceType(file: DiscoveredFile, _info?: ContainerInfo): EvidenceType {
  const name = file.filename.toLowerCase();
  const type = file.container_type.toLowerCase();
  
  if (type.includes("ufed") || type.includes("cellebrite")) return "MobilePhone";
  if (name.includes("tablet") || name.includes("ipad")) return "Tablet";
  if (name.includes("usb") || name.includes("thumb")) return "UsbDrive";
  if (name.includes("external")) return "ExternalDrive";
  if (name.includes("ssd")) return "SSD";
  if (name.includes("sd") || name.includes("memory")) return "MemoryCard";
  if (name.includes("laptop")) return "Laptop";
  if (name.includes("computer") || name.includes("desktop")) return "Computer";
  if (type.includes("e01") || type.includes("ad1") || type.includes("l01")) return "ForensicImage";
  if (name.includes("dvd") || name.includes("cd") || name.includes("iso")) return "OpticalDisc";
  if (name.includes("pcap") || name.includes("network")) return "NetworkCapture";
  if (name.includes("cloud") || name.includes("onedrive") || name.includes("gdrive")) return "CloudStorage";
  
  return "HardDrive";
}
