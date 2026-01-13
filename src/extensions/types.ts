// =============================================================================
// CORE-FFX EXTENSION SYSTEM - Type Definitions
// =============================================================================
// This module defines the interfaces for building extensions/add-ons for CORE-FFX.
// Extensions can add support for new forensic tools, artifact viewers, export
// formats, and more.
// =============================================================================

import type { Component, JSX } from "solid-js";
import type { ProcessedDatabase, ArtifactCategory, ProcessedDbType } from "../types/processed";
import type { ContainerInfo, DiscoveredFile } from "../types";

// =============================================================================
// EXTENSION METADATA
// =============================================================================

/** Semantic version string (e.g., "1.0.0") */
export type SemVer = string;

/** Unique extension identifier (e.g., "com.example.my-extension") */
export type ExtensionId = string;

/** Extension metadata */
export interface ExtensionManifest {
  /** Unique identifier for this extension */
  id: ExtensionId;
  /** Human-readable name */
  name: string;
  /** Version (semver) */
  version: SemVer;
  /** Brief description */
  description: string;
  /** Author name or organization */
  author: string;
  /** License identifier (e.g., "MIT", "GPL-3.0") */
  license?: string;
  /** Homepage URL */
  homepage?: string;
  /** Repository URL */
  repository?: string;
  /** Minimum CORE-FFX version required */
  minAppVersion?: SemVer;
  /** Keywords for discovery */
  keywords?: string[];
  /** Extension category */
  category: ExtensionCategory;
  /** Icon (emoji or URL) */
  icon?: string;
}

/** Extension categories */
export type ExtensionCategory =
  | "database-processor"    // Adds support for new processed database formats
  | "container-parser"      // Adds support for new evidence container formats
  | "artifact-viewer"       // Custom viewers for specific artifact types
  | "export-format"         // New export/report formats
  | "analysis-tool"         // Analysis and reporting tools
  | "integration"           // Third-party integrations
  | "theme"                 // UI themes
  | "utility";              // General utilities

// =============================================================================
// DATABASE PROCESSOR EXTENSION
// =============================================================================
// Adds support for parsing processed databases from forensic tools
// (e.g., new versions of AXIOM, Cellebrite, or entirely new tools)

/** Database processor extension interface */
export interface DatabaseProcessorExtension {
  manifest: ExtensionManifest;
  
  /** Database type identifier */
  dbType: ProcessedDbType | string;
  
  /** File patterns to detect this database type */
  filePatterns: string[];
  
  /** Check if a path contains this database type */
  detect(path: string): Promise<boolean>;
  
  /** Parse database and return metadata */
  parse(path: string): Promise<ProcessedDatabase>;
  
  /** Get artifact categories available in this database */
  getCategories?(db: ProcessedDatabase): Promise<ArtifactCategorySummary[]>;
  
  /** Query artifacts from a category */
  queryArtifacts?(db: ProcessedDatabase, category: string, options?: QueryOptions): Promise<ArtifactResult[]>;
  
  /** Get database-specific case information */
  getCaseInfo?(db: ProcessedDatabase): Promise<Record<string, unknown>>;
}

/** Artifact category with count */
export interface ArtifactCategorySummary {
  name: string;
  category: ArtifactCategory;
  count: number;
  icon?: string;
  description?: string;
}

/** Query options for artifact retrieval */
export interface QueryOptions {
  limit?: number;
  offset?: number;
  sortBy?: string;
  sortOrder?: "asc" | "desc";
  filters?: Record<string, unknown>;
}

/** Generic artifact result */
export interface ArtifactResult {
  id: string;
  type: string;
  category: ArtifactCategory;
  timestamp?: string;
  source?: string;
  data: Record<string, unknown>;
}

// =============================================================================
// CONTAINER PARSER EXTENSION
// =============================================================================
// Adds support for new evidence container formats (beyond E01, AD1, L01, etc.)

/** Container parser extension interface */
export interface ContainerParserExtension {
  manifest: ExtensionManifest;
  
  /** Container type identifier */
  containerType: string;
  
  /** File extensions this parser handles */
  fileExtensions: string[];
  
  /** Magic bytes to detect this container type */
  magicBytes?: Uint8Array;
  
  /** Check if a file is this container type */
  detect(path: string): Promise<boolean>;
  
  /** Parse container and return info */
  parse(path: string): Promise<ContainerInfo>;
  
  /** List files/entries in the container */
  listEntries?(path: string): Promise<ContainerEntry[]>;
  
  /** Extract a specific entry */
  extractEntry?(containerPath: string, entryPath: string, destPath: string): Promise<void>;
  
  /** Read raw bytes from container */
  readBytes?(path: string, offset: number, length: number): Promise<Uint8Array>;
  
  /** Verify container integrity */
  verify?(path: string): Promise<VerificationResult>;
}

/** Entry in a container */
export interface ContainerEntry {
  path: string;
  name: string;
  isDirectory: boolean;
  size: number;
  created?: string;
  modified?: string;
  attributes?: Record<string, unknown>;
}

/** Verification result */
export interface VerificationResult {
  valid: boolean;
  message: string;
  details?: Record<string, unknown>;
}

// =============================================================================
// ARTIFACT VIEWER EXTENSION
// =============================================================================
// Custom viewers for specific artifact types (e.g., chat viewer, timeline viewer)

/** Artifact viewer extension interface */
export interface ArtifactViewerExtension {
  manifest: ExtensionManifest;
  
  /** Artifact types this viewer handles */
  artifactTypes: string[];
  
  /** Categories this viewer supports */
  categories?: ArtifactCategory[];
  
  /** Priority (higher = preferred when multiple viewers match) */
  priority?: number;
  
