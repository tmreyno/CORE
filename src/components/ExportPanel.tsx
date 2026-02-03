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
} from "./icons";
import { createArchive, listenToProgress, estimateSize, formatBytes, getCompressionRatio, CompressionLevel, type ArchiveCreateProgress } from "../api/archiveCreate";
import { copyFiles, exportFiles, type CopyProgress } from "../api/fileExport";
import { useToast } from "./Toast";
import { createActivity, updateProgress, completeActivity, failActivity, type Activity } from "../types/activity";

/** Export operation mode */
export type ExportMode = "copy" | "export" | "archive";

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
  const [mode, setMode] = createSignal<ExportMode>("copy");
  const [sources, setSources] = createSignal<string[]>(props.initialSources || []);
  const [destination, setDestination] = createSignal("");
  const [archiveName, setArchiveName] = createSignal("evidence.7z");
  const [isProcessing, setIsProcessing] = createSignal(false);
  const [progress, setProgress] = createSignal<ArchiveCreateProgress | null>(null);
  const [showAdvanced, setShowAdvanced] = createSignal(false);
  
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
    } else if (currentMode === "copy") {
      await handleCopy();
    } else if (currentMode === "export") {
      await handleExport();
    }
  };
  
  const handleArchiveCreation = async () => {
    setIsProcessing(true);
    setProgress(null);
    
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
      
      // Update local progress state
      setProgress(prog);
      
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
      setProgress(null);
      setIsProcessing(false);
      
    } catch (error: any) {
      // Handle immediate errors (before operation starts)
      unlisten();
      props.onActivityUpdate?.(activity.id, failActivity(activity, error.message || String(error)));
      toast.error("Archive Creation Failed", error.message || String(error));
      setIsProcessing(false);
    }
  };
  
  const handleCopy = async () => {
    setIsProcessing(true);
    
    // Create activity record for copy operation
    const activity = createActivity(
      "copy",
      destination(),
      sources().length,
      {}
    );
    
    props.onActivityCreate?.(activity);
    
    try {
      const result = await copyFiles(
        sources(),
        destination(),
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
      );
      
      props.onActivityUpdate?.(activity.id, completeActivity(activity));
      toast.success(
        "Copy Complete", 
        `Copied ${result.filesCopied} files (${formatBytes(result.bytesCopied)}) in ${(result.durationMs / 1000).toFixed(1)}s`
      );
      props.onComplete?.(destination());
      setSources([]);
    } catch (error: any) {
      props.onActivityUpdate?.(activity.id, failActivity(activity, error.message || String(error)));
      toast.error("Copy Failed", error.message || String(error));
    } finally {
      setIsProcessing(false);
    }
  };
  
  const handleExport = async () => {
    setIsProcessing(true);
    
    // Create activity record for forensic export
    const activity = createActivity(
      "export",
      destination(),
      sources().length,
      { includeHashes: true }
    );
    
    props.onActivityCreate?.(activity);
    
    // Generate export name from destination folder
    const exportName = destination().split('/').pop() || "forensic_export";
    
    try {
      const result = await exportFiles(
        sources(),
        destination(),
        exportName,
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
      );
      
      props.onActivityUpdate?.(activity.id, completeActivity(activity));
      
      const hashNote = result.metadata && result.metadata.length > 0
        ? `\nHash manifest: ${exportName}_manifest.json`
        : '';
      
      toast.success(
        "Forensic Export Complete", 
        `Exported ${result.filesCopied} files (${formatBytes(result.bytesCopied)}) in ${(result.durationMs / 1000).toFixed(1)}s${hashNote}`
      );
      props.onComplete?.(destination());
      setSources([]);
    } catch (error: any) {
      props.onActivityUpdate?.(activity.id, failActivity(activity, error.message || String(error)));
      toast.error("Export Failed", error.message || String(error));
    } finally {
      setIsProcessing(false);
    }
  };
  
  // === Render ===
  
  return (
    <div class="flex flex-col h-full bg-bg">
      {/* Header */}
      <div class="panel-header">
        <h2 class="text-lg font-semibold text-txt">Export & Archive</h2>
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
      
      {/* Mode Selector */}
      <div class="p-4 border-b border-border">
        <div class="flex gap-2">
          <button
            class={`btn btn-sm flex-1 ${mode() === "copy" ? "btn-primary" : "btn-secondary"}`}
            onClick={() => setMode("copy")}
            title="Copy files directly"
          >
            <HiOutlineDocumentDuplicate class="w-4 h-4" />
            Copy
          </button>
          
          <button
            class={`btn btn-sm flex-1 ${mode() === "export" ? "btn-primary" : "btn-secondary"}`}
            onClick={() => setMode("export")}
            title="Export with forensic metadata"
          >
            <HiOutlineArrowUpTray class="w-4 h-4" />
            Export
          </button>
          
          <button
            class={`btn btn-sm flex-1 ${mode() === "archive" ? "btn-primary" : "btn-secondary"}`}
            onClick={() => setMode("archive")}
            title="Create 7z archive"
          >
            <HiOutlineArchiveBox class="w-4 h-4" />
            Archive
          </button>
        </div>
        
        {/* Mode Description */}
        <div class="mt-2 text-xs text-txt-secondary">
          <Show when={mode() === "copy"}>
            Direct file/folder copy with verification
          </Show>
          <Show when={mode() === "export"}>
            Copy with forensic metadata (hashes, timestamps, chain-of-custody)
          </Show>
          <Show when={mode() === "archive"}>
            Create encrypted 7z archive with compression
          </Show>
        </div>
      </div>
      
      {/* Content */}
      <div class="flex-1 overflow-y-auto p-4 space-y-4">
        {/* Source Files */}
        <div class="space-y-2">
          <label class="label">Source Files</label>
          <div class="flex gap-2">
            <button class="btn btn-sm" onClick={handleAddSources}>
              <HiOutlineFolderOpen class="w-4 h-4" />
              Add Files
            </button>
            <button class="btn btn-sm" onClick={handleAddFolder}>
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
            <button class="btn btn-sm" onClick={handleSelectDestination}>
              <HiOutlineFolderOpen class="w-4 h-4" />
              Browse
            </button>
          </div>
        </div>
        
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
                  class="btn btn-sm"
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
      </div>
      
      {/* Footer */}
      <div class="p-4 border-t border-border flex justify-between items-center">
        <div class="text-xs text-txt-muted">
          {sources().length} item{sources().length !== 1 ? 's' : ''} selected
        </div>
        
        <button
          class="btn btn-primary"
          onClick={handleStart}
          disabled={isProcessing() || sources().length === 0 || !destination()}
        >
          <Show when={!isProcessing()} fallback={<span>Processing...</span>}>
            <HiOutlinePlay class="w-4 h-4" />
            Start {mode() === "copy" ? "Copy" : mode() === "export" ? "Export" : "Archive"}
          </Show>
        </button>
      </div>
    </div>
  );
}
