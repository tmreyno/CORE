// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Section router — dispatches a ViewerMetadataSection to the correct
 * viewer-specific section component based on its `kind` discriminant.
 */

import type { ViewerMetadataSection } from "../../types/viewerMetadata";
import { ExifSection } from "./ExifSection";
import { RegistrySection } from "./RegistrySection";
import { DatabaseSection } from "./DatabaseSection";
import { BinarySection } from "./BinarySection";
import { EmailSection } from "./EmailSection";
import { PlistSection } from "./PlistSection";
import { DocumentSection } from "./DocumentSection";
import { SpreadsheetSection } from "./SpreadsheetSection";
import { OfficeSection } from "./OfficeSection";
import { ArchiveSection } from "./ArchiveSection";

export function MetadataSectionRenderer(props: {
  section: ViewerMetadataSection;
}) {
  switch (props.section.kind) {
    case "exif":
      return <ExifSection data={props.section} />;
    case "registry":
      return <RegistrySection data={props.section} />;
    case "database":
      return <DatabaseSection data={props.section} />;
    case "binary":
      return <BinarySection data={props.section} />;
    case "email":
      return <EmailSection data={props.section} />;
    case "plist":
      return <PlistSection data={props.section} />;
    case "document":
      return <DocumentSection data={props.section} />;
    case "spreadsheet":
      return <SpreadsheetSection data={props.section} />;
    case "office":
      return <OfficeSection data={props.section} />;
    case "archive":
      return <ArchiveSection data={props.section} />;
    default:
      return null;
  }
}
