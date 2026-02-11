// =============================================================================
// CORE-FFX EXTENSION REGISTRY
// =============================================================================
// Central registry for managing extensions. Handles registration, discovery,
// lifecycle management, and extension lookup.
// =============================================================================

import { createSignal, createRoot } from "solid-js";
import { getSetting, setSetting } from "../hooks/useDatabase";
import { logger } from "../utils/logger";
const log = logger.scope('ExtensionRegistry');
import type {
  Extension,
  ExtensionId,
  ExtensionManifest,
  ExtensionState,
  ExtensionCategory,
  DatabaseProcessorExtension,
  ContainerParserExtension,
  ArtifactViewerExtension,
  ExportFormatExtension,
  AnalysisToolExtension,
  IntegrationExtension,
  ArtifactResult,
} from "./types";

// =============================================================================
// REGISTRY STATE
// =============================================================================

/** Internal extension storage */
interface ExtensionEntry {
  extension: Extension;
  state: ExtensionState;
}

// Create reactive state in a root scope
const registryState = createRoot(() => {
  const [extensions, setExtensions] = createSignal<Map<ExtensionId, ExtensionEntry>>(new Map());
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  
  return { extensions, setExtensions, loading, setLoading, error, setError };
});

// =============================================================================
// REGISTRATION
// =============================================================================

/**
 * Register an extension with the registry
 */
export async function registerExtension(extension: Extension): Promise<void> {
  const { extensions, setExtensions } = registryState;
  const id = extension.manifest.id;
  
  // Check for duplicate
  if (extensions().has(id)) {
    throw new Error(`Extension already registered: ${id}`);
  }
  
  // Validate manifest
  validateManifest(extension.manifest);
  
  // Create entry
  const entry: ExtensionEntry = {
    extension,
    state: {
      manifest: extension.manifest,
      enabled: false,
      loaded: false,
    },
  };
  
  // Add to registry
  setExtensions((prev) => {
    const next = new Map(prev);
    next.set(id, entry);
    return next;
  });
  
  log.debug(`Registered: ${extension.manifest.name} (${id})`);
}

/**
 * Unregister an extension
 */
export async function unregisterExtension(id: ExtensionId): Promise<void> {
  const { extensions, setExtensions } = registryState;
  
  const entry = extensions().get(id);
  if (!entry) {
    throw new Error(`Extension not found: ${id}`);
  }
  
  // Disable and unload if needed
  if (entry.state.enabled) {
    await disableExtension(id);
  }
  
  // Remove from registry
  setExtensions((prev) => {
    const next = new Map(prev);
    next.delete(id);
    return next;
  });
  
  log.debug(`Unregistered: ${id}`);
}

// =============================================================================
// LIFECYCLE MANAGEMENT
// =============================================================================

/**
 * Enable an extension
 */
export async function enableExtension(id: ExtensionId): Promise<void> {
  const { extensions, setExtensions } = registryState;
  
  const entry = extensions().get(id);
  if (!entry) {
    throw new Error(`Extension not found: ${id}`);
  }
  
  if (entry.state.enabled) {
    return; // Already enabled
  }
  
  try {
    // Load if not loaded
    if (!entry.state.loaded && entry.extension.onLoad) {
      await entry.extension.onLoad();
    }
    
    // Enable
    if (entry.extension.onEnable) {
      await entry.extension.onEnable();
    }
    
    // Update state
    setExtensions((prev) => {
      const next = new Map(prev);
      const updated = next.get(id)!;
      updated.state = { ...updated.state, enabled: true, loaded: true, error: undefined };
      return next;
    });
    
    // Persist enabled state
    await saveEnabledExtensions();
    
    log.debug(`Enabled: ${id}`);
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    setExtensions((prev) => {
      const next = new Map(prev);
      const updated = next.get(id)!;
      updated.state = { ...updated.state, error: message };
      return next;
    });
    throw err;
  }
}

/**
 * Disable an extension
 */
