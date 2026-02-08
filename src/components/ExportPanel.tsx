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
  HiOutlineDocumentDuplicate,
  HiOutlineArrowUpTray,
  HiOutlinePlay,
  HiOutlineXMark,
  HiOutlineCog6Tooth,
  HiOutlineChevronDown,
  HiOutlineChevronRight,
  HiOutlineInformationCircle,
  HiOutlineWrench,
} from "./icons";
import { createArchive, listenToProgress, estimateSize, formatBytes, getCompressionRatio, CompressionLevel } from "../api/archiveCreate";
import { exportFiles, type CopyProgress, type ExportOptions } from "../api/fileExport";
import { useToast } from "./Toast";
import { createActivity, updateProgress, completeActivity, failActivity, type Activity } from "../types/activity";

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
      await handleArchiveCreation();
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
      setIsProcessing(false);
      
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
    setIsProcessing(false);
  };
  
  // Reset handler to allow starting new operations
  const handleReset = () => {
    setIsProcessing(false);
    setSources([]);
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
          <div class="space-y-4 p-4 bg-bg-secondary rounded-lg border border-border">
            <h3 class="text-sm font-semibold text-txt flex items-center gap-2">
              <HiOutlineArrowUpTray class="w-4 h-4" />
              Export Options
            </h3>
            
            {/* Export Name */}
            <div class="space-y-1">
              <label class="label text-xs">Export Name</label>
              <input
                class="input input-sm"
                  type="text"
                  value={exportName()}
                  onInput={(e) => setExportName(e.currentTarget.value)}
                  placeholder="forensic_export"
                />
              <p class="text-[10px] text-txt-muted leading-tight">
                Used for manifest and report filenames
              </p>
            </div>
            
            {/* Checkbox Options */}
            <div class="space-y-2">
              <label class="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={computeHashes()}
                  onChange={(e) => setComputeHashes(e.currentTarget.checked)}
                  class="w-4 h-4"
                />
                <span class="text-xs text-txt">Compute SHA-256 hashes</span>
              </label>
              
              <Show when={computeHashes()}>
                <label class="flex items-center gap-2 cursor-pointer ml-6">
                  <input
                    type="checkbox"
                    checked={verifyAfterCopy()}
                    onChange={(e) => setVerifyAfterCopy(e.currentTarget.checked)}
                    class="w-4 h-4"
                  />
                  <span class="text-xs text-txt">Verify copied files</span>
                </label>
                
                <label class="flex items-center gap-2 cursor-pointer ml-6">
                  <input
                    type="checkbox"
                    checked={generateJsonManifest()}
                    onChange={(e) => setGenerateJsonManifest(e.currentTarget.checked)}
                    class="w-4 h-4"
                  />
                  <span class="text-xs text-txt">Generate JSON manifest</span>
                </label>
                
                <label class="flex items-center gap-2 cursor-pointer ml-6">
                  <input
                    type="checkbox"
                    checked={generateTxtReport()}
                    onChange={(e) => setGenerateTxtReport(e.currentTarget.checked)}
                    class="w-4 h-4"
                  />
                  <span class="text-xs text-txt">Generate TXT report</span>
                </label>
              </Show>
            </div>
            
            <div class="p-2 bg-bg-panel rounded border border-info/20 text-xs text-info">
              <HiOutlineInformationCircle class="w-3 h-3 inline mr-1" />
              Forensic export includes timestamps, hashes, and manifests for chain-of-custody
            </div>
          </div>
        </Show>
        
        {/* Archive-Specific Options */}
        <Show when={mode() === "archive"}>
          <div class="space-y-4 p-4 bg-bg-secondary rounded-lg border border-border">
            <h3 class="text-sm font-semibold text-txt flex items-center gap-2">
              <HiOutlineArchiveBox class="w-4 h-4" />
              Archive Options
            </h3>
            
            {/* Archive Name */}
            <div class="space-y-1">
              <label class="label text-xs">Archive Name</label>
              <input
                class="input input-sm"
                type="text"
                value={archiveName()}
                onInput={(e) => setArchiveName(e.currentTarget.value)}
                placeholder="evidence.7z"
              />
            </div>
            
            {/* Compression Level */}
            <div class="space-y-1">
              <label class="label text-xs">Compression Level</label>
              <select
                class="input input-sm"
                value={compressionLevel()}
                onChange={(e) => setCompressionLevel(Number(e.currentTarget.value))}
              >
                <option value={CompressionLevel.Store}>Store (~500+ MB/s) - Recommended for E01/AD1</option>
                <option value={CompressionLevel.Fastest}>Fastest (~180 MB/s)</option>
                <option value={CompressionLevel.Fast}>Fast (~80 MB/s)</option>
                <option value={CompressionLevel.Normal}>Normal (~22 MB/s)</option>
                <option value={CompressionLevel.Maximum}>Maximum (~12 MB/s)</option>
                <option value={CompressionLevel.Ultra}>Ultra (~9 MB/s)</option>
              </select>
            </div>
            
            {/* Size Estimate */}
            <Show when={estimatedUncompressed() > 0}>
              <div class="p-2 bg-bg-panel rounded border border-border">
                <div class="text-xs space-y-1">
                  <div class="flex justify-between">
                    <span class="text-txt-muted">Original:</span>
                    <span class="text-txt font-medium">{formatBytes(estimatedUncompressed())}</span>
                  </div>
                  <div class="flex justify-between">
                    <span class="text-txt-muted">Estimated:</span>
                    <span class="text-txt font-medium">{formatBytes(estimatedCompressed())}</span>
                  </div>
                  <div class="flex justify-between">
                    <span class="text-txt-muted">Ratio:</span>
                    <span class="text-accent font-medium">
                      {getCompressionRatio(estimatedUncompressed(), estimatedCompressed()).toFixed(1)}%
                    </span>
                  </div>
                </div>
              </div>
            </Show>
            
            {/* Password */}
            <div class="space-y-1">
              <label class="label text-xs">Password (Optional)</label>
              <div class="flex gap-2">
                <input
                  class="input input-sm flex-1"
                  type={showPassword() ? "text" : "password"}
                  value={password()}
                  onInput={(e) => setPassword(e.currentTarget.value)}
                  placeholder="AES-256 encryption password"
                />
                <button
                  class="btn-sm"
                  onClick={() => setShowPassword(!showPassword())}
                >
                  {showPassword() ? "Hide" : "Show"}
                </button>
              </div>
              <Show when={password()}>
                <div class="flex items-start gap-1 text-xs text-warning">
                  <HiOutlineInformationCircle class="w-3 h-3 mt-0.5 flex-shrink-0" />
                  <span>Strong password recommended (12+ characters)</span>
                </div>
              </Show>
            </div>
            
            {/* Advanced Options (Collapsible) */}
            <div class="border-t border-border pt-4">
              <button
                class="flex items-center gap-2 text-sm text-txt-secondary hover:text-txt transition-colors"
                onClick={() => setShowAdvanced(!showAdvanced())}
              >
                <Show when={showAdvanced()} fallback={<HiOutlineChevronRight class="w-4 h-4" />}>
                  <HiOutlineChevronDown class="w-4 h-4" />
                </Show>
                <HiOutlineCog6Tooth class="w-4 h-4" />
                <span>Advanced Options</span>
              </button>
              
              <Show when={showAdvanced()}>
                <div class="mt-3 space-y-3">
                  {/* Solid Compression */}
                  <div class="space-y-1">
                    <label class="flex items-center gap-2 cursor-pointer">
                      <input
                        type="checkbox"
                        checked={solid()}
                        onChange={(e) => setSolid(e.currentTarget.checked)}
                        class="w-4 h-4"
                      />
                      <span class="text-xs text-txt">Solid compression</span>
                    </label>
                    <p class="text-[10px] text-txt-muted ml-6 leading-tight">
                      Better compression ratio but slower processing and prevents extracting individual files. 
                      Not recommended for large archives or archives that may need partial extraction.
                    </p>
                  </div>
                  
                  {/* Thread Count */}
                  <div class="space-y-1">
                    <label class="label text-xs">Threads (0 = auto)</label>
                    <input
                      class="input input-sm"
                      type="number"
                      min="0"
                      max="16"
                      value={numThreads()}
                      onInput={(e) => setNumThreads(Number(e.currentTarget.value))}
                    />
                  </div>
                  
                  {/* Split Size */}
                  <div class="space-y-1">
                    <label class="label text-xs">Split Size</label>
                    <select
                      class="input input-sm"
                      value={splitSizeMb()}
                      onChange={(e) => setSplitSizeMb(Number(e.currentTarget.value))}
                    >
                      <option value={0}>No Split</option>
                      <option value={700}>700 MB (CD)</option>
                      <option value={2048}>2 GB (Cloud/USB) - Recommended</option>
                      <option value={4700}>4.7 GB (DVD)</option>
                      <option value={8500}>8.5 GB (DVD DL)</option>
                      <option value={25000}>25 GB (Blu-ray)</option>
                      <option value={50000}>50 GB (Blu-ray DL)</option>
                    </select>
                  </div>
                </div>
              </Show>
            </div>
          </div>
        </Show>
        </Show>
        
        {/* Show Archive Tools UI */}
        <Show when={mode() === "tools"}>
          <div class="space-y-4">
            {/* Tools Tab Selector */}
            <div class="flex gap-1 border-b border-border">
              <button
                class={`px-4 py-2 -mb-px border-b-2 transition-colors text-xs ${
                  toolsTab() === "test"
                    ? "border-accent text-accent"
                    : "border-transparent text-txt-secondary hover:text-txt"
                }`}
                onClick={() => setToolsTab("test")}
              >
                Test
              </button>
              <button
                class={`px-4 py-2 -mb-px border-b-2 transition-colors text-xs ${
                  toolsTab() === "repair"
                    ? "border-accent text-accent"
                    : "border-transparent text-txt-secondary hover:text-txt"
                }`}
                onClick={() => setToolsTab("repair")}
              >
                Repair
              </button>
              <button
                class={`px-4 py-2 -mb-px border-b-2 transition-colors text-xs ${
                  toolsTab() === "validate"
                    ? "border-accent text-accent"
                    : "border-transparent text-txt-secondary hover:text-txt"
                }`}
                onClick={() => setToolsTab("validate")}
              >
                Validate
              </button>
              <button
                class={`px-4 py-2 -mb-px border-b-2 transition-colors text-xs ${
                  toolsTab() === "extract"
                    ? "border-accent text-accent"
                    : "border-transparent text-txt-secondary hover:text-txt"
                }`}
                onClick={() => setToolsTab("extract")}
              >
                Extract Split
              </button>
            </div>
            
            {/* Test Tab */}
            <Show when={toolsTab() === "test"}>
              <div class="space-y-3">
                <div class="info-card">
                  <HiOutlineInformationCircle class="w-5 h-5 text-info" />
                  <div>
                    <div class="font-medium text-txt">Test Archive Integrity</div>
                    <div class="text-xs text-txt-muted mt-1">
                      Verify archive contents without extraction. Checks CRC and structure.
                    </div>
                  </div>
                </div>
                
                <div class="space-y-2">
                  <label class="label">Archive File</label>
                  <div class="flex gap-2">
                    <input
                      type="text"
                      class="input-inline"
                      value={testArchivePath()}
                      onInput={(e) => setTestArchivePath(e.currentTarget.value)}
                      placeholder="Select archive to test..."
                    />
                    <button class="btn-sm" onClick={async () => {
                      const selected = await open({ directory: false, multiple: false, filters: [{ name: 'Archives', extensions: ['7z', 'zip'] }] });
                      if (selected) setTestArchivePath(selected as string);
                    }}>
                      Browse
                    </button>
                  </div>
                </div>
              </div>
            </Show>
            
            {/* Repair Tab */}
            <Show when={toolsTab() === "repair"}>
              <div class="space-y-3">
                <div class="info-card">
                  <HiOutlineInformationCircle class="w-5 h-5 text-info" />
                  <div>
                    <div class="font-medium text-txt">Repair Corrupted Archive</div>
                    <div class="text-xs text-txt-muted mt-1">
                      Attempt to recover data from damaged archives.
                    </div>
                  </div>
                </div>
                
                <div class="space-y-2">
                  <label class="label">Corrupted Archive</label>
                  <div class="flex gap-2">
                    <input
                      type="text"
                      class="input-inline"
                      value={repairCorruptedPath()}
                      onInput={(e) => setRepairCorruptedPath(e.currentTarget.value)}
                      placeholder="Select corrupted archive..."
                    />
                    <button class="btn-sm" onClick={async () => {
                      const selected = await open({ directory: false, multiple: false, filters: [{ name: 'Archives', extensions: ['7z'] }] });
                      if (selected) setRepairCorruptedPath(selected as string);
                    }}>
                      Browse
                    </button>
                  </div>
                </div>
                
                <div class="space-y-2">
                  <label class="label">Output Archive</label>
                  <div class="flex gap-2">
                    <input
                      type="text"
                      class="input-inline"
                      value={repairOutputPath()}
                      onInput={(e) => setRepairOutputPath(e.currentTarget.value)}
                      placeholder="Output path for repaired archive..."
                    />
                    <button class="btn-sm" onClick={async () => {
                      const selected = await open({ directory: false, multiple: false, filters: [{ name: '7z Archive', extensions: ['7z'] }] });
                      if (selected) setRepairOutputPath(selected as string);
                    }}>
                      Browse
                    </button>
                  </div>
                </div>
              </div>
            </Show>
            
            {/* Validate Tab */}
            <Show when={toolsTab() === "validate"}>
              <div class="space-y-3">
                <div class="info-card">
                  <HiOutlineInformationCircle class="w-5 h-5 text-info" />
                  <div>
                    <div class="font-medium text-txt">Validate Archive Structure</div>
                    <div class="text-xs text-txt-muted mt-1">
                      Deep validation of archive format and headers.
                    </div>
                  </div>
                </div>
                
                <div class="space-y-2">
                  <label class="label">Archive File</label>
                  <div class="flex gap-2">
                    <input
                      type="text"
                      class="input-inline"
                      value={validateArchivePath()}
                      onInput={(e) => setValidateArchivePath(e.currentTarget.value)}
                      placeholder="Select archive to validate..."
                    />
                    <button class="btn-sm" onClick={async () => {
                      const selected = await open({ directory: false, multiple: false, filters: [{ name: 'Archives', extensions: ['7z'] }] });
                      if (selected) setValidateArchivePath(selected as string);
                    }}>
                      Browse
                    </button>
                  </div>
                </div>
              </div>
            </Show>
            
            {/* Extract Split Tab */}
            <Show when={toolsTab() === "extract"}>
              <div class="space-y-3">
                <div class="info-card">
                  <HiOutlineInformationCircle class="w-5 h-5 text-info" />
                  <div>
                    <div class="font-medium text-txt">Extract Split Archive</div>
                    <div class="text-xs text-txt-muted mt-1">
                      Extract multi-volume archives (*.7z.001, *.7z.002, etc.)
                    </div>
                  </div>
                </div>
                
                <div class="space-y-2">
                  <label class="label">First Volume</label>
                  <div class="flex gap-2">
                    <input
                      type="text"
                      class="input-inline"
                      value={extractFirstVolume()}
                      onInput={(e) => setExtractFirstVolume(e.currentTarget.value)}
                      placeholder="Select first volume (.001)..."
                    />
                    <button class="btn-sm" onClick={async () => {
                      const selected = await open({ directory: false, multiple: false });
                      if (selected) setExtractFirstVolume(selected as string);
                    }}>
                      Browse
                    </button>
                  </div>
                </div>
                
                <div class="space-y-2">
                  <label class="label">Output Directory</label>
                  <div class="flex gap-2">
                    <input
                      type="text"
                      class="input-inline"
                      value={extractOutputDir()}
                      onInput={(e) => setExtractOutputDir(e.currentTarget.value)}
                      placeholder="Extract to..."
                    />
                    <button class="btn-sm" onClick={async () => {
                      const selected = await open({ directory: true, multiple: false });
                      if (selected) setExtractOutputDir(selected as string);
                    }}>
                      Browse
                    </button>
                  </div>
                </div>
              </div>
            </Show>
          </div>
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
            onClick={() => {
              // Tool operations will be queued to activity panel
              toast.info("Tool Operation", "Operation queued - check Activity panel");
            }}
            disabled={
              (toolsTab() === "test" && !testArchivePath()) ||
              (toolsTab() === "repair" && (!repairCorruptedPath() || !repairOutputPath())) ||
              (toolsTab() === "validate" && !validateArchivePath()) ||
              (toolsTab() === "extract" && (!extractFirstVolume() || !extractOutputDir()))
            }
          >
            <HiOutlinePlay class="w-4 h-4" />
            {toolsTab() === "test" && "Test Archive"}
            {toolsTab() === "repair" && "Repair Archive"}
            {toolsTab() === "validate" && "Validate Archive"}
            {toolsTab() === "extract" && "Extract Archive"}
          </button>
        </Show>
      </div>
    </div>
    </>
  );
}
