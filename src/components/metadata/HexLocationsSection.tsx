// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show } from "solid-js";
import {
  HiOutlineMapPin,
  HiOutlineFolder,
  HiOutlineDocument,
  HiOutlineClipboardDocument,
  HiOutlineCircleStack,
  HiOutlineArchiveBox,
  HiOutlineArrowRightOnRectangle,
  HiOutlineUserGroup,
} from "../icons";
import type { SelectedEntry } from "../EvidenceTree";
import { formatBytes, formatOffsetLabel, getBasename } from "../../utils";

interface HexLocationsSectionProps {
  selectedEntry?: SelectedEntry | null;
  isExpanded: (category: string) => boolean;
  toggleCategory: (category: string) => void;
  handleRowClick: (offset: number | null | undefined) => void;
  categoryHeader: string;
  rowBase: string;
  rowGrid: string;
  rowClickable: string;
  keyStyle: string;
  valueStyle: string;
  offsetStyle: string;
  offsetClickable: string;
}

export const HexLocationsSection: Component<HexLocationsSectionProps> = (props) => {
  return (
    <Show when={props.selectedEntry}>
      {entry => (
        <div class="bg-bg-panel/80 border-2 border-accent/50">
          <div 
            class={`${props.categoryHeader} bg-accent/30`}
            onClick={() => props.toggleCategory("_hexLocations")}
          >
            <span class="text-[10px] leading-tight text-txt-muted w-3">
              {props.isExpanded("_hexLocations") ? "▾" : "▸"}
            </span>
            <span class="flex items-center gap-1 text-[10px] leading-tight font-medium text-txt-tertiary flex-1">
              <HiOutlineMapPin class="w-3 h-3" /> HEX LOCATIONS - {entry().isDir ? <HiOutlineFolder class="w-3 h-3 inline" /> : <HiOutlineDocument class="w-3 h-3 inline" />} {entry().name}
            </span>
          </div>
          
          <Show when={props.isExpanded("_hexLocations")}>
            <div class="py-1">
              {/* Entry Type */}
              <div class={`${props.rowBase} ${props.rowGrid}`}>
                <span class={props.keyStyle}>TYPE</span>
                <span class={props.valueStyle}>{entry().isDir ? "Folder" : "File"}</span>
                <span class={props.offsetStyle}></span>
              </div>
              
              {/* Entry Path */}
              <div class={`${props.rowBase} ${props.rowGrid}`}>
                <span class={props.keyStyle}>PATH</span>
                <span class={`${props.valueStyle} text-[10px] leading-tight break-all`} title={entry().entryPath}>{entry().entryPath}</span>
                <span class={props.offsetStyle}></span>
              </div>
              
              {/* Decompressed Size */}
              <div class={`${props.rowBase} ${props.rowGrid}`}>
                <span class={props.keyStyle}>SIZE</span>
                <span class={props.valueStyle}>{formatBytes(entry().size)}</span>
                <span class={props.offsetStyle}></span>
              </div>
              
              {/* Item Header Address - clickable to jump */}
              <Show when={entry().itemAddr != null}>
                <div 
                  class={`${props.rowBase} ${props.rowGrid} ${props.rowClickable} bg-green-900/20 border-l-2 border-green-500`}
                  onClick={() => props.handleRowClick(entry().itemAddr)}
                  title="Click to view item header in hex"
                >
                  <span class="flex items-center gap-0.5">{props.keyStyle && <HiOutlineMapPin class="w-2 h-2 text-green-400" />} ITEM HEADER</span>
                  <span class={`${props.valueStyle} text-accent font-bold`}>0x{entry().itemAddr!.toString(16).toUpperCase().padStart(8, '0')}</span>
                  <span class={`${props.offsetStyle} ${props.offsetClickable}`}>{formatOffsetLabel(entry().itemAddr!)}</span>
                </div>
              </Show>
              
              {/* Metadata Address */}
              <Show when={entry().metadataAddr != null}>
                <div 
                  class={`${props.rowBase} ${props.rowGrid} ${props.rowClickable}`}
                  onClick={() => props.handleRowClick(entry().metadataAddr)}
                  title="Click to view metadata in hex"
                >
                  <span class="flex items-center gap-0.5">{props.keyStyle && <HiOutlineClipboardDocument class="w-2 h-2 text-txt-secondary" />} METADATA</span>
                  <span class={`${props.valueStyle} text-accent font-bold`}>0x{entry().metadataAddr!.toString(16).toUpperCase().padStart(8, '0')}</span>
                  <span class={`${props.offsetStyle} ${props.offsetClickable}`}>{formatOffsetLabel(entry().metadataAddr!)}</span>
                </div>
              </Show>
              
              {/* Data Start Address (for files) */}
              <Show when={entry().dataAddr != null && !entry().isDir}>
                <div 
                  class={`${props.rowBase} ${props.rowGrid} ${props.rowClickable} bg-blue-900/20 border-l-2 border-blue-500`}
                  onClick={() => props.handleRowClick(entry().dataAddr)}
                  title="Click to view compressed data start in hex"
                >
                  <span class="flex items-center gap-0.5">{props.keyStyle && <HiOutlineCircleStack class="w-2 h-2 text-blue-400" />} DATA START</span>
                  <span class={`${props.valueStyle} text-accent font-bold`}>0x{entry().dataAddr!.toString(16).toUpperCase().padStart(8, '0')}</span>
                  <span class={`${props.offsetStyle} ${props.offsetClickable}`}>{formatOffsetLabel(entry().dataAddr!)}</span>
                </div>
              </Show>
              
              {/* Compressed Size (for files) */}
              <Show when={entry().compressedSize != null && !entry().isDir}>
                <div class={`${props.rowBase} ${props.rowGrid}`}>
                  <span class="flex items-center gap-0.5">{props.keyStyle && <HiOutlineArchiveBox class="w-2 h-2 text-txt-secondary" />} COMPRESSED</span>
                  <span class={props.valueStyle}>{formatBytes(entry().compressedSize!)}</span>
                  <span class={props.offsetStyle}></span>
                </div>
              </Show>
              
              {/* Data End Address (for files) */}
              <Show when={entry().dataEndAddr != null && !entry().isDir}>
                <div 
                  class={`${props.rowBase} ${props.rowGrid} ${props.rowClickable} bg-red-900/20 border-l-2 border-red-500`}
                  onClick={() => props.handleRowClick(entry().dataEndAddr)}
                  title="Click to view compressed data end in hex"
                >
                  <span class="flex items-center gap-0.5">{props.keyStyle && <HiOutlineArrowRightOnRectangle class="w-2 h-2 text-red-400" />} DATA END</span>
                  <span class={`${props.valueStyle} text-accent font-bold`}>0x{entry().dataEndAddr!.toString(16).toUpperCase().padStart(8, '0')}</span>
                  <span class={`${props.offsetStyle} ${props.offsetClickable}`}>{formatOffsetLabel(entry().dataEndAddr!)}</span>
                </div>
              </Show>
              
              {/* First Child Address (for folders) */}
              <Show when={entry().firstChildAddr != null && entry().isDir}>
                <div 
                  class={`${props.rowBase} ${props.rowGrid} ${props.rowClickable}`}
                  onClick={() => props.handleRowClick(entry().firstChildAddr)}
                  title="Click to view first child item in hex"
                >
                  <span class="flex items-center gap-0.5">{props.keyStyle && <HiOutlineUserGroup class="w-2 h-2 text-txt-secondary" />} FIRST CHILD</span>
                  <span class={`${props.valueStyle} text-accent font-bold`}>0x{entry().firstChildAddr!.toString(16).toUpperCase().padStart(8, '0')}</span>
                  <span class={`${props.offsetStyle} ${props.offsetClickable}`}>{formatOffsetLabel(entry().firstChildAddr!)}</span>
                </div>
              </Show>
              
              {/* Container Path */}
              <div class={`${props.rowBase} ${props.rowGrid} border-t border-border mt-1 pt-1.5`}>
                <span class={props.keyStyle}>CONTAINER</span>
                <span class={props.valueStyle} title={entry().containerPath}>
                  {getBasename(entry().containerPath) || entry().containerPath}
                </span>
                <span class={props.offsetStyle}></span>
              </div>
            </div>
          </Show>
        </div>
      )}
    </Show>
  );
};
