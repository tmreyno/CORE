// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useWizardState — All state and async logic for the Project Setup Wizard.
 *
 * Owns 30+ signals, auto-discovery, browse handlers, hash loading,
 * and finalization. The main component just composes JSX around this hook.
 */

import { createSignal, createEffect, createMemo, on, type Accessor, type Setter } from "solid-js";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { logger } from "../../utils/logger";
import { getBasename, joinPath } from "../../utils/pathUtils";
import { checkPathWritable } from "../../api/drives";
import type { ProcessedDatabase } from "../../types/processed";
import type { StoredHash } from "../../types";
import type { ProjectLocations, ProjectSetupWizardProps } from "./types";
import caseFolderTemplate from "../../templates/project/case-folder-template.json";

const log = logger.scope("Wizard");

export interface HashLoadingProgress {
  current: number;
  total: number;
  currentFile: string;
  hashCount: number;
}

export interface WizardState {
  // Step navigation
  step: Accessor<number>;
  setStep: Setter<number>;

  // Names
  projectName: Accessor<string>;
  setProjectName: Setter<string>;
  ownerName: Accessor<string>;
  setOwnerName: Setter<string>;
  caseNumber: Accessor<string>;
  setCaseNumber: Setter<string>;
  caseName: Accessor<string>;
  setCaseName: Setter<string>;

  // Paths
  effectiveProjectRoot: Accessor<string>;
  evidencePath: Accessor<string>;
  setEvidencePath: Setter<string>;
  processedDbPath: Accessor<string>;
  setProcessedDbPath: Setter<string>;
  caseDocumentsPath: Accessor<string>;
  setCaseDocumentsPath: Setter<string>;

  // Options
  loadStoredHashes: Accessor<boolean>;
  setLoadStoredHashes: Setter<boolean>;

  // Discovery results
  discoveredEvidence: Accessor<string[]>;
  discoveredDatabases: Accessor<ProcessedDatabase[]>;
  discoveredCaseDocCount: Accessor<number>;
  setDiscoveredCaseDocCount: Setter<number>;

  // Scanning
  scanning: Accessor<boolean>;
  scanMessage: Accessor<string>;
  error: Accessor<string | null>;
  setError: Setter<string | null>;

  // Suggested path chips
  suggestedEvidence: Accessor<string[]>;
  suggestedProcessed: Accessor<string[]>;
  suggestedCaseDocs: Accessor<string[]>;
  evidenceChips: Accessor<string[]>;
  processedChips: Accessor<string[]>;
  caseDocsChips: Accessor<string[]>;

  // Derived
  showHashLoadingStep: Accessor<boolean>;
  hashProgressPercent: Accessor<number>;
  evidenceCount: Accessor<number>;
  databaseCount: Accessor<number>;

  // Hash loading
  hashLoadingProgress: Accessor<HashLoadingProgress>;
  loadedStoredHashes: Accessor<Map<string, StoredHash[]>>;

  // Actions
  browseProjectRoot: () => Promise<void>;
  browseAndCreateTemplate: (name: string, examiner: string) => Promise<void>;
  browseEvidence: () => Promise<void>;
  browseProcessed: () => Promise<void>;
  browseCaseDocs: () => Promise<void>;
  discoverEvidence: (path: string) => Promise<string[]>;
  discoverDatabases: (path: string) => Promise<ProcessedDatabase[]>;
  handleContinue: () => void;
  handleSkip: () => void;
  cancelHashLoading: () => void;
}

