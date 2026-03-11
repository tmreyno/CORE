// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show } from "solid-js";
import {
  HiOutlineCheckCircle,
  HiOutlineLockClosed,
  HiOutlineDocumentText,
} from "../icons";
import { formatBytes } from "../../utils";
import type { FileTooltipProps } from "./types";

export function FileTooltip(props: FileTooltipProps) {
  return (
    <div class="absolute left-full top-0 ml-2 z-50 min-w-[280px] max-w-[360px] p-3 bg-bg-panel border border-border rounded-lg shadow-xl text-sm">
      <div class="font-semibold text-accent mb-1">{props.file.container_type}</div>
      <div class="text-xs text-txt-secondary break-all mb-2">{props.file.path}</div>
      <div class="flex justify-between text-xs"><span class="text-txt-muted">Size:</span><span class="text-txt-tertiary">{formatBytes(props.file.size)}</span></div>
      
      <Show when={props.file.segment_count}>
        <div class="flex justify-between text-xs"><span class="text-txt-muted">Segments:</span><span class="text-txt-tertiary">{props.file.segment_count}</span></div>
      </Show>
      
      <Show when={props.fileInfo}>
        <div class="my-2 border-t border-border" />
        
        <Show when={props.fileInfo!.ad1}>
          <div class="flex justify-between text-xs"><span class="text-txt-muted">Items:</span><span class="text-txt-tertiary">{props.fileInfo!.ad1!.item_count}</span></div>
          <Show when={props.fileInfo!.ad1!.companion_log?.case_number}>
            <div class="flex justify-between text-xs"><span class="text-txt-muted">Case:</span><span class="text-txt-tertiary">{props.fileInfo!.ad1!.companion_log!.case_number}</span></div>
          </Show>
          <Show when={props.fileInfo!.ad1!.companion_log?.evidence_number}>
            <div class="flex justify-between text-xs"><span class="text-txt-muted">Evidence:</span><span class="text-txt-tertiary">{props.fileInfo!.ad1!.companion_log!.evidence_number}</span></div>
          </Show>
          <Show when={props.fileInfo!.ad1!.volume?.filesystem}>
            <div class="flex justify-between text-xs"><span class="text-txt-muted">FS:</span><span class="text-txt-tertiary">{props.fileInfo!.ad1!.volume!.filesystem}</span></div>
          </Show>
          <div class="flex justify-between text-xs"><span class="text-txt-muted">Source:</span><span class="text-txt-tertiary truncate ml-2">{props.fileInfo!.ad1!.logical.data_source_name}</span></div>
        </Show>
        
        <Show when={props.fileInfo!.e01}>
          <div class="flex justify-between text-xs"><span class="text-txt-muted">Format:</span><span class="text-txt-tertiary">{props.fileInfo!.e01!.format_version}</span></div>
          <div class="flex justify-between text-xs"><span class="text-txt-muted">Compression:</span><span class="text-txt-tertiary">{props.fileInfo!.e01!.compression}</span></div>
          <Show when={props.fileInfo!.e01!.case_number}>
            <div class="flex justify-between text-xs"><span class="text-txt-muted">Case:</span><span class="text-txt-tertiary">{props.fileInfo!.e01!.case_number}</span></div>
          </Show>
        </Show>
        
        <Show when={props.fileInfo!.raw}>
          <div class="flex justify-between text-xs"><span class="text-txt-muted">Segments:</span><span class="text-txt-tertiary">{props.fileInfo!.raw!.segment_count}</span></div>
        </Show>
        
        <Show when={(props.fileInfo?.e01?.stored_hashes?.length ?? 0) > 0 || (props.fileInfo?.companion_log?.stored_hashes?.length ?? 0) > 0}>
          <div class="my-2 border-t border-border" />
          <div class="flex items-center gap-1 text-xs font-semibold text-txt-secondary mb-1">
            <HiOutlineDocumentText class="w-3 h-3" /> Stored Hashes
          </div>
          
          <Show when={(props.fileInfo?.e01?.stored_hashes?.length ?? 0) > 0}>
            {props.fileInfo!.e01!.stored_hashes!.map((sh) => (
              <div class="flex items-center gap-2 text-xs py-0.5">
                <span class="text-accent font-mono">{sh.algorithm}</span>
                <code class="text-txt-secondary font-mono truncate">{sh.hash.substring(0, 16)}...</code>
                <Show when={sh.verified === true}><HiOutlineCheckCircle class="w-3 h-3 text-green-400" /></Show>
                <Show when={sh.timestamp}><span class="text-txt-muted text-2xs leading-tight">{sh.timestamp}</span></Show>
              </div>
            ))}
          </Show>
          
          <Show when={(props.fileInfo?.companion_log?.stored_hashes?.length ?? 0) > 0}>
            {props.fileInfo!.companion_log!.stored_hashes.map((sh) => (
              <div class="flex items-center gap-2 text-xs py-0.5">
                <span class="text-accent font-mono">{sh.algorithm}</span>
                <code class="text-txt-secondary font-mono truncate">{sh.hash.substring(0, 16)}...</code>
                <Show when={sh.verified === true}><HiOutlineCheckCircle class="w-3 h-3 text-green-400" /></Show>
                <Show when={sh.timestamp}><span class="text-txt-muted text-2xs leading-tight">{sh.timestamp}</span></Show>
              </div>
            ))}
          </Show>
        </Show>
      </Show>
      
      <Show when={props.fileHash}>
        <div class="my-2 border-t border-border" />
        <div class="flex flex-col gap-1">
          <span class="flex items-center gap-1 text-xs text-accent"><HiOutlineLockClosed class="w-3 h-3" /> {props.fileHash!.algorithm}</span>
          <code class="text-xs text-txt-tertiary font-mono break-all">{props.fileHash!.hash}</code>
        </div>
      </Show>
    </div>
  );
}