export async function disableExtension(id: ExtensionId): Promise<void> {
  const { extensions, setExtensions } = registryState;
  
  const entry = extensions().get(id);
  if (!entry) {
    throw new Error(`Extension not found: ${id}`);
  }
  
  if (!entry.state.enabled) {
    return; // Already disabled
  }
  
  try {
    if (entry.extension.onDisable) {
      await entry.extension.onDisable();
    }
    
    setExtensions((prev) => {
      const next = new Map(prev);
      const updated = next.get(id)!;
      updated.state = { ...updated.state, enabled: false };
      return next;
    });
    
    // Persist enabled state
    await saveEnabledExtensions();
    
    log.debug(`Disabled: ${id}`);
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    setExtensions((prev) => {
      const next = new Map(prev);
      const updated = next.get(id)!;
      updated.state = { ...updated.state, error: message };
      return next;
    });
    throw err;
  }
}

// =============================================================================
// PERSISTENCE
// =============================================================================

const ENABLED_EXTENSIONS_KEY = "enabled_extensions";
const EXTENSION_SETTINGS_PREFIX = "ext_settings_";

/**
 * Save the list of enabled extension IDs to storage
 */
async function saveEnabledExtensions(): Promise<void> {
  const { extensions } = registryState;
  const enabledIds = Array.from(extensions().entries())
    .filter(([_, entry]) => entry.state.enabled)
    .map(([id]) => id);
  
  try {
    await setSetting(ENABLED_EXTENSIONS_KEY, JSON.stringify(enabledIds));
  } catch (err) {
    log.warn("Failed to save enabled extensions:", err);
  }
}

/**
 * Load the list of enabled extension IDs from storage
 */
async function loadEnabledExtensions(): Promise<ExtensionId[]> {
  try {
    const stored = await getSetting(ENABLED_EXTENSIONS_KEY);
    if (stored) {
      return JSON.parse(stored) as ExtensionId[];
    }
  } catch (err) {
    log.warn("Failed to load enabled extensions:", err);
  }
  return [];
}

/**
 * Save settings for a specific extension
 */
export async function saveExtensionSettings(id: ExtensionId, settings: Record<string, unknown>): Promise<void> {
  try {
    await setSetting(`${EXTENSION_SETTINGS_PREFIX}${id}`, JSON.stringify(settings));
  } catch (err) {
    log.warn(`Failed to save settings for ${id}:`, err);
    throw err;
  }
}

/**
 * Load settings for a specific extension
 */
export async function loadExtensionSettings(id: ExtensionId): Promise<Record<string, unknown> | null> {
  try {
    const stored = await getSetting(`${EXTENSION_SETTINGS_PREFIX}${id}`);
    if (stored) {
      return JSON.parse(stored) as Record<string, unknown>;
    }
  } catch (err) {
    log.warn(`Failed to load settings for ${id}:`, err);
  }
  return null;
}

// =============================================================================
// QUERIES
// =============================================================================

/**
 * Get all registered extensions
 */
export function getAllExtensions(): ExtensionState[] {
  return Array.from(registryState.extensions().values()).map((e) => e.state);
}

/**
 * Get enabled extensions
 */
export function getEnabledExtensions(): Extension[] {
  return Array.from(registryState.extensions().values())
    .filter((e) => e.state.enabled)
    .map((e) => e.extension);
}

/**
 * Get extensions by category
 */
export function getExtensionsByCategory(category: ExtensionCategory): Extension[] {
  return Array.from(registryState.extensions().values())
    .filter((e) => e.state.enabled && e.state.manifest.category === category)
    .map((e) => e.extension);
}

/**
 * Get extension by ID
 */
export function getExtension(id: ExtensionId): Extension | undefined {
  return registryState.extensions().get(id)?.extension;
}

/**
 * Get extension state by ID
 */
export function getExtensionState(id: ExtensionId): ExtensionState | undefined {
  return registryState.extensions().get(id)?.state;
}

// =============================================================================
// SPECIALIZED GETTERS
// =============================================================================

/**
 * Get all database processor extensions
 */
export function getDatabaseProcessors(): DatabaseProcessorExtension[] {
  return getExtensionsByCategory("database-processor") as DatabaseProcessorExtension[];
}

/**
 * Get all container parser extensions
 */