  /** Check if this viewer can handle the artifact */
  canHandle(artifact: ArtifactResult): boolean;
  
  /** The viewer component */
  Component: Component<ArtifactViewerProps>;
}

/** Props passed to artifact viewer components */
export interface ArtifactViewerProps {
  artifact: ArtifactResult;
  database?: ProcessedDatabase;
  onNavigate?: (artifactId: string) => void;
  onExport?: (artifact: ArtifactResult, format: string) => void;
}

// =============================================================================
// EXPORT FORMAT EXTENSION
// =============================================================================
// Adds new export/report formats (e.g., custom PDF templates, Excel, etc.)

/** Export format extension interface */
export interface ExportFormatExtension {
  manifest: ExtensionManifest;
  
  /** Format identifier */
  formatId: string;
  
  /** Display name */
  formatName: string;
  
  /** File extension */
  fileExtension: string;
  
  /** MIME type */
  mimeType: string;
  
  /** Icon for the format */
  icon?: string;
  
  /** Export data to this format */
  export(data: ExportData, options?: ExportOptions): Promise<Uint8Array | string>;
  
  /** Optional: Render preview */
  preview?(data: ExportData): JSX.Element;
  
  /** Optional: Configuration UI */
  ConfigComponent?: Component<ExportConfigProps>;
}

/** Data to export */
export interface ExportData {
  type: "report" | "artifacts" | "evidence" | "custom";
  title?: string;
  data: unknown;
  metadata?: Record<string, unknown>;
}

/** Export options */
export interface ExportOptions {
  filename?: string;
  template?: string;
  includeMetadata?: boolean;
  customOptions?: Record<string, unknown>;
}

/** Props for export config component */
export interface ExportConfigProps {
  options: ExportOptions;
  onChange: (options: ExportOptions) => void;
}

// =============================================================================
// ANALYSIS TOOL EXTENSION
// =============================================================================
// Tools for analysis, searching, correlation, etc.

/** Analysis tool extension interface */
export interface AnalysisToolExtension {
  manifest: ExtensionManifest;
  
  /** Tool identifier */
  toolId: string;
  
  /** Display name */
  toolName: string;
  
  /** Icon */
  icon?: string;
  
  /** Where to show this tool in the UI */
  placement: ToolPlacement;
  
  /** The tool component */
  Component: Component<AnalysisToolProps>;
  
  /** Optional: Run analysis programmatically */
  analyze?(input: AnalysisInput): Promise<AnalysisResult>;
}

/** Where to place the tool in the UI */
export type ToolPlacement =
  | "toolbar"           // Main toolbar
  | "sidebar"           // Left sidebar
  | "panel"             // New panel tab
  | "menu"              // Tools menu
  | "context-menu";     // Right-click context menu

/** Props for analysis tool component */
export interface AnalysisToolProps {
  databases: ProcessedDatabase[];
  files: DiscoveredFile[];
  selectedFile?: DiscoveredFile;
  selectedDatabase?: ProcessedDatabase;
  onResult?: (result: AnalysisResult) => void;
}

/** Input for programmatic analysis */
export interface AnalysisInput {
  type: "files" | "artifacts" | "timeline" | "custom";
  data: unknown;
  options?: Record<string, unknown>;
}

/** Analysis result */
export interface AnalysisResult {
  success: boolean;
  type: string;
  title: string;
  summary?: string;
  data: unknown;
  exportable?: boolean;
}

// =============================================================================
// INTEGRATION EXTENSION
// =============================================================================
// Third-party integrations (APIs, external tools, cloud services)

/** Integration extension interface */
export interface IntegrationExtension {
  manifest: ExtensionManifest;
  
  /** Integration identifier */
  integrationId: string;
  
  /** Service name */
  serviceName: string;
  
  /** Icon */
  icon?: string;
  
  /** Check if integration is configured/available */
  isAvailable(): Promise<boolean>;
  
  /** Configure the integration */
  configure?(config: Record<string, unknown>): Promise<void>;
  
  /** Get configuration UI */
  ConfigComponent?: Component<IntegrationConfigProps>;
  
  /** Actions provided by this integration */
  actions: IntegrationAction[];
}

/** Integration configuration props */
export interface IntegrationConfigProps {
  config: Record<string, unknown>;
  onChange: (config: Record<string, unknown>) => void;
  onTest: () => Promise<boolean>;
}

/** Action provided by an integration */
export interface IntegrationAction {
  id: string;
  name: string;
  description?: string;
  icon?: string;
  execute(input: unknown): Promise<unknown>;
}

// =============================================================================
// EXTENSION LIFECYCLE
// =============================================================================

/** Extension lifecycle hooks */
export interface ExtensionLifecycle {
  /** Called when extension is loaded */
  onLoad?(): Promise<void>;
  
  /** Called when extension is enabled */
  onEnable?(): Promise<void>;
  
  /** Called when extension is disabled */
  onDisable?(): Promise<void>;
  
  /** Called when extension is unloaded */
  onUnload?(): Promise<void>;
  
  /** Called when app settings change */
  onSettingsChange?(settings: Record<string, unknown>): void;
}

// =============================================================================
// UNION TYPE FOR ALL EXTENSIONS
// =============================================================================

/** Any extension type */
export type Extension =
  | (DatabaseProcessorExtension & ExtensionLifecycle)
  | (ContainerParserExtension & ExtensionLifecycle)
  | (ArtifactViewerExtension & ExtensionLifecycle)
  | (ExportFormatExtension & ExtensionLifecycle)
  | (AnalysisToolExtension & ExtensionLifecycle)
  | (IntegrationExtension & ExtensionLifecycle);

/** Extension state */
export interface ExtensionState {
  manifest: ExtensionManifest;
  enabled: boolean;
  loaded: boolean;
  error?: string;
}
