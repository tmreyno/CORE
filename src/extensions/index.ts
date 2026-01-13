// =============================================================================
// CORE-FFX EXTENSION SYSTEM
// =============================================================================
// Public API for the extension system. Import from here to build extensions.
// =============================================================================

// Type exports
export type {
  // Metadata
  ExtensionManifest,
  ExtensionId,
  ExtensionCategory,
  ExtensionState,
  SemVer,
  
  // Extension interfaces
  Extension,
  DatabaseProcessorExtension,
  ContainerParserExtension,
  ArtifactViewerExtension,
  ExportFormatExtension,
  AnalysisToolExtension,
  IntegrationExtension,
  ExtensionLifecycle,
  
  // Supporting types
  ArtifactCategorySummary,
  ArtifactResult,
  QueryOptions,
  ContainerEntry,
  VerificationResult,
  ArtifactViewerProps,
  ExportData,
  ExportOptions,
  ExportConfigProps,
  ToolPlacement,
  AnalysisToolProps,
  AnalysisInput,
  AnalysisResult,
  IntegrationAction,
  IntegrationConfigProps,
} from "./types";

// Registry exports
export {
  // Registration
  registerExtension,
  unregisterExtension,
  
  // Lifecycle
  enableExtension,
  disableExtension,
  
  // Queries
  getAllExtensions,
  getEnabledExtensions,
  getExtensionsByCategory,
  getExtension,
  getExtensionState,
  
  // Specialized getters
  getDatabaseProcessors,
  getContainerParsers,
  getArtifactViewers,
  getExportFormats,
  getAnalysisTools,
  getIntegrations,
  findViewerForArtifact,
  
  // Hook
  useExtensions,
  
  // Initialization
  initializeRegistry,
} from "./registry";

// Utility exports
export {
  createExtension,
  createDatabaseProcessor,
  createContainerParser,
  createArtifactViewer,
  createExportFormat,
  createAnalysisTool,
  createIntegration,
} from "./helpers";
