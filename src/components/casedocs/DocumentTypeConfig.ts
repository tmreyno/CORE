// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Document Type Configuration
 * 
 * Styling and icon configuration for different case document types
 * and file format icons with distinct colors.
 */

import {
  HiOutlineClipboardDocumentList,
  HiOutlineDocumentCheck,
  HiOutlineDocumentText,
  HiOutlineClipboard,
  HiOutlineMagnifyingGlass,
  HiOutlineDocument,
  HiOutlineTableCells,
  HiOutlinePhoto,
  HiOutlineGlobeAlt,
  HiOutlineEnvelope,
  HiOutlineCodeBracket,
  HiOutlineRectangleGroup,
  HiOutlineArchiveBox,
  HiOutlineCircleStack,
  HiOutlineFilm,
  HiOutlineMusicalNote,
} from "../icons";
import type { CaseDocumentType } from "../../types";
import type { Component } from "solid-js";

export const documentTypeColors: Record<CaseDocumentType, string> = {
  ChainOfCustody: "text-accent bg-accent/10 border-accent/30",
  EvidenceIntake: "text-green-400 bg-green-500/10 border-green-500/30",
  CaseNotes: "text-yellow-400 bg-yellow-500/10 border-yellow-500/30",
  EvidenceReceipt: "text-purple-400 bg-purple-500/10 border-purple-500/30",
  LabRequest: "text-blue-400 bg-blue-500/10 border-blue-500/30",
  ExternalReport: "text-orange-400 bg-orange-500/10 border-orange-500/30",
  Other: "text-txt-secondary bg-bg-muted/10 border-border-subtle/30",
};

export const documentTypeIcons: Record<CaseDocumentType, typeof HiOutlineClipboard> = {
  ChainOfCustody: HiOutlineClipboardDocumentList,
  EvidenceIntake: HiOutlineDocumentCheck,
  CaseNotes: HiOutlineDocumentText,
  EvidenceReceipt: HiOutlineClipboard,
  LabRequest: HiOutlineMagnifyingGlass,
  ExternalReport: HiOutlineDocumentText,
  Other: HiOutlineDocumentText,
};

// =============================================================================
// File Format Icons & Colors
// =============================================================================

interface FormatIconConfig {
  icon: Component<{ class?: string }>;
  color: string;
}

const defaultFormatConfig: FormatIconConfig = {
  icon: HiOutlineDocument,
  color: "text-txt-muted",
};

/**
 * Map of file format strings to icon + color pairs.
 * Format strings are matched case-insensitively in getDocumentFormatIcon().
 */
const formatIconMap: Record<string, FormatIconConfig> = {
  // Documents
  pdf:   { icon: HiOutlineDocumentText,    color: "text-red-400" },
  doc:   { icon: HiOutlineDocumentText,    color: "text-blue-400" },
  docx:  { icon: HiOutlineDocumentText,    color: "text-blue-400" },
  rtf:   { icon: HiOutlineDocumentText,    color: "text-blue-300" },
  odt:   { icon: HiOutlineDocumentText,    color: "text-blue-300" },
  txt:   { icon: HiOutlineDocument,        color: "text-txt-secondary" },
  md:    { icon: HiOutlineDocument,        color: "text-txt-secondary" },

  // Spreadsheets
  xls:   { icon: HiOutlineTableCells,      color: "text-green-400" },
  xlsx:  { icon: HiOutlineTableCells,      color: "text-green-400" },
  csv:   { icon: HiOutlineTableCells,      color: "text-green-300" },
  ods:   { icon: HiOutlineTableCells,      color: "text-green-300" },

  // Presentations
  ppt:   { icon: HiOutlineRectangleGroup,  color: "text-orange-400" },
  pptx:  { icon: HiOutlineRectangleGroup,  color: "text-orange-400" },
  odp:   { icon: HiOutlineRectangleGroup,  color: "text-orange-300" },

  // Images
  jpg:   { icon: HiOutlinePhoto,           color: "text-purple-400" },
  jpeg:  { icon: HiOutlinePhoto,           color: "text-purple-400" },
  png:   { icon: HiOutlinePhoto,           color: "text-purple-400" },
  gif:   { icon: HiOutlinePhoto,           color: "text-purple-300" },
  bmp:   { icon: HiOutlinePhoto,           color: "text-purple-300" },
  tiff:  { icon: HiOutlinePhoto,           color: "text-purple-300" },
  svg:   { icon: HiOutlinePhoto,           color: "text-pink-400" },
  webp:  { icon: HiOutlinePhoto,           color: "text-purple-300" },

  // Web / Code
  html:  { icon: HiOutlineGlobeAlt,        color: "text-orange-400" },
  htm:   { icon: HiOutlineGlobeAlt,        color: "text-orange-400" },
  xml:   { icon: HiOutlineCodeBracket,     color: "text-yellow-400" },
  json:  { icon: HiOutlineCodeBracket,     color: "text-yellow-300" },
  css:   { icon: HiOutlineCodeBracket,     color: "text-blue-300" },

  // Email
  eml:   { icon: HiOutlineEnvelope,        color: "text-cyan-400" },
  msg:   { icon: HiOutlineEnvelope,        color: "text-cyan-400" },
  mbox:  { icon: HiOutlineEnvelope,        color: "text-cyan-300" },

  // Archives
  zip:   { icon: HiOutlineArchiveBox,      color: "text-amber-400" },
  "7z":  { icon: HiOutlineArchiveBox,      color: "text-amber-400" },
  rar:   { icon: HiOutlineArchiveBox,      color: "text-amber-400" },
  tar:   { icon: HiOutlineArchiveBox,      color: "text-amber-300" },
  gz:    { icon: HiOutlineArchiveBox,      color: "text-amber-300" },

  // Database
  db:    { icon: HiOutlineCircleStack,     color: "text-emerald-400" },
  sqlite:{ icon: HiOutlineCircleStack,     color: "text-emerald-400" },

  // Media
  mp4:   { icon: HiOutlineFilm,            color: "text-indigo-400" },
  avi:   { icon: HiOutlineFilm,            color: "text-indigo-400" },
  mkv:   { icon: HiOutlineFilm,            color: "text-indigo-400" },
  mov:   { icon: HiOutlineFilm,            color: "text-indigo-300" },
  mp3:   { icon: HiOutlineMusicalNote,     color: "text-pink-400" },
  wav:   { icon: HiOutlineMusicalNote,     color: "text-pink-400" },
  flac:  { icon: HiOutlineMusicalNote,     color: "text-pink-300" },
};

/**
 * Get the icon component and color class for a document format string.
 * Returns a distinct icon + color combination for visual differentiation.
 */
export function getDocumentFormatIcon(format: string): FormatIconConfig {
  const key = format.toLowerCase().replace(/^\./, "");
  return formatIconMap[key] ?? defaultFormatConfig;
}
