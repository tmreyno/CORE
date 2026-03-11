// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * viewerMetadata barrel — re-exports everything needed by the parent
 * ViewerMetadataPanel.tsx and external consumers.
 */

export { CollapsibleGroup, MetadataRow, OptionalMetadataRow, SectionHeader, SummaryRow, StatusBadge } from "./shared";
export { FileInfoTab } from "./FileInfoTab";
export { MetadataSectionRenderer } from "./MetadataSectionRenderer";
export { ExifSection } from "./ExifSection";
export { RegistrySection } from "./RegistrySection";
export { DatabaseSection } from "./DatabaseSection";
export { BinarySection } from "./BinarySection";
export { EmailSection } from "./EmailSection";
export { PlistSection } from "./PlistSection";
export { DocumentSection } from "./DocumentSection";
export { SpreadsheetSection } from "./SpreadsheetSection";
export { ArchiveSection } from "./ArchiveSection";
export { OfficeSection } from "./OfficeSection";
