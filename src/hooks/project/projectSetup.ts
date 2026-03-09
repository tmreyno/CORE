// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Project setup handlers: directory picker, wizard completion, and
 * CaseDocument → SelectedEntry conversion.
 */

import type { Setter } from "solid-js";
import type { SelectedEntry } from "../../components";
import type { CaseDocument } from "../../types";
import type { useFileManager, useHashManager, useProject, useProcessedDatabases } from "../../hooks";
import type { LeftPanelTab } from "../../components";
import type { ProjectLocations } from "../../components";
import { logError, logInfo } from "../../utils/telemetry";
import { announce } from "../../utils/accessibility";
import { logger } from "../../utils/logger";
import { joinPath } from "../../utils/pathUtils";
import { invoke } from "@tauri-apps/api/core";
import caseFolderTemplate from "../../templates/project/case-folder-template.json";

// Create a scoped logger for project operations
const log = logger.scope("Project");

// ─── createDocumentEntry ─────────────────────────────────────────────────────

/**
 * Create a SelectedEntry from a CaseDocument for viewing.
 */
export function createDocumentEntry(doc: CaseDocument, isDiskFile = true): SelectedEntry {
  return {
    containerPath: doc.path,
    entryPath: doc.path,
    name: doc.filename,
    size: doc.size,
    isDir: false,
    isDiskFile,
    containerType: doc.format || "file",
    metadata: {
      document_type: doc.document_type,
      case_number: doc.case_number,
      evidence_id: doc.evidence_id,
      format: doc.format,
      modified: doc.modified,
    },
  };
}

// ─── handleOpenDirectory ─────────────────────────────────────────────────────

export interface HandleOpenDirectoryParams {
  setPendingProjectRoot: Setter<string | null>;
  setShowProjectWizard: Setter<boolean>;
  toast: {
    success: (title: string, message?: string) => void;
    error: (title: string, message?: string) => void;
    info: (title: string, message?: string) => void;
  };
}

/**
 * Handle opening a project directory - shows the setup wizard.
 */
export async function handleOpenDirectory(params: HandleOpenDirectoryParams) {
  const { setPendingProjectRoot, setShowProjectWizard, toast } = params;
  const { open } = await import("@tauri-apps/plugin-dialog");

  try {
    const selected = await open({
      title: "Select Project Directory",
      multiple: false,
      directory: true,
    });
    if (selected) {
      setPendingProjectRoot(selected);
      setShowProjectWizard(true);
    }
  } catch (err) {
    log.error("Failed to open directory:", err);
    logError(
      err instanceof Error ? err : new Error("Failed to open directory dialog"),
      { category: "ui", source: "handleOpenDirectory" },
    );
    toast.error("Failed to Open", "Could not open directory dialog");
  }
}

// ─── handleProjectSetupComplete ──────────────────────────────────────────────

export interface HandleProjectSetupCompleteParams {
  fileManager: ReturnType<typeof useFileManager>;
  hashManager: ReturnType<typeof useHashManager>;
  processedDbManager: ReturnType<typeof useProcessedDatabases>;
  projectManager: ReturnType<typeof useProject>;
  setShowProjectWizard: Setter<boolean>;
  setCaseDocumentsPath: Setter<string | null>;
  setLeftPanelTab: Setter<LeftPanelTab>;
  setPendingProjectRoot: Setter<string | null>;
  toast: {
    success: (title: string, message?: string) => void;
    error: (title: string, message?: string) => void;
    info: (title: string, message?: string) => void;
  };
  /** Optional: Function to get save options for initial save */
  getSaveOptions?: () => import("./types").BuildProjectOptions | null;
}

/**
 * Handle when project setup wizard completes.
 */
