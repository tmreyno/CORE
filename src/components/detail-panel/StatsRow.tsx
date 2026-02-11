// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show, Component, Accessor } from "solid-js";
import type { DiscoveredFile, ContainerInfo } from "../../types";
import { formatBytes } from "../../utils";

interface StatsRowProps {
  file: DiscoveredFile;
  ad1Info: Accessor<ContainerInfo["ad1"] | undefined>;
  e01Info: Accessor<ContainerInfo["e01"] | undefined>;
  ufedInfo: Accessor<ContainerInfo["ufed"] | undefined>;
  hasAcquiryDate: Accessor<boolean>;
}

export const StatsRow: Component<StatsRowProps> = (props) => {
  return (
    <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-2 p-3 bg-bg-panel/50 rounded-lg border border-border/50">
      <div class="flex flex-col gap-0.5">
        <span class={`text-[10px] leading-tight text-txt-muted uppercase tracking-wider`}>Size</span>
        <span class="text-sm text-txt font-medium" title={`${props.file.size.toLocaleString()} bytes`}>
          {formatBytes(props.file.size)}
        </span>
      </div>
      
      <Show when={props.file.segment_count}>
        <div class="flex flex-col gap-0.5">
          <span class={`text-[10px] leading-tight text-txt-muted uppercase tracking-wider`}>Segments</span>
          <span class="text-sm text-txt font-medium" title={`${props.file.segment_count} segments`}>
            {props.file.segment_count}
          </span>
        </div>
      </Show>
      
      {/* E01: Show acquisition date from header */}
      <Show when={props.e01Info()?.acquiry_date}>
        <div class="flex flex-col gap-0.5">
          <span class={`text-[10px] leading-tight text-txt-muted uppercase tracking-wider`}>Acquired</span>
          <span class="text-sm text-txt font-medium" title={`Acquisition date from E01 header: ${props.e01Info()!.acquiry_date}`}>
            {props.e01Info()!.acquiry_date}
          </span>
        </div>
      </Show>
      
      {/* AD1: Show acquisition date from companion log */}
      <Show when={props.ad1Info()?.companion_log?.acquisition_date}>
        <div class="flex flex-col gap-0.5">
          <span class={`text-[10px] leading-tight text-txt-muted uppercase tracking-wider`}>Acquired</span>
          <span class="text-sm text-txt font-medium" title={`Acquisition date from AD1 companion log: ${props.ad1Info()!.companion_log!.acquisition_date}`}>
            {props.ad1Info()!.companion_log!.acquisition_date}
          </span>
        </div>
      </Show>
      
      {/* UFED: Show extraction date */}
      <Show when={props.ufedInfo()?.extraction_info?.start_time}>
        <div class="flex flex-col gap-0.5">
          <span class={`text-[10px] leading-tight text-txt-muted uppercase tracking-wider`}>Extracted</span>
          <span class="text-sm text-txt font-medium" title={`Extraction date from UFED metadata: ${props.ufedInfo()!.extraction_info!.start_time}`}>
            {props.ufedInfo()!.extraction_info!.start_time}
          </span>
        </div>
      </Show>
      
      {/* Fallback to filesystem dates only if no container date */}
      <Show when={!props.hasAcquiryDate()}>
        <Show when={props.file.created}>
          <div class="flex flex-col gap-0.5">
            <span class={`text-[10px] leading-tight text-txt-muted uppercase tracking-wider`}>File Created</span>
            <span class="text-sm text-txt font-medium" title={`Filesystem date (when file was created on disk): ${props.file.created}`}>
              {props.file.created}
            </span>
          </div>
        </Show>
        <Show when={props.file.modified}>
          <div class="flex flex-col gap-0.5">
            <span class={`text-[10px] leading-tight text-txt-muted uppercase tracking-wider`}>File Modified</span>
            <span class="text-sm text-txt font-medium" title={`Filesystem date (when file was last modified): ${props.file.modified}`}>
              {props.file.modified}
            </span>
          </div>
        </Show>
      </Show>
      
      <Show when={props.ad1Info()}>
        <div class="flex flex-col gap-0.5">
          <span class={`text-[10px] leading-tight text-txt-muted uppercase tracking-wider`}>Items</span>
          <span class="text-sm text-txt font-medium" title={`${props.ad1Info()!.item_count.toLocaleString()} items in AD1 container`}>
            {props.ad1Info()!.item_count.toLocaleString()}
          </span>
        </div>
      </Show>
      
      <Show when={props.e01Info()}>
        <div class="flex flex-col gap-0.5">
          <span class={`text-[10px] leading-tight text-txt-muted uppercase tracking-wider`}>Chunks</span>
          <span class="text-sm text-txt font-medium" title={`${props.e01Info()!.chunk_count.toLocaleString()} compressed chunks`}>
            {props.e01Info()!.chunk_count.toLocaleString()}
          </span>
        </div>
        <div class="flex flex-col gap-0.5">
          <span class={`text-[10px] leading-tight text-txt-muted uppercase tracking-wider`}>Sectors</span>
          <span class="text-sm text-txt font-medium" title={`${props.e01Info()!.sector_count.toLocaleString()} sectors`}>
            {props.e01Info()!.sector_count.toLocaleString()}
          </span>
        </div>
      </Show>
    </div>
  );
};
