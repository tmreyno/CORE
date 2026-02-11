// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ExportPanel - Modern export/copy/archive panel
 * 
 * Provides three export modes:
 * 1. Copy - Direct file/folder copy with verification
 * 2. Export - Copy with forensic metadata (hashes, timestamps)
 * 3. Archive - Create 7z archives with encryption and compression
 */

import { createSignal, createEffect, Show, For } from "solid-js";
import { open } from "@tauri-apps/plugin-dialog";
import {
  HiOutlineFolderOpen,
  HiOutlineArchiveBox,
  HiOutlineArrowUpTray,
  HiOutlinePlay,
  HiOutlineXMark,
  HiOutlineWrench,
} from "./icons";
import { createArchive, listenToProgress, estimateSize, formatBytes, CompressionLevel, testArchive, repairArchive, validateArchive, extractSplitArchive, listenToRepairProgress, listenToSplitExtractProgress } from "../api/archiveCreate";
import { exportFiles, type CopyProgress, type ExportOptions } from "../api/fileExport";
import { useToast } from "./Toast";
import { createActivity, updateProgress, completeActivity, failActivity, type Activity } from "../types/activity";
import { ExportMode } from "./export/ExportMode";
import { ArchiveMode } from "./export/ArchiveMode";
import { ToolsMode } from "./export/ToolsMode";

/** Export operation mode */
export type ExportMode = "export" | "archive" | "tools";

/** Export panel props */
export interface ExportPanelProps {
  /** Pre-selected source files (optional) */
  initialSources?: string[];
  /** Callback when export completes */
  onComplete?: (destination: string) => void;
  /** Callback when panel is closed */
  onClose?: () => void;
  /** Callback when an activity is created */
  onActivityCreate?: (activity: Activity) => void;
  /** Callback when an activity is updated */
  onActivityUpdate?: (id: string, updates: Partial<Activity>) => void;
}