export async function handleProjectSetupComplete(
  params: HandleProjectSetupCompleteParams,
  locations: ProjectLocations,
) {
  const {
    fileManager,
    hashManager,
    processedDbManager,
    projectManager,
    setShowProjectWizard,
    setCaseDocumentsPath,
    setLeftPanelTab,
    setPendingProjectRoot,
    toast,
  } = params;

  setShowProjectWizard(false);

  // Set scanDir FIRST so that when createProject triggers reactivity,
  // the toolbar dropdown value already matches the new evidence path.
  // This prevents the brief flash of the old/stale path.
  fileManager.setScanDir(locations.evidencePath);

  // Create a new project with the provided name, owner, and case identification
  await projectManager.createProject(
    locations.projectRoot,
    locations.projectName,
    locations.ownerName,
    locations.caseNumber,
    locations.caseName,
  );

  // Ensure standard forensic folder structure exists on disk.
  // create_folders_from_template is idempotent — it creates only missing dirs.
  // If auto-discovery already found existing directories, those paths are preserved.
  try {
    const templateJson = JSON.stringify(caseFolderTemplate);
    const result = await invoke<{
      createdCount: number;
      existingCount: number;
      rolePaths: Record<string, string>;
      allPaths: string[];
    }>("create_folders_from_template", {
      templateJson: templateJson,
      rootPath: locations.projectRoot,
      caseName: null,
    });

    log.info(
      `Folder structure: ${result.createdCount} created, ${result.existingCount} existing`,
    );

    // If wizard paths defaulted to the project root (no specific subdirectories
    // found during auto-discovery), update them to the template's role paths.
    if (
      locations.evidencePath === locations.projectRoot &&
      result.rolePaths.evidence
    ) {
      locations.evidencePath = result.rolePaths.evidence;
      fileManager.setScanDir(locations.evidencePath);
    }
    if (
      locations.processedDbPath === locations.projectRoot &&
      result.rolePaths.processedDb
    ) {
      locations.processedDbPath = result.rolePaths.processedDb;
    }
    if (
      locations.caseDocumentsPath === locations.projectRoot &&
      result.rolePaths.caseDocuments
    ) {
      locations.caseDocumentsPath = result.rolePaths.caseDocuments;
    }
  } catch (err) {
    log.warn("Failed to create folder structure:", err);
    // Non-fatal — continue with project setup even if folder creation fails
  }

  // Update project locations so toolbar dropdown is populated
  projectManager.updateLocations({
    project_root: locations.projectRoot,
    evidence_path: locations.evidencePath,
    processed_db_path: locations.processedDbPath,
    case_documents_path: locations.caseDocumentsPath,
    auto_discovered: true,
    configured_at: new Date().toISOString(),
    evidence_file_count: 0, // Will be updated after scan
    processed_db_count: locations.discoveredDatabases.length,
    load_stored_hashes: locations.loadStoredHashes ?? true,
  });

  // If we have pre-loaded stored hashes from wizard step 2, import them to hash manager
  if (locations.loadedStoredHashes && locations.loadedStoredHashes.size > 0) {
    hashManager.importPreloadedStoredHashes(locations.loadedStoredHashes);
    // Scan without auto-loading hashes (we already have them)
    await fileManager.scanForFiles(locations.evidencePath, undefined, true);
  } else {
    // No pre-loaded hashes - let scanForFiles auto-load in background
    await fileManager.scanForFiles(locations.evidencePath);
  }

  // Store case documents path for CaseDocumentsPanel
  setCaseDocumentsPath(locations.caseDocumentsPath || locations.evidencePath);

  // If processed databases were discovered, add them
  if (locations.discoveredDatabases.length > 0) {
    // Add discovered processed databases to the manager
    processedDbManager.addDatabases(locations.discoveredDatabases);
    // Select the first discovered database to show details
    await processedDbManager.selectDatabase(locations.discoveredDatabases[0]);
    // Switch to processed tab
    setLeftPanelTab("processed");
    log.debug(
      `Found ${locations.discoveredDatabases.length} processed databases in: ${locations.processedDbPath}`,
    );
  }

  // =========================================================================
  // SAVE THE PROJECT FILE (.cffx)
  // =========================================================================
  // Build the default project file path
  const projectFileName = (locations.projectName || "project").replace(
    /[^a-zA-Z0-9_-]/g,
    "_",
  );
  const projectFilePath = joinPath(locations.projectRoot, `${projectFileName}.cffx`);

  // Build save options with the initial state
  const saveOptions: import("./types").BuildProjectOptions =
    params.getSaveOptions?.() || {
      rootPath: locations.projectRoot,
      projectName: locations.projectName,
      hashHistory: hashManager.hashHistory(),
      processedDatabases: processedDbManager.databases(),
      selectedProcessedDb: processedDbManager.selectedDatabase(),
      evidenceCache: {
        discoveredFiles: fileManager.discoveredFiles(),
        fileInfoMap: fileManager.fileInfoMap(),
        fileHashMap: new Map(), // Will be populated from hashHistory
      },
      caseDocumentsCache: {
        documents: [],
        searchPath: locations.caseDocumentsPath || locations.evidencePath,
      },
    };

  // Save the project to disk
  try {
    const saveResult = await projectManager.saveProject(
      saveOptions,
      projectFilePath,
    );
    if (saveResult.success) {
      log.info(`Project saved to: ${saveResult.path}`);

      // Open the per-window project database (.ffxdb) so dbSync operations
      // work during the first session. Without this, all dbSync calls silently
      // fail because project_db_open is only called in the project-load path.
      const savedPath = saveResult.path || projectFilePath;
      try {
        const dbMsg = await invoke<string>("project_db_open", {
          cffxPath: savedPath,
        });
        log.info(`Project DB opened for new project: ${dbMsg}`);
      } catch (dbErr) {
        log.warn("Could not open project database for new project:", dbErr);
        // Non-fatal: project still works without the DB for this session
      }
    } else {
      log.warn(`Failed to save project: ${saveResult.error}`);
      toast.error(
        "Project Save Failed",
        saveResult.error || "Unknown error",
      );
    }
  } catch (saveErr) {
    log.error("Error saving project:", saveErr);
    toast.error(
      "Project Save Error",
      saveErr instanceof Error
        ? saveErr.message
        : "Failed to save project",
    );
  }

  // Log the project setup and notify user
  projectManager.logActivity(
    "project",
    "setup",
    `Project setup complete: Evidence=${locations.evidencePath}, Processed=${locations.processedDbPath}, CaseDocs=${locations.caseDocumentsPath}`,
  );
  logInfo("Project setup complete", {
    source: "handleProjectSetupComplete",
    context: { locations },
  });
  toast.success(
    "Project Ready",
    `Found ${fileManager.discoveredFiles().length} files`,
  );
  announce(
    `Project setup complete. Found ${fileManager.discoveredFiles().length} evidence files.`,
  );

  setPendingProjectRoot(null);
}
