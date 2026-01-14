// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ContainerEntryViewer - View file content from within forensic containers
 * 
 * This component reads and displays the content of files stored inside
 * forensic containers:
 * - AD1: Uses container_read_file_data_v2 (V2 implementation - 50x faster)
 * - E01/Raw (VFS): Uses vfs_read_file
 * 
 * Migration to V2:
 * - V2 reads entire file at once (no chunking needed)
 * - Uses address-based reading (itemAddr from SelectedEntry)
 * - Falls back to OLD API for backward compatibility
 */

import { createSignal, createEffect, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { HiOutlineDocument, HiOutlineExclamationTriangle, HiOutlineArrowLeft } from "./icons";
import type { SelectedEntry } from "./EvidenceTree";
import { formatBytes } from "../utils";

interface ContainerEntryViewerProps {
  /** The selected entry to display */
  entry: SelectedEntry;
  /** View mode: hex or text */
  viewMode: "hex" | "text";
  /** Callback when user wants to go back/close this view */
  onBack?: () => void;
  /** Callback when user toggles view mode */
  onViewModeChange?: (mode: "hex" | "text") => void;
}

// Chunk sizes for reading data
const HEX_DISPLAY_SIZE = 4 * 1024; // Display 4KB at a time in hex view
const TEXT_DISPLAY_SIZE = 32 * 1024; // Display 32KB at a time in text view

export function ContainerEntryViewer(props: ContainerEntryViewerProps) {
  const [data, setData] = createSignal<Uint8Array | null>(null);
  const [textContent, setTextContent] = createSignal<string>("");
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [offset, setOffset] = createSignal(0);

  // Load data when entry changes
  createEffect(() => {
    const entry = props.entry;
    if (entry && !entry.isDir) {
      loadData(entry, 0);
    }
  });

  const loadData = async (entry: SelectedEntry, newOffset: number) => {
    setLoading(true);
    setError(null);
    setOffset(newOffset);

    try {
      let bytes: number[];
      const size = props.viewMode === "hex" ? HEX_DISPLAY_SIZE : TEXT_DISPLAY_SIZE;

      // Check if this is a VFS entry (E01, Raw, etc.)
      if (entry.isVfsEntry) {
        // Use VFS read command for E01/Raw containers
        bytes = await invoke<number[]>("vfs_read_file", {
          containerPath: entry.containerPath,
          filePath: entry.entryPath,
          offset: newOffset,
          length: size,
        });
      } else if (entry.itemAddr && newOffset === 0) {
        // AD1 V2: Read entire file using address-based API (50x faster!)
        console.log(`[ContainerEntryViewer] Using V2 API for ${entry.name} at addr ${entry.itemAddr}`);
        bytes = await invoke<number[]>("container_read_file_data_v2", {
          containerPath: entry.containerPath,
          itemAddr: entry.itemAddr,
        });
      } else if (entry.dataAddr && newOffset === 0) {
        // AD1 OLD: Read all data at once using address (fallback for legacy)
        console.log(`[ContainerEntryViewer] Using OLD API (dataAddr) for ${entry.name}`);
        bytes = await invoke<number[]>("container_read_entry_by_addr", {
          containerPath: entry.containerPath,
          dataAddr: entry.dataAddr,
          size: entry.size,
        });
      } else {
        // AD1 OLD: Fall back to path-based chunk reading (slowest)
        console.log(`[ContainerEntryViewer] Using OLD API (path-based) for ${entry.name}`);
        bytes = await invoke<number[]>("container_read_entry_chunk", {
          containerPath: entry.containerPath,
          entryPath: entry.entryPath,
          offset: newOffset,
          size,
        });
      }

      const uint8Array = new Uint8Array(bytes);
      setData(uint8Array);

      if (props.viewMode === "text") {
        // Try to decode as text
        try {
          const decoder = new TextDecoder("utf-8", { fatal: false });
          setTextContent(decoder.decode(uint8Array));
        } catch {
          setTextContent("[Binary content - unable to decode as text]");
        }
      }
    } catch (err) {
      console.error("Failed to load entry content:", err);
      setError(String(err));
      setData(null);
    } finally {
      setLoading(false);
    }
  };

  // Navigate to next/previous chunk
  const prevChunk = () => {
    const newOffset = Math.max(0, offset() - (props.viewMode === "hex" ? HEX_DISPLAY_SIZE : TEXT_DISPLAY_SIZE));
    loadData(props.entry, newOffset);
  };

  const nextChunk = () => {
    const newOffset = offset() + (props.viewMode === "hex" ? HEX_DISPLAY_SIZE : TEXT_DISPLAY_SIZE);
    if (newOffset < props.entry.size) {
      loadData(props.entry, newOffset);
    }
  };

  // Format hex view
  const formatHex = () => {
    const bytes = data();
    if (!bytes) return [];

    const lines: { offset: string; hex: string; ascii: string }[] = [];
    const bytesPerLine = 16;

    for (let i = 0; i < bytes.length; i += bytesPerLine) {
      const lineBytes = bytes.slice(i, i + bytesPerLine);
      const lineOffset = offset() + i;

      // Format offset
      const offsetStr = lineOffset.toString(16).padStart(8, "0").toUpperCase();

      // Format hex
      const hexParts: string[] = [];
      for (let j = 0; j < bytesPerLine; j++) {
        if (j < lineBytes.length) {
          hexParts.push(lineBytes[j].toString(16).padStart(2, "0").toUpperCase());
        } else {
          hexParts.push("  ");
        }
      }
      const hexStr = hexParts.join(" ");

      // Format ASCII
      let ascii = "";
      for (let j = 0; j < lineBytes.length; j++) {
        const byte = lineBytes[j];
        ascii += byte >= 32 && byte < 127 ? String.fromCharCode(byte) : ".";
      }

      lines.push({ offset: offsetStr, hex: hexStr, ascii });
    }

    return lines;
  };

  return (
    <div class="flex flex-col h-full bg-zinc-900">
      {/* Header */}
      <div class="panel-header gap-3">
        <Show when={props.onBack}>
          <button class="btn-text flex items-center gap-1" onClick={props.onBack} title="Back to file list">
            <HiOutlineArrowLeft class="w-3.5 h-3.5" /> Back
          </button>
        </Show>
        <div class="row flex-1 min-w-0">
          <span class="text-sm text-zinc-200 truncate flex items-center gap-1.5" title={props.entry.entryPath}>
            <HiOutlineDocument class="w-4 h-4 shrink-0" /> {props.entry.name}
          </span>
          <span class="text-xs text-zinc-500">{formatBytes(props.entry.size)}</span>
        </div>
        
        {/* View mode toggle */}
        <Show when={props.onViewModeChange}>
          <div class="flex items-center gap-0.5 bg-zinc-800 rounded border border-zinc-700">
            <button 
              class={`px-2 py-1 text-xs rounded ${props.viewMode === "hex" ? 'bg-cyan-600 text-white' : 'text-zinc-400 hover:text-zinc-200'}`}
              onClick={() => props.onViewModeChange?.("hex")}
              title="View as hex"
            >
              Hex
            </button>
            <button 
              class={`px-2 py-1 text-xs rounded ${props.viewMode === "text" ? 'bg-cyan-600 text-white' : 'text-zinc-400 hover:text-zinc-200'}`}
              onClick={() => props.onViewModeChange?.("text")}
              title="View as text"
            >
              Text
            </button>
          </div>
        </Show>
        
        <div class="flex items-center gap-2">
          <button 
            class="btn-text" 
            onClick={prevChunk} 
            disabled={offset() === 0 || loading()}
            title="Previous chunk"
          >
            ◀
          </button>
          <span class="text-xs text-zinc-500 font-mono">
            {formatBytes(offset())} / {formatBytes(props.entry.size)}
          </span>
          <button 
            class="btn-text" 
            onClick={nextChunk} 
            disabled={offset() + (props.viewMode === "hex" ? HEX_DISPLAY_SIZE : TEXT_DISPLAY_SIZE) >= props.entry.size || loading()}
            title="Next chunk"
          >
            ▶
          </button>
        </div>
      </div>

      {/* Content */}
      <div class="flex-1 overflow-auto p-2">
        <Show when={loading()}>
          <div class="flex items-center justify-center gap-2 py-8 text-zinc-500">
            <span class="animate-spin">⟳</span>
            Loading...
          </div>
        </Show>

        <Show when={error()}>
          <div class="error-alert">
            <HiOutlineExclamationTriangle class="w-5 h-5 shrink-0" />
            <span>{error()}</span>
          </div>
        </Show>

        <Show when={!loading() && !error() && data()}>
          <Show when={props.viewMode === "hex"} fallback={
            <pre class="font-mono text-sm text-zinc-300 whitespace-pre-wrap">{textContent()}</pre>
          }>
            <div class="font-mono text-xs">
              <div class="flex flex-col">
                {formatHex().map((line) => (
                  <div class="flex items-baseline py-0.5 hover:bg-zinc-800/50">
                    <span class="w-20 text-cyan-400 shrink-0">{line.offset}</span>
                    <span class="flex-1 text-zinc-400">{line.hex}</span>
                    <span class="w-20 text-zinc-500 shrink-0 pl-3 border-l border-zinc-700">{line.ascii}</span>
                  </div>
                ))}
              </div>
            </div>
          </Show>
        </Show>
      </div>
    </div>
  );
}

export default ContainerEntryViewer;