export function useWizardState(props: ProjectSetupWizardProps): WizardState {
  // Local project root (allows selection if props.projectRoot is empty)
  const [localProjectRoot, setLocalProjectRoot] = createSignal("");

  const [projectName, setProjectName] = createSignal("");
  const [ownerName, setOwnerName] = createSignal("");
  const [caseNumber, setCaseNumber] = createSignal("");
  const [caseName, setCaseName] = createSignal("");

  const effectiveProjectRoot = createMemo(() => props.projectRoot || localProjectRoot());

  // Step: -1 = select folder, 0 = scanning, 1 = configure, 2 = load hashes
  const [step, setStep] = createSignal(0);

  const [evidencePath, setEvidencePath] = createSignal("");
  const [processedDbPath, setProcessedDbPath] = createSignal("");
  const [caseDocumentsPath, setCaseDocumentsPath] = createSignal("");

  const [loadStoredHashes, setLoadStoredHashes] = createSignal(true);

  const [discoveredEvidence, setDiscoveredEvidence] = createSignal<string[]>([]);
  const [discoveredDatabases, setDiscoveredDatabases] = createSignal<ProcessedDatabase[]>([]);

  const [scanning, setScanning] = createSignal(false);
  const [scanMessage, setScanMessage] = createSignal("Scanning project directory...");
  const [error, setError] = createSignal<string | null>(null);

  const [suggestedEvidence, setSuggestedEvidence] = createSignal<string[]>([]);
  const [suggestedProcessed, setSuggestedProcessed] = createSignal<string[]>([]);
  const [suggestedCaseDocs, setSuggestedCaseDocs] = createSignal<string[]>([]);

  const [discoveredCaseDocCount, setDiscoveredCaseDocCount] = createSignal(0);
  const [discoveryStarted, setDiscoveryStarted] = createSignal(false);

  const [hashLoadingProgress, setHashLoadingProgress] = createSignal<HashLoadingProgress>({
    current: 0,
    total: 0,
    currentFile: "",
    hashCount: 0,
  });
  const [loadedStoredHashes, setLoadedStoredHashes] = createSignal<Map<string, StoredHash[]>>(
    new Map(),
  );
  const [hashLoadingCancelled, setHashLoadingCancelled] = createSignal(false);

  // ── Derived memos ───────────────────────────────────────────────────────

  const showHashLoadingStep = createMemo(
    () => loadStoredHashes() && discoveredEvidence().length > 0,
  );
  const hashProgressPercent = createMemo(() => {
    const progress = hashLoadingProgress();
    return progress.total > 0 ? (progress.current / progress.total) * 100 : 0;
  });
  const evidenceCount = createMemo(() => discoveredEvidence().length);
  const databaseCount = createMemo(() => discoveredDatabases().length);
  const evidenceChips = createMemo(() => suggestedEvidence().slice(0, 3));
  const processedChips = createMemo(() => suggestedProcessed().slice(0, 3));
  const caseDocsChips = createMemo(() => suggestedCaseDocs().slice(0, 3));

  // ── Discovery ───────────────────────────────────────────────────────────

  const discoverEvidence = async (path: string): Promise<string[]> => {
    try {
      log.debug(" Discovering evidence files in:", path);
      const files = await invoke<string[]>("discover_evidence_files", {
        dirPath: path,
        recursive: true,
      });
      log.debug(" Found evidence files:", files.length);
      setDiscoveredEvidence(files);
      return files;
    } catch (err) {
      log.warn("Failed to discover evidence:", err);
      setDiscoveredEvidence([]);
      return [];
    }
  };

  const discoverDatabases = async (path: string): Promise<ProcessedDatabase[]> => {
    try {
      log.debug(" Discovering processed databases in:", path);
      const dbs = await invoke<ProcessedDatabase[]>("scan_for_processed_databases", {
        dirPath: path,
      });
      log.debug(" Found processed databases:", dbs.length);
      setDiscoveredDatabases(dbs);
      return dbs;
    } catch (err) {
      log.warn("Failed to discover processed databases:", err);
      setDiscoveredDatabases([]);
      return [];
    }
  };

  const startAutoDiscovery = async (projectRoot: string) => {
    log.debug(" Starting auto-discovery for:", projectRoot);
    setStep(0);
    setScanning(true);
    setError(null);
    setScanMessage("Looking for common directory structures...");

    try {
      const commonEvidencePaths = [
        "1.Evidence",
        "Evidence",
        "evidence",
        "Images",
        "Forensic Images",
        "Source",
      ];
      const commonProcessedPaths = [
        "2.Processed",
        "2.Processed.Database",
        "Processed",
        "processed",
        "AXIOM",
        "Cellebrite",
        "Exports",
        "Analysis",
      ];
      const commonCaseDocPaths = [
        "4.Case.Documents",
        "Case.Documents",
        "Case Documents",
        "CaseDocuments",
        "Documents",
        "Paperwork",
        "Forms",
        "COC",
        "Chain of Custody",
      ];

      const evidenceMatches: string[] = [];
      const processedMatches: string[] = [];
      const caseDocMatches: string[] = [];

      setScanMessage("Checking for evidence directories...");
      for (const subdir of commonEvidencePaths) {
        const testPath = joinPath(projectRoot, subdir);
        try {
          const exists = await invoke<boolean>("path_exists", { path: testPath });
          if (exists) {
            const isDir = await invoke<boolean>("path_is_directory", { path: testPath });
            if (isDir) evidenceMatches.push(testPath);
          }
        } catch (e) {
          log.warn("Error checking path:", testPath, e);
        }
      }
      evidenceMatches.push(projectRoot);

      setScanMessage("Checking for processed database directories...");
      for (const subdir of commonProcessedPaths) {
        const testPath = joinPath(projectRoot, subdir);
        try {
          const exists = await invoke<boolean>("path_exists", { path: testPath });
          if (exists) {
            const isDir = await invoke<boolean>("path_is_directory", { path: testPath });
            if (isDir) processedMatches.push(testPath);
          }
        } catch (e) {
          log.warn("Error checking processed path:", testPath, e);
        }
      }
      processedMatches.push(projectRoot);

      setScanMessage("Checking for case document directories...");
      for (const subdir of commonCaseDocPaths) {
        const testPath = joinPath(projectRoot, subdir);
        try {
          const exists = await invoke<boolean>("path_exists", { path: testPath });
          if (exists) {
            const isDir = await invoke<boolean>("path_is_directory", { path: testPath });
            if (isDir) caseDocMatches.push(testPath);
          }
        } catch (e) {
          log.warn("Error checking case doc path:", testPath, e);
        }
      }
      caseDocMatches.push(projectRoot);

      setSuggestedEvidence(evidenceMatches);
      setSuggestedProcessed(processedMatches);
      setSuggestedCaseDocs(caseDocMatches);

      const defaultEvidence = evidenceMatches[0] || projectRoot;
      const defaultProcessed = processedMatches[0] || projectRoot;
      const defaultCaseDocs = caseDocMatches[0] || projectRoot;

      setEvidencePath(defaultEvidence);
      setProcessedDbPath(defaultProcessed);
      setCaseDocumentsPath(defaultCaseDocs);

      setScanMessage("Discovering evidence files...");
      await discoverEvidence(defaultEvidence);

      setScanMessage("Discovering processed databases...");
      await discoverDatabases(defaultProcessed);

      setScanMessage("Looking for case documents...");
      try {
        const docs = await invoke<{ length: number }[]>("discover_case_documents", {
          evidencePath: defaultCaseDocs,
        });
        setDiscoveredCaseDocCount(Array.isArray(docs) ? docs.length : 0);
      } catch {
        setDiscoveredCaseDocCount(0);
      }

      log.debug(" Auto-discovery complete, moving to step 1");
      setStep(1);
    } catch (err) {
      log.error("Auto-discovery error:", err);
      setError(String(err));
      setEvidencePath(projectRoot);
      setProcessedDbPath(projectRoot);
      setCaseDocumentsPath(projectRoot);
      setStep(1);
    } finally {
      setScanning(false);
    }
  };

  // ── Folder template ─────────────────────────────────────────────────────

  const applyFolderTemplate = async (
    rootPath: string,
  ): Promise<Record<string, string> | null> => {
    try {
      const templateJson = JSON.stringify(caseFolderTemplate);
      const result = await invoke<{
        createdCount: number;
        existingCount: number;
        rolePaths: Record<string, string>;
        allPaths: string[];
      }>("create_folders_from_template", {
        templateJson: templateJson,
        rootPath: rootPath,
        caseName: null,
      });
      log.info(`Template applied: ${result.createdCount} created, ${result.existingCount} existing`);
      return result.rolePaths;
    } catch (err) {
      log.warn("Failed to apply folder template:", err);
      setError(`Failed to create folder structure: ${String(err)}`);
      return null;
    }
  };

  // ── Browse handlers ─────────────────────────────────────────────────────

  const browseProjectRoot = async () => {
    try {
      const selected = await open({
        title: "Select Project Folder",
        directory: true,
        multiple: false,
      });
      if (selected) {
        setLocalProjectRoot(selected);
        if (!projectName()) {
          const folderName = getBasename(selected);
          setProjectName(folderName);
        }
        setStep(0);
        setDiscoveryStarted(true);
        startAutoDiscovery(selected);
      }
    } catch (err) {
      setError(String(err));
    }
  };

  const browseAndCreateTemplate = async (name: string, examiner: string) => {
    try {
      const selected = await open({
        title: "Select Location for Case Folders",
        directory: true,
        multiple: false,
      });
      if (!selected) return;

      try {
        const check = await checkPathWritable(selected);
        if (!check.writable) {
          setError(check.reason);
          return;
        }
      } catch (err) {
        log.warn("Writability check failed, attempting anyway:", err);
      }

      setProjectName(name);
      if (examiner) setOwnerName(examiner);

      const projectRoot = joinPath(selected, name);
      log.info(`Creating case folder structure at: ${projectRoot}`);
      setLocalProjectRoot(projectRoot);

      setScanMessage("Creating case folder structure...");
      setStep(0);
      setScanning(true);
      const rolePaths = await applyFolderTemplate(projectRoot);
      if (!rolePaths) {
        log.warn("Template folder creation failed, aborting wizard flow");
        setScanning(false);
        setStep(-1);
        return;
      }

      if (rolePaths.evidence) setEvidencePath(rolePaths.evidence);
      if (rolePaths.processedDb) setProcessedDbPath(rolePaths.processedDb);
      if (rolePaths.caseDocuments) setCaseDocumentsPath(rolePaths.caseDocuments);

      setDiscoveryStarted(true);
      startAutoDiscovery(projectRoot);
    } catch (err) {
      log.error("browseAndCreateTemplate error:", err);
      setError(String(err));
      setScanning(false);
      setStep(-1);
    }
  };

  const browseEvidence = async () => {
    try {
      const selected = await open({
        title: "Select Evidence Directory",
        directory: true,
        multiple: false,
        defaultPath: effectiveProjectRoot(),
      });
      if (selected) {
        setEvidencePath(selected);
        setScanning(true);
        setScanMessage("Scanning for evidence files...");
        await discoverEvidence(selected);
        setScanning(false);
      }
    } catch (err) {
      setError(String(err));
    }
  };

  const browseProcessed = async () => {
    try {
      const selected = await open({
        title: "Select Processed Database Directory",
        directory: true,
        multiple: false,
        defaultPath: effectiveProjectRoot(),
      });
      if (selected) {
        setProcessedDbPath(selected);
        setScanning(true);
        setScanMessage("Scanning for processed databases...");
        await discoverDatabases(selected);
        setScanning(false);
      }
    } catch (err) {
      setError(String(err));
    }
  };

  const browseCaseDocs = async () => {
    try {
      const selected = await open({
        title: "Select Case Documents Directory",
        directory: true,
        multiple: false,
        defaultPath: effectiveProjectRoot(),
      });
      if (selected) {
        setCaseDocumentsPath(selected);
        setScanning(true);
        setScanMessage("Looking for case documents...");
        try {
          const docs = await invoke<{ length: number }[]>("discover_case_documents", {
            evidencePath: selected,
          });
          setDiscoveredCaseDocCount(Array.isArray(docs) ? docs.length : 0);
        } catch {
          setDiscoveredCaseDocCount(0);
        }
        setScanning(false);
      }
    } catch (err) {
      setError(String(err));
    }
  };

  // ── Hash loading ────────────────────────────────────────────────────────

  const loadHashesForEvidence = async () => {
    const files = discoveredEvidence();
    if (files.length === 0) {
      finalizeSetup();
      return;
    }

    setHashLoadingCancelled(false);
    setHashLoadingProgress({ current: 0, total: files.length, currentFile: "", hashCount: 0 });
    const hashMap = new Map<string, StoredHash[]>();
    let totalHashCount = 0;

    for (let i = 0; i < files.length; i++) {
      if (hashLoadingCancelled()) {
        setLoadedStoredHashes(hashMap);
        finalizeSetup();
        return;
      }

      const filePath = files[i];
      const filename = getBasename(filePath) || filePath;
      setHashLoadingProgress({
        current: i + 1,
        total: files.length,
        currentFile: filename,
        hashCount: totalHashCount,
      });

      try {
        const hashes = await invoke<StoredHash[]>("get_stored_hashes_only", {
          inputPath: filePath,
        });
        if (hashes && hashes.length > 0) {
          hashMap.set(filePath, hashes);
          totalHashCount += hashes.length;
          setHashLoadingProgress((prev) => ({ ...prev, hashCount: totalHashCount }));
        }
      } catch (err) {
        log.warn(`Failed to load hashes for ${filename}:`, err);
      }
    }

    setLoadedStoredHashes(hashMap);
    finalizeSetup();
  };

  // ── Finalize ────────────────────────────────────────────────────────────

  const finalizeSetup = () => {
    const root = effectiveProjectRoot();
    const locations: ProjectLocations = {
      projectName: projectName() || getBasename(root),
      ownerName: ownerName() || undefined,
      caseNumber: caseNumber() || undefined,
      caseName: caseName() || undefined,
      projectRoot: root,
      evidencePath: evidencePath(),
      processedDbPath: processedDbPath(),
      caseDocumentsPath: caseDocumentsPath(),
      discoveredEvidence: discoveredEvidence(),
      discoveredDatabases: discoveredDatabases(),
      loadStoredHashes: loadStoredHashes(),
      loadedStoredHashes: loadStoredHashes() ? loadedStoredHashes() : undefined,
    };
    props.onComplete(locations);
  };

  const handleContinue = () => {
    if (showHashLoadingStep()) {
      setStep(2);
      loadHashesForEvidence();
    } else {
      finalizeSetup();
    }
  };

  const cancelHashLoading = () => {
    setHashLoadingCancelled(true);
  };

  const handleSkip = () => {
    const root = effectiveProjectRoot();
    const locations: ProjectLocations = {
      projectName: projectName() || getBasename(root),
      ownerName: ownerName() || undefined,
      caseNumber: caseNumber() || undefined,
      caseName: caseName() || undefined,
      projectRoot: root,
      evidencePath: root,
      processedDbPath: root,
      caseDocumentsPath: root,
      discoveredEvidence: [],
      discoveredDatabases: [],
      loadStoredHashes: true,
    };
    props.onComplete(locations);
  };

  // ── Open/close effect ───────────────────────────────────────────────────

  createEffect(
    on(
      () => [props.isOpen, props.projectRoot] as const,
      ([isOpen, projectRoot]) => {
        if (isOpen && projectRoot && !discoveryStarted()) {
          log.debug(" Effect triggered - starting discovery with provided root");
          setDiscoveryStarted(true);
          setStep(0);
          if (!projectName()) {
            const folderName = getBasename(projectRoot);
            setProjectName(folderName);
          }
          startAutoDiscovery(projectRoot);
        } else if (isOpen && !projectRoot && !discoveryStarted()) {
          log.debug(" Effect triggered - no project root, showing folder selection");
          setStep(-1);
        } else if (!isOpen) {
          setDiscoveryStarted(false);
          setLocalProjectRoot("");
          setProjectName("");
          setOwnerName("");
          setCaseNumber("");
          setCaseName("");
          setStep(0);
          setDiscoveredEvidence([]);
          setDiscoveredDatabases([]);
          setSuggestedEvidence([]);
          setSuggestedProcessed([]);
          setSuggestedCaseDocs([]);
          setDiscoveredCaseDocCount(0);
          setLoadStoredHashes(true);
          setError(null);
        }
      },
    ),
  );

  return {
    step,
    setStep,
    projectName,
    setProjectName,
    ownerName,
    setOwnerName,
    caseNumber,
    setCaseNumber,
    caseName,
    setCaseName,
    effectiveProjectRoot,
    evidencePath,
    setEvidencePath,
    processedDbPath,
    setProcessedDbPath,
    caseDocumentsPath,
    setCaseDocumentsPath,
    loadStoredHashes,
    setLoadStoredHashes,
    discoveredEvidence,
    discoveredDatabases,
    discoveredCaseDocCount,
    setDiscoveredCaseDocCount,
    scanning,
    scanMessage,
    error,
    setError,
    suggestedEvidence,
    suggestedProcessed,
    suggestedCaseDocs,
    evidenceChips,
    processedChips,
    caseDocsChips,
    showHashLoadingStep,
    hashProgressPercent,
    evidenceCount,
    databaseCount,
    hashLoadingProgress,
    loadedStoredHashes,
    browseProjectRoot,
    browseAndCreateTemplate,
    browseEvidence,
    browseProcessed,
    browseCaseDocs,
    discoverEvidence,
    discoverDatabases,
    handleContinue,
    handleSkip,
    cancelHashLoading,
  };
}