export function getContainerParsers(): ContainerParserExtension[] {
  return getExtensionsByCategory("container-parser") as ContainerParserExtension[];
}

/**
 * Get all artifact viewer extensions
 */
export function getArtifactViewers(): ArtifactViewerExtension[] {
  return getExtensionsByCategory("artifact-viewer") as ArtifactViewerExtension[];
}

/**
 * Find a viewer for an artifact
 */
export function findViewerForArtifact(artifact: ArtifactResult): ArtifactViewerExtension | undefined {
  const viewers = getArtifactViewers()
    .filter((v) => v.canHandle(artifact))
    .sort((a, b) => (b.priority ?? 0) - (a.priority ?? 0));
  
  return viewers[0];
}

/**
 * Get all export format extensions
 */
export function getExportFormats(): ExportFormatExtension[] {
  return getExtensionsByCategory("export-format") as ExportFormatExtension[];
}

/**
 * Get all analysis tool extensions
 */
export function getAnalysisTools(): AnalysisToolExtension[] {
  return getExtensionsByCategory("analysis-tool") as AnalysisToolExtension[];
}

/**
 * Get all integration extensions
 */
export function getIntegrations(): IntegrationExtension[] {
  return getExtensionsByCategory("integration") as IntegrationExtension[];
}

// =============================================================================
// VALIDATION
// =============================================================================

function validateManifest(manifest: ExtensionManifest): void {
  if (!manifest.id || typeof manifest.id !== "string") {
    throw new Error("Extension manifest must have a valid id");
  }
  
  if (!manifest.name || typeof manifest.name !== "string") {
    throw new Error("Extension manifest must have a valid name");
  }
  
  if (!manifest.version || typeof manifest.version !== "string") {
    throw new Error("Extension manifest must have a valid version");
  }
  
  if (!manifest.category) {
    throw new Error("Extension manifest must have a category");
  }
  
  // Validate ID format (reverse domain notation recommended)
  if (!/^[a-z0-9-]+(\.[a-z0-9-]+)*$/i.test(manifest.id)) {
    logger.warn(
      `[ExtensionRegistry] Extension ID "${manifest.id}" doesn't follow recommended format (e.g., "com.example.my-extension")`
    );
  }
}

// =============================================================================
// HOOKS FOR COMPONENTS
// =============================================================================

/**
 * Hook to get reactive extension list
 */
export function useExtensions() {
  return {
    extensions: registryState.extensions,
    loading: registryState.loading,
    error: registryState.error,
    
    // Actions
    register: registerExtension,
    unregister: unregisterExtension,
    enable: enableExtension,
    disable: disableExtension,
    
    // Queries
    getAll: getAllExtensions,
    getEnabled: getEnabledExtensions,
    getByCategory: getExtensionsByCategory,
    get: getExtension,
    getState: getExtensionState,
    
    // Specialized
    getDatabaseProcessors,
    getContainerParsers,
    getArtifactViewers,
    getExportFormats,
    getAnalysisTools,
    getIntegrations,
    findViewerForArtifact,
  };
}

// =============================================================================
// INITIALIZATION
// =============================================================================

/**
 * Initialize the extension registry (load built-in and saved extensions)
 */
export async function initializeRegistry(): Promise<void> {
  const { extensions, setLoading, setError } = registryState;
  
  setLoading(true);
  setError(null);
  
  try {
    // Load previously enabled extensions from storage
    const enabledIds = await loadEnabledExtensions();
    log.debug(`Found ${enabledIds.length} previously enabled extensions`);
    
    // Auto-enable previously enabled extensions that are currently registered
    const registered = extensions();
    for (const id of enabledIds) {
      if (registered.has(id)) {
        try {
          await enableExtension(id);
          log.debug(`Auto-enabled: ${id}`);
        } catch (err) {
          log.warn(`Failed to auto-enable ${id}:`, err);
        }
      } else {
        log.warn(`Previously enabled extension not found: ${id}`);
      }
    }
    
    log.debug("Initialized");
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    setError(message);
    log.error("Initialization error:", err);
  } finally {
    setLoading(false);
  }
}
