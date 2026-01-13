// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Report Templates - Predefined report templates for different use cases
 */

import type { Classification } from "./types";

// =============================================================================
// TEMPLATE TYPES
// =============================================================================

export type ReportTemplateType = 
  | "law_enforcement" 
  | "corporate" 
  | "academic" 
  | "minimal" 
  | "custom";

export interface ReportTemplateSections {
  executiveSummary: boolean;
  scope: boolean;
  methodology: boolean;
  chainOfCustody: boolean;
  timeline: boolean;
  conclusions: boolean;
  appendices: boolean;
}

export interface ReportTemplate {
  id: ReportTemplateType;
  name: string;
  description: string;
  icon: string;
  defaultClassification: Classification;
  defaultSections: ReportTemplateSections;
  defaultMethodology: string;
  defaultScope: string;
  requiredFields: string[];
}

// =============================================================================
// TEMPLATE DEFINITIONS
// =============================================================================

export const REPORT_TEMPLATES: ReportTemplate[] = [
  {
    id: "law_enforcement",
    name: "Law Enforcement",
    description: "Comprehensive template for criminal investigations with chain of custody and court-ready formatting",
    icon: "🛡️",
    defaultClassification: "LawEnforcementSensitive",
    defaultSections: {
      executiveSummary: true,
      scope: true,
      methodology: true,
      chainOfCustody: true,
      timeline: true,
      conclusions: true,
      appendices: true,
    },
    defaultMethodology: `This examination was conducted in accordance with accepted forensic procedures and practices. The following methodology was employed:

1. **Evidence Intake**: All evidence was received, documented, and secured following chain of custody protocols.

2. **Forensic Imaging**: Bit-for-bit forensic images were created using validated forensic tools. Hash values were calculated to verify image integrity.

3. **Analysis**: The forensic images were analyzed using industry-standard forensic software. All artifacts were documented with their source locations and timestamps.

4. **Verification**: All findings were verified through multiple methods where possible. Hash values were re-calculated to ensure data integrity throughout the examination.

5. **Documentation**: All actions taken during the examination were documented in examination notes and are reflected in this report.`,
    defaultScope: `This examination was requested to analyze digital evidence related to the investigation. The scope includes:

• Analysis of forensic images of digital storage media
• Recovery and examination of files and artifacts
• Timeline reconstruction of relevant events
• Hash verification of evidence integrity
• Documentation of findings for potential court presentation`,
    requiredFields: ["case_number", "examiner_name", "badge_number", "agency"],
  },
  {
    id: "corporate",
    name: "Corporate/Internal",
    description: "Business-focused template for internal investigations and compliance audits",
    icon: "🏢",
    defaultClassification: "Confidential",
    defaultSections: {
      executiveSummary: true,
      scope: true,
      methodology: true,
      chainOfCustody: false,
      timeline: true,
      conclusions: true,
      appendices: false,
    },
    defaultMethodology: `This investigation followed corporate digital forensics and incident response procedures:

1. **Evidence Collection**: Digital evidence was collected following corporate data handling policies and legal hold requirements.

2. **Analysis**: Evidence was analyzed using approved forensic tools and techniques to identify relevant artifacts.

3. **Documentation**: All findings were documented with supporting evidence references.

4. **Review**: Findings were reviewed for accuracy and completeness before inclusion in this report.`,
    defaultScope: `This examination was conducted as part of an internal investigation. The scope includes analysis of relevant digital evidence to determine facts and findings related to the matter under investigation.`,
    requiredFields: ["case_number", "examiner_name", "organization"],
  },
  {
    id: "academic",
    name: "Academic/Research",
    description: "Research-oriented template with detailed methodology and reproducibility focus",
    icon: "🎓",
    defaultClassification: "Internal",
    defaultSections: {
      executiveSummary: true,
      scope: true,
      methodology: true,
      chainOfCustody: false,
      timeline: false,
      conclusions: true,
      appendices: true,
    },
    defaultMethodology: `This analysis employed the following research methodology:

1. **Data Acquisition**: Evidence was acquired using forensically sound methods to preserve data integrity.

2. **Tool Validation**: All analysis tools were validated for accuracy and reliability.

3. **Analysis Protocol**: A systematic analysis protocol was followed to ensure reproducibility.

4. **Peer Review**: Findings were subject to peer review prior to final documentation.`,
    defaultScope: `This examination was conducted for research and educational purposes, analyzing digital evidence using forensic techniques and documenting the methodology for reproducibility.`,
    requiredFields: ["case_number", "examiner_name"],
  },
  {
    id: "minimal",
    name: "Minimal/Quick",
    description: "Streamlined template for quick assessments with essential information only",
    icon: "⚡",
    defaultClassification: "Internal",
    defaultSections: {
      executiveSummary: true,
      scope: false,
      methodology: false,
      chainOfCustody: false,
      timeline: false,
      conclusions: true,
      appendices: false,
    },
    defaultMethodology: "",
    defaultScope: "",
    requiredFields: ["case_number"],
  },
  {
    id: "custom",
    name: "Custom",
    description: "Start from scratch with a blank template",
    icon: "✏️",
    defaultClassification: "Internal",
    defaultSections: {
      executiveSummary: false,
      scope: false,
      methodology: false,
      chainOfCustody: false,
      timeline: false,
      conclusions: false,
      appendices: false,
    },
    defaultMethodology: "",
    defaultScope: "",
    requiredFields: [],
  },
];

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/** Get template by ID */
export function getTemplateById(id: ReportTemplateType): ReportTemplate | undefined {
  return REPORT_TEMPLATES.find(t => t.id === id);
}

/** Get default template */
export function getDefaultTemplate(): ReportTemplate {
  return REPORT_TEMPLATES[0]; // law_enforcement
}
