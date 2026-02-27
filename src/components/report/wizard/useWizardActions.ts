// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useWizardActions — Action functions for the Report Wizard.
 *
 * Contains navigation, preset application, preview generation,
 * report export, report building, and project-data seeding lifecycle.
 * Must be called within a component's reactive scope (WizardProvider).
 */

import { onMount } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import type {
  CaseInfo,
  CustodyRecord,
  Finding,
  ForensicReport,
  OutputFormat,
} from "../types";
import { REPORT_PRESETS, type ReportPreset } from "../constants";
import { getPreference } from "../../preferences";
import type { WizardStep, ReportWizardProps } from "./types";
import type { WizardState } from "./useWizardState";
import { buildForensicReport } from "./utils/reportBuilder";
import { dbSync } from "../../../hooks/project/useProjectDbSync";
import { persistCocItemsToDb, loadCocItemsFromDb } from "./cocDbSync";
import { generateId, nowISO } from "../../../types/project";
import type { ProjectReportRecord } from "../../../types/project";
import { logger } from "../../../utils/logger";

const log = logger.scope("ReportWizard");

// =============================================================================
// ACTIONS INTERFACE
// =============================================================================

export interface WizardActions {
  goToStep: (step: WizardStep) => void;
  nextStep: () => void;
  prevStep: () => void;
  applyPreset: (presetId: ReportPreset) => void;
  generatePreview: () => Promise<void>;
  exportReport: () => Promise<void>;
  buildReport: () => ForensicReport;
}

// =============================================================================
// ACTIONS CREATION
// =============================================================================

