// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

// =============================================================================
// EXAMPLE EXTENSION: Custom Timeline Viewer
// =============================================================================
// This is a complete example of building an artifact viewer extension.
// Copy this file and modify it to create your own extensions.
// =============================================================================

/* @jsxImportSource solid-js */
import { Component } from "solid-js";
import { logger } from "../../utils/logger";
import {
  createArtifactViewer,
  ArtifactViewerExtension,
  ArtifactViewerProps,
  ArtifactResult,
  ExtensionLifecycle,
} from "../index";

// =============================================================================
// 1. CREATE THE VIEWER COMPONENT
// =============================================================================

/**
 * Timeline viewer component that displays timeline artifacts
 */
const TimelineViewer: Component<ArtifactViewerProps> = (props) => {
  const artifact = () => props.artifact;
  
  // Extract timeline-specific data
  const timestamp = () => artifact().timestamp || "Unknown time";
  const source = () => artifact().source || "Unknown source";
  const data = () => artifact().data as Record<string, unknown>;
  
  return (
    <div class="timeline-viewer">
      <div class="timeline-header">
        <h3>📅 Timeline Event</h3>
        <span class="timestamp">{timestamp()}</span>
      </div>
      
      <div class="timeline-content">
        <div class="field">
          <label>Source:</label>
          <span>{source()}</span>
        </div>
        
        <div class="field">
          <label>Type:</label>
          <span>{artifact().type}</span>
        </div>
        
        {/* Render dynamic data fields */}
        <div class="data-fields">
          {Object.entries(data()).map(([key, value]) => (
            <div class="field">
              <label>{key}:</label>
              <span>{String(value)}</span>
            </div>
          ))}
        </div>
      </div>
      
      {/* Actions */}
      <div class="timeline-actions">
        {props.onExport && (
          <button onClick={() => props.onExport?.(artifact(), "json")}>
            Export JSON
          </button>
        )}
      </div>
    </div>
  );
};

// =============================================================================
// 2. CREATE THE EXTENSION
// =============================================================================

/**
 * Timeline viewer extension definition
 */
export const timelineViewerExtension: ArtifactViewerExtension & ExtensionLifecycle = createArtifactViewer({
  // Manifest - describes the extension
  manifest: {
    id: "com.core-ffx.example.timeline-viewer",
    name: "Timeline Viewer",
    version: "1.0.0",
    description: "Custom viewer for timeline artifacts with enhanced display",
    author: "CORE-FFX Team",
    license: "MIT",
    keywords: ["timeline", "viewer", "artifacts"],
    icon: "📅",
  },
  
  // What artifact types this viewer handles
  artifactTypes: ["Timeline", "Event", "Activity", "WebHistory"],
  
  // What categories it supports
  categories: ["Timeline", "WebHistory"],
  
  // Priority (higher = preferred)
  priority: 100,
  
  // Check if this viewer can handle a specific artifact
  canHandle(artifact: ArtifactResult): boolean {
    // Handle any artifact with a timestamp or in our supported types
    return (
      this.artifactTypes.includes(artifact.type) ||
      artifact.timestamp !== undefined ||
      artifact.category === "Timeline"
    );
  },
  
  // The viewer component
  Component: TimelineViewer,
  
  // Lifecycle hooks (optional)
  async onLoad() {
    logger.debug("[TimelineViewer] Extension loaded");
    // Initialize resources, load settings, etc.
  },
  
  async onEnable() {
    logger.debug("[TimelineViewer] Extension enabled");
    // Register event listeners, start services, etc.
  },
  
  async onDisable() {
    logger.debug("[TimelineViewer] Extension disabled");
    // Clean up event listeners, stop services, etc.
  },
  
  onSettingsChange(settings: Record<string, unknown>) {
    logger.debug("[TimelineViewer] Settings changed:", settings);
    // React to settings changes
  },
});

// =============================================================================
// 3. REGISTER THE EXTENSION (in your app initialization)
// =============================================================================
// 
// import { registerExtension, enableExtension } from "@core-ffx/extensions";
// import { timelineViewerExtension } from "./examples/timeline-viewer";
// 
// // Register and enable
// await registerExtension(timelineViewerExtension);
// await enableExtension(timelineViewerExtension.manifest.id);
// 
// =============================================================================

export default timelineViewerExtension;
