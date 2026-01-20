// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * FileTreeNode Component
 * 
 * Renders a single node in the file tree preview with expand/collapse,
 * file type icons, and progress indicators.
 */

import { Show, For, JSX } from "solid-js";
import {
  HiOutlineFolderOpen,
  HiOutlineFolder,
  HiOutlineDocument,
  HiOutlineChevronRight,
  HiOutlineChevronDown,
  HiOutlineCheckCircle,
  HiOutlineExclamationTriangle,
  HiOutlinePhoto,
  HiOutlineFilm,
  HiOutlineMusicalNote,
  HiOutlineDocumentText,
  HiOutlineCodeBracket,
  HiOutlineCircleStack,
  HiOutlineCpuChip,
  HiOutlineArchiveBox,
} from "../icons";
import type { FileTreeNodeProps, FileType, ContainerType } from "./types";

// =============================================================================
// Icon Helper
// =============================================================================

/** Get icon for file type */
export function getFileIcon(fileType: FileType, containerType: ContainerType): JSX.Element {
  if (containerType !== "unknown") {
    return <HiOutlineArchiveBox class={`w-3.5 h-3.5 text-accent`} />;
  }
  switch (fileType) {
    case "image": return <HiOutlinePhoto class={`w-3.5 h-3.5 text-purple-400`} />;
    case "video": return <HiOutlineFilm class={`w-3.5 h-3.5 text-pink-400`} />;
    case "audio": return <HiOutlineMusicalNote class={`w-3.5 h-3.5 text-green-400`} />;
    case "document": return <HiOutlineDocumentText class={`w-3.5 h-3.5 text-blue-400`} />;
    case "code": return <HiOutlineCodeBracket class={`w-3.5 h-3.5 text-yellow-400`} />;
    case "database": return <HiOutlineCircleStack class={`w-3.5 h-3.5 text-orange-400`} />;
    case "archive": return <HiOutlineCpuChip class={`w-3.5 h-3.5 text-cyan-400`} />;
    default: return <HiOutlineDocument class={`w-3.5 h-3.5 text-txt-secondary`} />;
  }
}

// =============================================================================
// Component
// =============================================================================

export function FileTreeNodeComponent(props: FileTreeNodeProps) {
  const isCurrentFile = () => props.currentFile === props.node.path;
  const fileStatus = () => props.fileProgress?.get(props.node.path);
  
  return (
    <div>
      <div 
        class={`flex items-center gap-1.5 py-0.5 px-1 rounded cursor-default hover:bg-bg-hover/50 ${
          isCurrentFile() ? 'bg-accent/20' : ''
        }`}
        style={{ "padding-left": `${props.depth * 12 + 4}px` }}
      >
        {/* Expand/Collapse for directories */}
        <Show when={props.node.isDirectory}>
          <button 
            class={`w-3.5 h-3.5 flex items-center justify-center text-txt-muted hover:text-txt-tertiary`}
            onClick={() => props.onToggle(props.node.path)}
          >
            {props.node.expanded ? 
              <HiOutlineChevronDown class="w-2 h-2" /> : 
              <HiOutlineChevronRight class="w-2 h-2" />
            }
          </button>
        </Show>
        <Show when={!props.node.isDirectory}>
          <span class="w-4" />
        </Show>
        
        {/* Icon */}
        <Show when={props.node.isDirectory} fallback={
          getFileIcon(props.node.fileType, props.node.containerType)
        }>
          {props.node.expanded ? 
            <HiOutlineFolderOpen class={`w-3.5 h-3.5 text-amber-400`} /> :
            <HiOutlineFolder class={`w-3.5 h-3.5 text-amber-400`} />
          }
        </Show>
        
        {/* Name */}
        <span class={`flex-1 truncate text-[11px] leading-tight ${
          props.node.containerType !== "unknown" ? 'text-accent font-medium' : 'text-txt-tertiary'
        }`} title={props.node.path}>
          {props.node.name}
        </span>
        
        {/* Container badge */}
        <Show when={props.node.containerType !== "unknown"}>
          <span class={`text-[11px] leading-tight text-accent px-1 py-0.5 rounded bg-accent/10`}>
            {props.node.containerType.toUpperCase()}
          </span>
        </Show>
        
        {/* Size */}
        <Show when={!props.node.isDirectory && props.node.sizeFormatted}>
          <span class={`text-[11px] leading-tight text-txt-muted tabular-nums`}>
            {props.node.sizeFormatted}
          </span>
        </Show>
        
        {/* Progress indicator */}
        <Show when={fileStatus()}>
          {(status) => (
            <div class="flex items-center gap-1">
              <Show when={status().status === "transferring" || status().status === "hashing"}>
                <div class="w-16 h-1.5 bg-bg-hover rounded-full overflow-hidden">
                  <div 
                    class={`h-full transition-all duration-150 ${
                      status().status === "hashing" ? 'bg-purple-500' : 'bg-accent'
                    }`}
                    style={{ width: `${status().progress}%` }}
                  />
                </div>
              </Show>
              <Show when={status().status === "completed"}>
                <HiOutlineCheckCircle class={`w-3 h-3 text-green-400`} />
              </Show>
              <Show when={status().status === "failed"}>
                <HiOutlineExclamationTriangle class={`w-3 h-3 text-red-400`} />
              </Show>
            </div>
          )}
        </Show>
      </div>
      
      {/* Children */}
      <Show when={props.node.isDirectory && props.node.expanded}>
        <For each={props.node.children}>
          {(child) => (
            <FileTreeNodeComponent 
              node={child} 
              depth={props.depth + 1}
              onToggle={props.onToggle}
              currentFile={props.currentFile}
              fileProgress={props.fileProgress}
            />
          )}
        </For>
      </Show>
    </div>
  );
}