export function ExportPanel(props: ExportPanelProps) {
  const toast = useToast();
  
  // === State ===
  const [mode, setMode] = createSignal<ExportMode>("export");
  const [sources, setSources] = createSignal<string[]>(props.initialSources || []);
  const [destination, setDestination] = createSignal("");
  const [archiveName, setArchiveName] = createSignal("evidence.7z");
  const [exportName, setExportName] = createSignal("forensic_export");
  const [isProcessing, setIsProcessing] = createSignal(false);
  const [showAdvanced, setShowAdvanced] = createSignal(false);
  
  // === Export Options ===
  const [computeHashes, setComputeHashes] = createSignal(true);
  const [verifyAfterCopy, setVerifyAfterCopy] = createSignal(true);
  const [generateJsonManifest, setGenerateJsonManifest] = createSignal(true);
  const [generateTxtReport, setGenerateTxtReport] = createSignal(true);
  
  // === Archive Options ===
  const [compressionLevel, setCompressionLevel] = createSignal<number>(CompressionLevel.Store);
  const [password, setPassword] = createSignal("");
  const [showPassword, setShowPassword] = createSignal(false);
  const [numThreads, setNumThreads] = createSignal(0); // 0 = auto
  const [solid, setSolid] = createSignal(false);
  const [splitSizeMb, setSplitSizeMb] = createSignal(2048); // 2GB default - good for cloud/USB
  
  // === Size Estimation ===
  const [estimatedUncompressed, setEstimatedUncompressed] = createSignal(0);
  const [estimatedCompressed, setEstimatedCompressed] = createSignal(0);
  
  // === Archive Tools State ===
  const [toolsTab, setToolsTab] = createSignal<"test" | "repair" | "validate" | "extract">("test");
  const [testArchivePath, setTestArchivePath] = createSignal("");
  const [repairCorruptedPath, setRepairCorruptedPath] = createSignal("");
  const [repairOutputPath, setRepairOutputPath] = createSignal("");
  const [validateArchivePath, setValidateArchivePath] = createSignal("");
  const [extractFirstVolume, setExtractFirstVolume] = createSignal("");
  const [extractOutputDir, setExtractOutputDir] = createSignal("");
  
  // Update size estimate when sources or compression level changes
  createEffect(() => {
    const sourceList = sources();
    const level = compressionLevel();
    if (sourceList.length > 0 && mode() === "archive") {
      estimateSize(sourceList, level)
        .then(([uncompressed, compressed]) => {
          setEstimatedUncompressed(uncompressed);
          setEstimatedCompressed(compressed);
        })
        .catch(() => {
          setEstimatedUncompressed(0);
          setEstimatedCompressed(0);
        });
    }
  });
  
  // === Handlers ===
  
  const handleAddSources = async () => {
    const selected = await open({
      multiple: true,
      directory: false,
      title: "Select Files to Export",
    });
    
    if (selected) {
      const newSources = Array.isArray(selected) ? selected : [selected];
      setSources([...sources(), ...newSources]);
    }
  };
  
  const handleAddFolder = async () => {
    const selected = await open({
      multiple: false,
      directory: true,
      title: "Select Folder to Export",
    });
    
    if (selected) {
      setSources([...sources(), selected as string]);
    }
  };
  
  const handleSelectDestination = async () => {
    const selected = await open({
      multiple: false,
      directory: true,
      title: "Select Destination Folder",
    });
    
    if (selected) {
      setDestination(selected as string);
    }
  };
  
  const handleRemoveSource = (index: number) => {
    setSources(sources().filter((_, i) => i !== index));
  };
  
  const handleStart = async () => {
    if (sources().length === 0) {
      toast.error("No Sources", "Please select files or folders to export");
      return;
    }
    
    if (!destination()) {
      toast.error("No Destination", "Please select a destination folder");
      return;
    }
    
    const currentMode = mode();
    
    if (currentMode === "archive") {
      await handleCreateArchive();
    } else {
      // Both copy and export use the unified handler
      await handleCopyOrExport();
    }
  };
  
  const handleCreateArchive = async () => {
    setIsProcessing(true);
    
    // Create activity record - simplified to match library output
    const activity = createActivity(
      "archive",
      `${destination()}/${archiveName()}`,
      sources().length,
      {
        compressionLevel: compressionLevel(),
        encrypted: !!password(),
      }
    );
    
    // Notify parent to track this activity
    props.onActivityCreate?.(activity);
    
    // Set up progress listener with throttling to prevent UI blocking
    let lastUpdate = 0;
    const UPDATE_INTERVAL = 250; // Update UI every 250ms max
    
    const unlisten = await listenToProgress((prog) => {
      const now = Date.now();
      // Throttle ALL progress updates to prevent overwhelming the UI
      if (now - lastUpdate < UPDATE_INTERVAL && prog.percent < 100) {
        return;
      }
      lastUpdate = now;
      
      // Update activity with progress - matches library output directly
      props.onActivityUpdate?.(activity.id, updateProgress(activity, {
        bytesProcessed: prog.bytesProcessed,
        bytesTotal: prog.bytesTotal,
        percent: prog.percent,
        currentFile: prog.currentFile || undefined,
        currentFileBytes: prog.currentFileBytes,
        currentFileTotal: prog.currentFileTotal,
      }));
    });
    
    try {
      const archivePath = `${destination()}/${archiveName()}`;
      
      const options = {
        compressionLevel: compressionLevel(),
        password: password() || undefined,
        numThreads: numThreads() || undefined,
        solid: solid(),
        splitSizeMb: splitSizeMb() || undefined,
      };
      
      // Start the archive creation (async, don't wait)
      createArchive(
        archivePath,
        sources(),
        options
      ).then((result) => {
        // Mark activity as completed
        props.onActivityUpdate?.(activity.id, completeActivity(activity));
        toast.success("Archive Created", `Successfully created: ${result}`);
        props.onComplete?.(result);
      }).catch((error: any) => {
        // Mark activity as failed
        props.onActivityUpdate?.(activity.id, failActivity(activity, error.message || String(error)));
        toast.error("Archive Creation Failed", error.message || String(error));
      }).finally(() => {
        // Cleanup progress listener when operation completes
        unlisten();
      });
      
      // Release the button immediately after starting the operation
      // Reset form to allow starting another archive
      setSources([]);
      setArchiveName("evidence.7z");
      setPassword("");
      setIsProcessing(false);
      
      // Show success toast immediately
      toast.success("Archive Started", `Creating ${archiveName()} - check Activity panel for progress`);
      
    } catch (error: any) {
      // Handle immediate errors (before operation starts)
      unlisten();
      props.onActivityUpdate?.(activity.id, failActivity(activity, error.message || String(error)));
      toast.error("Archive Creation Failed", error.message || String(error));
      setIsProcessing(false);
    }
  };
  
  const handleCopyOrExport = async () => {
    setIsProcessing(true);
    
    // Build export options
    const options: ExportOptions = {
      computeHashes: computeHashes(),
      verifyAfterCopy: verifyAfterCopy(),
      generateJsonManifest: generateJsonManifest(),
      generateTxtReport: generateTxtReport(),
      preserveTimestamps: true,
      overwrite: false,
      createDirs: true,
      exportName: exportName(),
    };
    
    // Create activity record
    const activity = createActivity(
      "export",
      destination(),
      sources().length,
      { includeHashes: options.computeHashes }
    );
    
    props.onActivityCreate?.(activity);
    
    // Start the export (async, don't wait) - same pattern as archive
    exportFiles(
      sources(),
      destination(),
      options,
      (prog: CopyProgress) => {
        // Update activity with progress
        props.onActivityUpdate?.(activity.id, updateProgress(activity, {
          bytesProcessed: prog.totalBytesCopied,
          bytesTotal: prog.totalBytes,
          percent: prog.percent,
          currentFile: prog.currentFile,
          filesProcessed: prog.currentIndex,
          totalFiles: prog.totalFiles,
        }));
      }
    ).then((result) => {
      props.onActivityUpdate?.(activity.id, completeActivity(activity));
      
      // Build success message
      let message = `Exported ${result.filesCopied} files (${formatBytes(result.bytesCopied)}) in ${(result.durationMs / 1000).toFixed(1)}s`;
      
      if (result.jsonManifestPath) {
        message += `\nManifest: ${result.jsonManifestPath.split('/').pop()}`;
      }
      if (result.txtReportPath) {
        message += `\nReport: ${result.txtReportPath.split('/').pop()}`;
      }
      
      toast.success("Export Complete", message);
      props.onComplete?.(destination());
    }).catch((error: any) => {
      props.onActivityUpdate?.(activity.id, failActivity(activity, error.message || String(error)));
      toast.error("Export Failed", error.message || String(error));
    });
    
    // Release the button immediately after starting the operation
    // Reset form to allow starting another export
    setSources([]);
    setExportName("forensic_export");
    setIsProcessing(false);
    
    // Show success toast immediately
    toast.success("Export Started", `Exporting to ${destination()} - check Activity panel for progress`);
  };
  
  // Reset handler to allow starting new operations
  const handleReset = () => {
    setSources([]);
    setDestination("");
    setArchiveName("evidence.7z");
    setExportName("forensic_export");
    setPassword("");
    setIsProcessing(false);
    
    // Reset tools fields
    setTestArchivePath("");
    setRepairCorruptedPath("");
    setRepairOutputPath("");
    setValidateArchivePath("");
    setExtractFirstVolume("");
    setExtractOutputDir("");
    
    toast.info("Form Reset", "All fields cleared");
  };
  
  // Tool operation handlers
  const handleTestArchive = async () => {
    setIsProcessing(true);
    const archivePath = testArchivePath();
    
    const activity = createActivity(
      "tool",
      archivePath,
      1,
      { operation: "Test Archive" }
    );
    
    props.onActivityCreate?.(activity);
    
    try {
      const isValid = await testArchive(archivePath);
      
      props.onActivityUpdate?.(activity.id, completeActivity(activity));
      
      if (isValid) {
        toast.success("Archive Test Passed", `${archivePath.split('/').pop()} is valid`);
      } else {
        toast.warning("Archive Test Failed", `${archivePath.split('/').pop()} has integrity issues`);
      }
      
      // Clear form after operation
      setTestArchivePath("");
    } catch (error: any) {
      props.onActivityUpdate?.(activity.id, failActivity(activity, error.message || String(error)));
      toast.error("Test Failed", error.message || String(error));
    } finally {
      setIsProcessing(false);
    }
  };
  
  const handleRepairArchive = async () => {
    setIsProcessing(true);
    const corruptedPath = repairCorruptedPath();
    const repairedPath = repairOutputPath();
    
    const activity = createActivity(
      "tool",
      `${corruptedPath} → ${repairedPath}`,
      1,
      { operation: "Repair Archive" }
    );
    
    props.onActivityCreate?.(activity);
    
    // Set up progress listener
    const unlisten = await listenToRepairProgress((prog) => {
      props.onActivityUpdate?.(activity.id, updateProgress(activity, {
        percent: prog.percent,
      }));
    });
    
    try {
      const result = await repairArchive(corruptedPath, repairedPath);
      
      props.onActivityUpdate?.(activity.id, completeActivity(activity));
      toast.success("Archive Repaired", `Saved to: ${result.split('/').pop()}`);
      
      // Clear form after operation
      setRepairCorruptedPath("");
      setRepairOutputPath("");
    } catch (error: any) {
      props.onActivityUpdate?.(activity.id, failActivity(activity, error.message || String(error)));
      toast.error("Repair Failed", error.message || String(error));
    } finally {
      unlisten();
      setIsProcessing(false);
    }
  };
  
  const handleValidateArchive = async () => {
    setIsProcessing(true);
    const archivePath = validateArchivePath();
    
    const activity = createActivity(
      "tool",
      archivePath,
      1,
      { operation: "Validate Archive" }
    );
    
    props.onActivityCreate?.(activity);
    
    try {
      const validation = await validateArchive(archivePath);
      
      props.onActivityUpdate?.(activity.id, completeActivity(activity));
      
      if (validation.isValid) {
        toast.success("Validation Passed", `${archivePath.split('/').pop()} structure is valid`);
      } else {
        toast.error("Validation Failed", validation.errorMessage || "Archive has structural errors");
      }
      
      // Clear form after operation
      setValidateArchivePath("");
    } catch (error: any) {
      props.onActivityUpdate?.(activity.id, failActivity(activity, error.message || String(error)));
      toast.error("Validation Failed", error.message || String(error));
    } finally {
      setIsProcessing(false);
    }
  };
  
  const handleExtractSplit = async () => {
    setIsProcessing(true);
    const firstVolume = extractFirstVolume();
    const outputDir = extractOutputDir();
    
    const activity = createActivity(
      "tool",
      `${firstVolume} → ${outputDir}`,
      1,
      { operation: "Extract Split Archive" }
    );
    
    props.onActivityCreate?.(activity);
    
    // Set up progress listener
    const unlisten = await listenToSplitExtractProgress((prog) => {
      props.onActivityUpdate?.(activity.id, updateProgress(activity, {
        percent: prog.percent,
      }));
    });
    
    try {
      const result = await extractSplitArchive(firstVolume, outputDir);
      
      props.onActivityUpdate?.(activity.id, completeActivity(activity));
      toast.success("Extraction Complete", `Files extracted to: ${result.split('/').pop()}`);
      
      // Clear form after operation
      setExtractFirstVolume("");
      setExtractOutputDir("");
    } catch (error: any) {
      props.onActivityUpdate?.(activity.id, failActivity(activity, error.message || String(error)));
      toast.error("Extraction Failed", error.message || String(error));
    } finally {
      unlisten();
      setIsProcessing(false);
    }
  };
  
  const handleToolAction = async () => {
    const currentTab = toolsTab();
    
    switch (currentTab) {
      case "test":
        await handleTestArchive();
        break;
      case "repair":
        await handleRepairArchive();
        break;
      case "validate":
        await handleValidateArchive();
        break;
      case "extract":
        await handleExtractSplit();
        break;
    }
  };
  
  // === Render ===
  
  return (
    <>
      <div class="flex flex-col h-full bg-bg">
        {/* Header */}
        <div class="panel-header">
          <h2 class="text-lg font-semibold text-txt">Export & Archive</h2>
          <div class="flex items-center gap-2">
            <Show when={props.onClose}>
              <button
                class="icon-btn-sm"
                onClick={props.onClose}
                title="Close"
              >
                <HiOutlineXMark class="w-4 h-4" />
              </button>
            </Show>
          </div>
        </div>
      
      {/* Mode Selector */}
      <div class="p-4 border-b border-border">
        <div class="flex gap-2 items-center justify-between">
          <div class="flex gap-2">
            <button
              class={mode() === "export" ? "btn-sm-primary" : "btn-sm"}
              onClick={() => setMode("export")}
              title="Export files with optional forensic features"
            >
              <HiOutlineArrowUpTray class="w-4 h-4" />
              Export
            </button>
            
            <button
              class={mode() === "archive" ? "btn-sm-primary" : "btn-sm"}
              onClick={() => setMode("archive")}
              title="Create 7z archive"
            >
              <HiOutlineArchiveBox class="w-4 h-4" />
              Archive
            </button>
            
            <button
              class={mode() === "tools" ? "btn-sm-primary" : "btn-sm"}
              onClick={() => setMode("tools")}
              title="Archive Tools (Test, Repair, Validate, Extract)"
            >
              <HiOutlineWrench class="w-4 h-4" />
              Tools
            </button>
          </div>
          
          {/* Clear Form Button */}
          <button
            class="btn-sm"
            onClick={handleReset}
            title="Clear all form fields"
          >
            <HiOutlineXMark class="w-4 h-4" />
            Clear
          </button>
        </div>
        
        {/* Mode Description */}
        <div class="mt-2 text-xs text-txt-secondary">
          <Show when={mode() === "export"}>
            Copy files with optional hashing, verification, and forensic manifests
          </Show>
          <Show when={mode() === "archive"}>
            Create encrypted 7z archive with compression
          </Show>
          <Show when={mode() === "tools"}>
            Test, repair, validate, or extract split archives
          </Show>
        </div>
      </div>
      
      {/* Content */}
      <div class="flex-1 overflow-y-auto p-4 space-y-4">
        {/* Show Export/Archive UI */}
        <Show when={mode() !== "tools"}>
        {/* Source Files */}
        <div class="space-y-2">
          <label class="label">Source Files</label>
          <div class="flex gap-2">
            <button class="btn-sm" onClick={handleAddSources}>
              <HiOutlineFolderOpen class="w-4 h-4" />
              Add Files
            </button>
            <button class="btn-sm" onClick={handleAddFolder}>
              <HiOutlineFolderOpen class="w-4 h-4" />
              Add Folder
            </button>
          </div>
          
          {/* Source List */}
          <Show when={sources().length > 0}>
            <div class="space-y-1 mt-2">
              <For each={sources()}>
                {(source, index) => (
                  <div class="flex items-center gap-2 p-2 bg-bg-secondary rounded text-sm">
                    <span class="flex-1 truncate text-txt" title={source}>
                      {source.split('/').pop() || source}
                    </span>
                    <button
                      class="icon-btn-sm"
                      onClick={() => handleRemoveSource(index())}
                      title="Remove"
                    >
                      <HiOutlineXMark class="w-3 h-3" />
                    </button>
                  </div>
                )}
              </For>
            </div>
          </Show>
          
          <Show when={sources().length === 0}>
            <div class="text-sm text-txt-muted italic">
              No files selected
            </div>
          </Show>
        </div>
        
        {/* Destination */}
        <div class="space-y-2">
          <label class="label">Destination</label>
          <div class="flex gap-2">
            <input
              class="input flex-1"
              type="text"
              value={destination()}
              placeholder="Select destination folder..."
              readOnly
            />
            <button class="btn-sm" onClick={handleSelectDestination}>
              <HiOutlineFolderOpen class="w-4 h-4" />
              Browse
            </button>
          </div>
        </div>
        
        {/* Export Options */}
        <Show when={mode() === "export"}>
          <ExportMode
            exportName={exportName}
            setExportName={setExportName}
            computeHashes={computeHashes}
            setComputeHashes={setComputeHashes}
            verifyAfterCopy={verifyAfterCopy}
            setVerifyAfterCopy={setVerifyAfterCopy}
            generateJsonManifest={generateJsonManifest}
            setGenerateJsonManifest={setGenerateJsonManifest}
            generateTxtReport={generateTxtReport}
            setGenerateTxtReport={setGenerateTxtReport}
          />
        </Show>
        
        {/* Archive-Specific Options */}
        <Show when={mode() === "archive"}>
          <ArchiveMode
            archiveName={archiveName}
            setArchiveName={setArchiveName}
            compressionLevel={compressionLevel}
            setCompressionLevel={setCompressionLevel}
            estimatedUncompressed={estimatedUncompressed}
            estimatedCompressed={estimatedCompressed}
            password={password}
            setPassword={setPassword}
            showPassword={showPassword}
            setShowPassword={setShowPassword}
            showAdvanced={showAdvanced}
            setShowAdvanced={setShowAdvanced}
            solid={solid}
            setSolid={setSolid}
            numThreads={numThreads}
            setNumThreads={setNumThreads}
            splitSizeMb={splitSizeMb}
            setSplitSizeMb={setSplitSizeMb}
          />
        </Show>
        </Show>
        
        {/* Show Archive Tools UI */}
        <Show when={mode() === "tools"}>
          <ToolsMode
            toolsTab={toolsTab}
            setToolsTab={setToolsTab}
            testArchivePath={testArchivePath}
            setTestArchivePath={setTestArchivePath}
            repairCorruptedPath={repairCorruptedPath}
            setRepairCorruptedPath={setRepairCorruptedPath}
            repairOutputPath={repairOutputPath}
            setRepairOutputPath={setRepairOutputPath}
            validateArchivePath={validateArchivePath}
            setValidateArchivePath={setValidateArchivePath}
            extractFirstVolume={extractFirstVolume}
            setExtractFirstVolume={setExtractFirstVolume}
            extractOutputDir={extractOutputDir}
            setExtractOutputDir={setExtractOutputDir}
          />
        </Show>
      </div>
      
      {/* Footer */}
      <div class="p-4 border-t border-border flex justify-between items-center">
        <Show when={mode() !== "tools"}>
          <div class="text-xs text-txt-muted">
            {sources().length} item{sources().length !== 1 ? 's' : ''} selected
          </div>
          
          <button
            class="btn-sm-primary"
            onClick={handleStart}
            disabled={isProcessing() || sources().length === 0 || !destination()}
          >
            <Show when={!isProcessing()} fallback={<span>Processing...</span>}>
              <HiOutlinePlay class="w-4 h-4" />
              Start {mode() === "export" ? "Export" : "Archive"}
            </Show>
          </button>
        </Show>
        
        <Show when={mode() === "tools"}>
          <div class="flex-1" />
          <button
            class="btn-sm-primary"
            onClick={handleToolAction}
            disabled={
              isProcessing() ||
              (toolsTab() === "test" && !testArchivePath()) ||
              (toolsTab() === "repair" && (!repairCorruptedPath() || !repairOutputPath())) ||
              (toolsTab() === "validate" && !validateArchivePath()) ||
              (toolsTab() === "extract" && (!extractFirstVolume() || !extractOutputDir()))
            }
          >
            <Show when={!isProcessing()} fallback={<span>Processing...</span>}>
              <HiOutlinePlay class="w-4 h-4" />
              {toolsTab() === "test" && "Test Archive"}
              {toolsTab() === "repair" && "Repair Archive"}
              {toolsTab() === "validate" && "Validate Archive"}
              {toolsTab() === "extract" && "Extract Archive"}
            </Show>
          </button>
        </Show>
      </div>
    </div>
    </>
  );
}
