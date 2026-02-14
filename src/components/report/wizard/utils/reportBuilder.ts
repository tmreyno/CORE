// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { Accessor } from "solid-js";
import type {
  ForensicReport,
  ReportMetadata,
  CaseInfo,
  ExaminerInfo,
  Finding,
  CustodyRecord,
  EvidenceItem,
  SignatureRecord,
  HashAlgorithmType,
  TimelineEvent,
} from "../../types";
import type { EvidenceGroup } from "../types";
import type { ContainerInfo } from "../../../../types";
import type { FileHashInfo } from "../../../../types/hash";
import { getPreference } from "../../../preferences";
import { detectEvidenceType } from "./evidenceUtils";

export interface ReportBuilderParams {
  metadata: Accessor<ReportMetadata>;
  caseInfo: Accessor<CaseInfo>;
  examiner: Accessor<ExaminerInfo>;
  executiveSummary: Accessor<string>;
  scope: Accessor<string>;
  methodology: Accessor<string>;
  conclusions: Accessor<string>;
  findings: Accessor<Finding[]>;
  chainOfCustody: Accessor<CustodyRecord[]>;
  groupedEvidence: Accessor<EvidenceGroup[]>;
  selectedEvidence: Accessor<Set<string>>;
  examinerSignature: Accessor<string>;
  examinerSignedDate: Accessor<string>;
  supervisorName: Accessor<string>;
  supervisorSignature: Accessor<string>;
  supervisorSignedDate: Accessor<string>;
  digitalSignatureConfirmed: Accessor<boolean>;
  approvalNotes: Accessor<string>;
  fileInfoMap: Map<string, ContainerInfo>;
  fileHashMap: Map<string, FileHashInfo>;
  /** Timeline events derived from project activity log */
  projectTimeline?: Accessor<TimelineEvent[]>;
}

export function buildForensicReport(params: ReportBuilderParams): ForensicReport {
  const {
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
    fileInfoMap,
    fileHashMap,
    projectTimeline,
  } = params;

  // Get report preferences
  const includeHashes = getPreference("includeHashesInReports");
  const includeTimestamps = getPreference("includeTimestampsInReports");
  const includeMetadata = getPreference("includeMetadataInReports");

  // Build evidence items from selected files
  const evidenceItems: EvidenceItem[] = [];
  const groups = groupedEvidence();

  for (const group of groups) {
    if (!selectedEvidence().has(group.primaryFile.path)) continue;

    const info = fileInfoMap.get(group.primaryFile.path);
    const hashInfo = fileHashMap.get(group.primaryFile.path);

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
      acquisition_date: includeTimestamps
        ? (ewfInfo?.acquiry_date ?? ad1Info?.companion_log?.acquisition_date ?? undefined)
        : undefined,
      acquisition_method: includeMetadata ? (ewfInfo?.description ?? undefined) : undefined,
      acquisition_tool: undefined, // Not available in current EWF schema
      // Only include hashes if preference enabled
      acquisition_hashes:
        includeHashes && hashInfo
          ? [
              {
                item: group.primaryFile.filename,
                algorithm: hashInfo.algorithm as HashAlgorithmType,
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
    timeline: projectTimeline ? projectTimeline() : [],
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
}
