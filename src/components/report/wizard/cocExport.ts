// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Export functions for evidence collections.
 *
 * Supports PDF (via generate_report command), CSV, XLSX, and HTML
 * (via export_evidence_collection command).
 */

import { invoke } from "@tauri-apps/api/core";
import { logger } from "../../../utils/logger";
import type { ForensicReport } from "../types";
import { loadEvidenceCollectionById } from "./cocPersistence";

const log = logger.scope("CocExport");

/**
 * Export an evidence collection as PDF.
 * Loads the collection, builds a minimal ForensicReport, shows a save dialog,
 * and invokes the generate_report Tauri command.
 *
 * @returns The output file path on success, or null if cancelled/failed.
 */
export async function exportEvidenceCollectionPdf(
  collectionId: string,
  caseNumber?: string,
): Promise<string | null> {
  try {
    const result = await loadEvidenceCollectionById(collectionId);
    if (!result) {
      log.warn("Collection not found for PDF export:", collectionId);
      return null;
    }

    const { data } = result;

    // Build minimal ForensicReport for the PDF generator
    const report: ForensicReport = {
      metadata: {
        title: "Evidence Collection Report",
        report_number: caseNumber || "EC-001",
        version: "1.0",
        classification: "LawEnforcementSensitive",
        generated_at: new Date().toISOString(),
        generated_by: "CORE-FFX",
      },
      case_info: {
        case_number: caseNumber || "",
        case_name: "",
        agency: "",
        requestor: data.collecting_officer || "",
      },
      examiner: {
        name: data.collecting_officer || "",
        title: "",
        organization: "",
        phone: "",
        email: "",
        certifications: [],
      },
      report_type: "evidence_collection",
      evidence_items: [],
      chain_of_custody: [],
      findings: [],
      timeline: [],
      hash_records: [],
      tools: [],
      appendices: [],
      evidence_collection: data,
    };

    // Show save dialog
    const { save } = await import("@tauri-apps/plugin-dialog");
    const defaultName = `Evidence_Collection_${caseNumber || "report"}_${new Date().toISOString().split("T")[0]}.pdf`;
    const path = await save({
      defaultPath: defaultName,
      filters: [{ name: "PDF", extensions: ["pdf"] }],
    });

    if (!path) return null;

    // Generate PDF
    const outputPath = await invoke<string>("generate_report", {
      report,
      format: "Pdf",
      outputPath: path,
    });

    log.info("Evidence collection PDF exported:", outputPath);
    return outputPath;
  } catch (e) {
    log.error("Failed to export evidence collection PDF:", e);
    return null;
  }
}

/** Supported evidence collection export formats */
export type EvidenceExportFormat = "pdf" | "csv" | "xlsx" | "html";

const FORMAT_LABELS: Record<EvidenceExportFormat, { name: string; extension: string }> = {
  pdf: { name: "PDF Document", extension: "pdf" },
  csv: { name: "CSV Spreadsheet", extension: "csv" },
  xlsx: { name: "Excel Spreadsheet", extension: "xlsx" },
  html: { name: "HTML Report", extension: "html" },
};

/**
 * Export an evidence collection in the specified format.
 * Loads the collection, shows a save dialog, and invokes the
 * export_evidence_collection Tauri command.
 *
 * @returns The output file path on success, or null if cancelled/failed.
 */
export async function exportEvidenceCollection(
  collectionId: string,
  format: EvidenceExportFormat,
  caseNumber?: string,
): Promise<string | null> {
  try {
    const result = await loadEvidenceCollectionById(collectionId);
    if (!result) {
      log.warn("Collection not found for export:", collectionId);
      return null;
    }

    const { data } = result;
    const info = FORMAT_LABELS[format];
    const { save } = await import("@tauri-apps/plugin-dialog");
    const dateSuffix = new Date().toISOString().split("T")[0];
    const defaultName = `Evidence_Collection_${caseNumber || "report"}_${dateSuffix}.${info.extension}`;

    const path = await save({
      defaultPath: defaultName,
      filters: [{ name: info.name, extensions: [info.extension] }],
    });
    if (!path) return null;

    const outputPath = await invoke<string>("export_evidence_collection", {
      data,
      caseNumber: caseNumber || "",
      format,
      outputPath: path,
    });

    log.info(`Evidence collection ${format.toUpperCase()} exported:`, outputPath);
    return outputPath;
  } catch (e) {
    log.error(`Failed to export evidence collection as ${format}:`, e);
    return null;
  }
}
