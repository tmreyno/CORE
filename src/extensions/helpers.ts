// =============================================================================
// CORE-FFX EXTENSION HELPERS
// =============================================================================
// Factory functions to simplify extension creation with proper defaults.
// =============================================================================

import type {
  ExtensionManifest,
  DatabaseProcessorExtension,
  ContainerParserExtension,
  ArtifactViewerExtension,
  ExportFormatExtension,
  AnalysisToolExtension,
  IntegrationExtension,
  ExtensionLifecycle,
} from "./types";

// =============================================================================
// MANIFEST HELPERS
// =============================================================================

/**
 * Create a base extension manifest with defaults
 */
export function createManifest(
  id: string,
  name: string,
  version: string,
  options: Partial<ExtensionManifest> = {}
): ExtensionManifest {
  return {
    id,
    name,
    version,
    description: options.description ?? "",
    author: options.author ?? "Unknown",
    category: options.category ?? "utility",
    ...options,
  };
}

// =============================================================================
// EXTENSION FACTORY FUNCTIONS
// =============================================================================

/**
 * Generic extension creator (use specialized functions below for type safety)
 */
export function createExtension<T extends { manifest: ExtensionManifest }>(
  config: T & Partial<ExtensionLifecycle>
): T & ExtensionLifecycle {
  return {
    ...config,
    onLoad: config.onLoad,
    onEnable: config.onEnable,
    onDisable: config.onDisable,
    onUnload: config.onUnload,
    onSettingsChange: config.onSettingsChange,
  };
}

/**
 * Create a database processor extension
 */
export function createDatabaseProcessor(
  config: Omit<DatabaseProcessorExtension, "manifest"> & {
    manifest: Partial<ExtensionManifest> & Pick<ExtensionManifest, "id" | "name" | "version">;
  } & Partial<ExtensionLifecycle>
): DatabaseProcessorExtension & ExtensionLifecycle {
  const manifest: ExtensionManifest = {
    ...config.manifest,
    description: config.manifest.description ?? `Database processor for ${config.dbType}`,
    author: config.manifest.author ?? "Unknown",
    category: "database-processor",
  };
  
  return createExtension({
    ...config,
    manifest,
  });
}

/**
 * Create a container parser extension
 */
export function createContainerParser(
  config: Omit<ContainerParserExtension, "manifest"> & {
    manifest: Partial<ExtensionManifest> & Pick<ExtensionManifest, "id" | "name" | "version">;
  } & Partial<ExtensionLifecycle>
): ContainerParserExtension & ExtensionLifecycle {
  const manifest: ExtensionManifest = {
    ...config.manifest,
    description: config.manifest.description ?? `Parser for ${config.containerType} containers`,
    author: config.manifest.author ?? "Unknown",
    category: "container-parser",
  };
  
  return createExtension({
    ...config,
    manifest,
  });
}

/**
 * Create an artifact viewer extension
 */
export function createArtifactViewer(
  config: Omit<ArtifactViewerExtension, "manifest"> & {
    manifest: Partial<ExtensionManifest> & Pick<ExtensionManifest, "id" | "name" | "version">;
  } & Partial<ExtensionLifecycle>
): ArtifactViewerExtension & ExtensionLifecycle {
  const manifest: ExtensionManifest = {
    ...config.manifest,
    description: config.manifest.description ?? `Viewer for ${config.artifactTypes.join(", ")} artifacts`,
    author: config.manifest.author ?? "Unknown",
    category: "artifact-viewer",
  };
  
  return createExtension({
    ...config,
    manifest,
  });
}

/**
 * Create an export format extension
 */
export function createExportFormat(
  config: Omit<ExportFormatExtension, "manifest"> & {
    manifest: Partial<ExtensionManifest> & Pick<ExtensionManifest, "id" | "name" | "version">;
  } & Partial<ExtensionLifecycle>
): ExportFormatExtension & ExtensionLifecycle {
  const manifest: ExtensionManifest = {
    ...config.manifest,
    description: config.manifest.description ?? `Export to ${config.formatName} format`,
    author: config.manifest.author ?? "Unknown",
    category: "export-format",
  };
  
  return createExtension({
    ...config,
    manifest,
  });
}

/**
 * Create an analysis tool extension
 */
export function createAnalysisTool(
  config: Omit<AnalysisToolExtension, "manifest"> & {
    manifest: Partial<ExtensionManifest> & Pick<ExtensionManifest, "id" | "name" | "version">;
  } & Partial<ExtensionLifecycle>
): AnalysisToolExtension & ExtensionLifecycle {
  const manifest: ExtensionManifest = {
    ...config.manifest,
    description: config.manifest.description ?? `Analysis tool: ${config.toolName}`,
    author: config.manifest.author ?? "Unknown",
    category: "analysis-tool",
  };
  
  return createExtension({
    ...config,
    manifest,
  });
}

/**
 * Create an integration extension
 */
export function createIntegration(
  config: Omit<IntegrationExtension, "manifest"> & {
    manifest: Partial<ExtensionManifest> & Pick<ExtensionManifest, "id" | "name" | "version">;
  } & Partial<ExtensionLifecycle>
): IntegrationExtension & ExtensionLifecycle {
  const manifest: ExtensionManifest = {
    ...config.manifest,
    description: config.manifest.description ?? `Integration with ${config.serviceName}`,
    author: config.manifest.author ?? "Unknown",
    category: "integration",
  };
  
  return createExtension({
    ...config,
    manifest,
  });
}
