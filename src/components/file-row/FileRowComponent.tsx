// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show } from "solid-js";
import { formatBytes, typeClass } from "../../utils";
import { getContainerTypeIcon } from "../tree";
import { HiOutlineFolderOpen } from "../icons";
import { HashIndicators } from "./HashIndicators";
import { FileTooltip } from "./FileTooltip";
import { getTotalContainerSize, buildSizeLabel } from "./hashHelpers";
import type { FileRowProps } from "./types";

export function FileRowComponent(props: FileRowProps) {
  const totalSize = () => getTotalContainerSize(props.fileInfo);
  const displaySize = () => totalSize() ?? props.file.size;
  const sizeLabel = () => buildSizeLabel(props.file.size, totalSize(), props.file.segment_count);

  const rowClass = () => {
    let base = "flex items-center gap-2 px-3 py-2 cursor-pointer transition-colors border-b border-border/50";
    if (props.isSelected) base += " bg-accent-soft";
    if (props.isActive) base += " bg-accent/20 outline outline-1 outline-accent";
    if (props.isFocused && !props.isActive) base += " outline-2 outline-dashed outline-accent outline-offset-[-2px]";
    if (!props.isSelected && !props.isActive) base += " hover:bg-bg-panel/50";
    return base;
  };

  return (
    <div
      id={`file-row-${props.index}`}
      class={rowClass()}
      onMouseEnter={props.onMouseEnter}
      onMouseLeave={props.onMouseLeave}
      onClick={props.onSelect}
      onContextMenu={(e) => props.onContextMenu?.(e)}
      data-index={props.index}
      role="option"
      aria-selected={props.isSelected}
      tabIndex={props.isFocused ? 0 : -1}
    >
      <input
        type="checkbox"
        class="shrink-0 accent-accent"
        checked={props.isSelected}
        onChange={(e) => { e.stopPropagation(); props.onToggleSelection(); }}
        onClick={(e) => e.stopPropagation()}
      />

      <span class={`shrink-0 ${typeClass(props.file.container_type)}`} title={props.file.container_type}>
        {(() => {
          const IconComponent = getContainerTypeIcon(props.file.container_type);
          return <IconComponent class="w-4 h-4" />;
        })()}
      </span>

      <div class="flex-1 min-w-0 flex flex-col">
        <span class="text-sm text-txt truncate" title={props.file.path}>{props.file.filename}</span>
        <span class="text-xs text-txt-muted flex items-center gap-1">
          <span title={sizeLabel()}>{formatBytes(displaySize())}</span>
          <Show when={props.file.segment_count && props.file.segment_count > 1}>
            <span class="text-txt-muted">• {props.file.segment_count} segs</span>
          </Show>
        </span>
      </div>

      <div class="flex items-center gap-1.5 shrink-0">
        <Show when={props.fileInfo?.ad1?.item_count}>
          <span
            class="flex items-center gap-0.5 px-1.5 py-0.5 text-xs bg-bg-hover rounded text-txt-tertiary"
            title={`${props.fileInfo!.ad1!.item_count.toLocaleString()} items`}
          >
            <HiOutlineFolderOpen class="w-3 h-3" />
            {props.fileInfo!.ad1!.item_count > 999
              ? Math.round(props.fileInfo!.ad1!.item_count / 1000) + "k"
              : props.fileInfo!.ad1!.item_count}
          </span>
        </Show>

        <HashIndicators
          fileStatus={props.fileStatus}
          fileInfo={props.fileInfo}
          fileHash={props.fileHash}
          hashHistory={props.hashHistory}
          busy={props.busy}
          onHash={props.onHash}
        />
      </div>

      <Show when={props.isHovered && !props.isActive}>
        <FileTooltip
          file={props.file}
          fileInfo={props.fileInfo}
          fileHash={props.fileHash}
        />
      </Show>
    </div>
  );
}