export function useWizardActions(
  state: WizardState,
  props: ReportWizardProps,
): WizardActions {

  // ==========================================================================
  // NAVIGATION
  // ==========================================================================

  const goToStep = (step: WizardStep) => state.setCurrentStep(step);

  const nextStep = () => {
    const steps = state.activeSteps();
    const idx = steps.findIndex((s) => s.id === state.currentStep());
    if (idx < steps.length - 1) {
      state.setCurrentStep(steps[idx + 1].id);
    }
  };

  const prevStep = () => {
    const steps = state.activeSteps();
    const idx = steps.findIndex((s) => s.id === state.currentStep());
    if (idx > 0) {
      state.setCurrentStep(steps[idx - 1].id);
    }
  };

  // ==========================================================================
  // PRESET
  // ==========================================================================

  const applyPreset = (presetId: ReportPreset) => {
    state.setSelectedPreset(presetId);
    const preset = REPORT_PRESETS.find((p) => p.id === presetId);
    if (preset) {
      state.setMetadata((prev) => ({ ...prev, classification: preset.defaultClassification }));
      state.setEnabledSections(preset.defaultSections);
    }
  };

  // ==========================================================================
  // PREVIEW
  // ==========================================================================

  const generatePreview = async () => {
    state.setPreviewLoading(true);
    try {
      const report = buildReport();
      const html = await invoke<string>("preview_report", { report });
      state.setPreviewHtml(html);
    } catch (e) {
      log.error("Preview failed:", e);
      state.setPreviewHtml(`<div style="color: red; padding: 20px;">Preview failed: ${e}</div>`);
    } finally {
      state.setPreviewLoading(false);
    }
  };

  // ==========================================================================
  // EXPORT
  // ==========================================================================

  const exportReport = async () => {
    state.setExporting(true);
    state.setExportError(null);

    try {
      const report = buildReport();
      const format = state.outputFormats().find((f) => f.format === state.selectedFormat());

      // Import save dialog dynamically
      const { save } = await import("@tauri-apps/plugin-dialog");

      // Open save dialog
      const path = await save({
        title: "Save Report",
        defaultPath: `${report.metadata.report_number}.${format?.extension || "pdf"}`,
        filters: format ? [{ name: format.name, extensions: [format.extension] }] : [],
      });

      if (!path) {
        state.setExporting(false);
        return;
      }

      // Generate report
      const outputPath = await invoke<string>("generate_report", {
        report,
        format: state.selectedFormat(),
        outputPath: path,
      });

      // Write-through to .ffxdb for report history tracking
      const reportRecord: ProjectReportRecord = {
        id: generateId(),
        title: report.metadata.title || "Forensic Report",
        report_type: "detailed",
        format: state.selectedFormat().toLowerCase() as ProjectReportRecord["format"],
        output_path: outputPath,
        generated_at: nowISO(),
        generated_by: report.examiner?.name || "unknown",
        status: "completed",
      };
      dbSync.insertReport(reportRecord);

      // Persist COC data to .ffxdb
      const rt = state.reportType();
      if (rt === "chain_of_custody" && state.cocItems().length > 0) {
        await persistCocItemsToDb(state.cocItems());
      }

      props.onGenerated?.(outputPath, state.selectedFormat());
      props.onClose();
    } catch (e) {
      log.error("Export failed:", e);
      state.setExportError(String(e));

      // Record failed export in .ffxdb for audit trail
      const failedRecord: ProjectReportRecord = {
        id: generateId(),
        title: state.metadata().title || "Forensic Report",
        report_type: "detailed",
        format: state.selectedFormat().toLowerCase() as ProjectReportRecord["format"],
        generated_at: nowISO(),
        generated_by: state.examiner().name || "unknown",
        status: "failed",
        error: String(e),
      };
      dbSync.insertReport(failedRecord);
    } finally {
      state.setExporting(false);
    }
  };

  // ==========================================================================
  // REPORT BUILDING
  // ==========================================================================

  const buildReport = (): ForensicReport => {
    const base = buildForensicReport({
      metadata: state.metadata,
      caseInfo: state.caseInfo,
      examiner: state.examiner,
      executiveSummary: state.executiveSummary,
      scope: state.scope,
      methodology: state.methodology,
      conclusions: state.conclusions,
      findings: state.findings,
      chainOfCustody: state.chainOfCustody,
      groupedEvidence: state.groupedEvidence,
      selectedEvidence: state.selectedEvidence,
      examinerSignature: state.examinerSignature,
      examinerSignedDate: state.examinerSignedDate,
      supervisorName: state.supervisorName,
      supervisorSignature: state.supervisorSignature,
      supervisorSignedDate: state.supervisorSignedDate,
      digitalSignatureConfirmed: state.digitalSignatureConfirmed,
      approvalNotes: state.approvalNotes,
      fileInfoMap: props.fileInfoMap,
      fileHashMap: props.fileHashMap,
      projectTimeline: state.projectTimeline,
    });

    // Attach report type and type-specific data
    base.report_type = state.reportType();
    const rt = state.reportType();
    if (rt === "chain_of_custody") {
      base.coc_items = state.cocItems();
    } else if (rt === "investigative_activity") {
      base.iar_data = { summary: state.iarSummary(), entries: state.iarEntries() };
    } else if (rt === "user_activity") {
      base.user_activity = state.userActivityData();
    } else if (rt === "timeline") {
      base.timeline_report = state.timelineReportData();
    }

    return base;
  };

  // ==========================================================================
  // LIFECYCLE: PROJECT DATA SEEDING
  // ==========================================================================

  onMount(() => {
    // --- Case Info: seed from project data ---
    const caseUpdates: Partial<CaseInfo> = {};

    // Project name → case_name (if not already set)
    if (props.projectName && !state.caseInfo().case_name) {
      caseUpdates.case_name = props.projectName;
    }

    // Project description → case description
    if (props.projectDescription && !state.caseInfo().description) {
      caseUpdates.description = props.projectDescription;
    }

    // Extract case number from case documents cache (first match wins)
    if (!state.caseInfo().case_number || state.caseInfo().case_number === (getPreference("caseNumberPrefix") || "")) {
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
    if (prefAgency && !state.caseInfo().agency) {
      caseUpdates.agency = prefAgency;
    }

    // Exam dates from sessions (earliest start, latest end)
    const sessions = props.sessions;
    if (sessions && sessions.length > 0) {
      if (!state.caseInfo().exam_start_date) {
        const earliest = sessions
          .map(s => s.started_at)
          .filter(Boolean)
          .sort()[0];
        if (earliest) {
          caseUpdates.exam_start_date = earliest.split("T")[0]; // date-only
        }
      }
      if (!state.caseInfo().exam_end_date) {
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
      state.setCaseInfo(prev => ({ ...prev, ...caseUpdates }));
    }

    // --- Metadata: seed generated_by from examiner name ---
    if (state.examiner().name) {
      state.setMetadata(prev => ({
        ...prev,
        generated_by: `${state.examiner().name} via FFX`,
      }));
    }

    // --- Evidence: auto-select all items ---
    if (props.files.length > 0 && state.selectedEvidence().size === 0) {
      const allPrimaries = state.groupedEvidence().map(g => g.primaryFile.path);
      state.setSelectedEvidence(new Set(allPrimaries));
    }

    // --- Chain of custody: seed from project sessions (skip for report types that don't use it) ---
    const rt = state.reportType();
    if (rt !== "chain_of_custody" &&
        sessions && sessions.length > 0 && state.chainOfCustody().length === 0) {
      const custodyFromSessions: CustodyRecord[] = sessions.map(session => ({
        timestamp: session.started_at,
        action: session.ended_at ? "Examination session" : "Active session",
        handler: session.user || state.examiner().name || "",
        location: session.hostname || undefined,
        notes: session.summary || 
          (session.duration_seconds 
            ? `Duration: ${Math.round(session.duration_seconds / 60)} min (${session.app_version})`
            : `App version: ${session.app_version}`),
      }));
      state.setChainOfCustody(custodyFromSessions);
    }

    // --- Findings: seed from bookmarks and notes ---
    if (state.findings().length === 0) {
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
        state.setFindings(autoFindings);
      }
    }

    log.info("Project data seeding complete", {
      caseUpdates: Object.keys(caseUpdates),
      evidenceAutoSelected: state.selectedEvidence().size,
      custodyRecords: state.chainOfCustody().length,
      findingsSeeded: state.findings().length,
    });
  });

  // ==========================================================================
  // LIFECYCLE: LOAD OUTPUT FORMATS
  // ==========================================================================

  onMount(async () => {
    try {
      const formats = await invoke<OutputFormat[]>("get_output_formats");
      state.setOutputFormats(formats);
    } catch (e) {
      log.error("Failed to get output formats:", e);
      // Set defaults
      state.setOutputFormats([
        { format: "Pdf", name: "PDF", description: "Portable Document Format", extension: "pdf", supported: true },
        { format: "Html", name: "HTML", description: "Web page format", extension: "html", supported: true },
        { format: "Markdown", name: "Markdown", description: "Markdown text format", extension: "md", supported: true },
      ]);
    }
  });

  // ==========================================================================
  // LIFECYCLE: LOAD COC FROM DB
  // ==========================================================================

  onMount(async () => {
    const caseNum = state.caseInfo().case_number || undefined;
    try {
      const dbCocItems = await loadCocItemsFromDb(caseNum);
      if (dbCocItems.length > 0) {
        state.setCocItems(dbCocItems);
        log.info(`Loaded ${dbCocItems.length} COC items from .ffxdb`);
      }
    } catch (e) {
      log.warn("Could not load COC items from DB:", e);
    }
  });

  // ==========================================================================
  // RETURN ACTIONS
  // ==========================================================================

  return {
    goToStep,
    nextStep,
    prevStep,
    applyPreset,
    generatePreview,
    exportReport,
    buildReport,
  };
}
