// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

export { HexViewer } from "./HexViewerComponent";
export { HexLine } from "./HexLine";
export { HexToolbar } from "./HexToolbar";
export { useHexData } from "./useHexData";
export type { HexViewerProps } from "./types";
export type { UseHexDataOptions } from "./useHexData";

// Re-export viewer types for backward compatibility
export type { FileChunk, HeaderRegion, MetadataField, ParsedMetadata, FileTypeInfo } from "../../types";
