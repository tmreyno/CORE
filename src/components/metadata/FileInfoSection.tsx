// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show } from "solid-js";
import { formatBytes } from "../../utils";
import type { FileInfo } from "./types";

interface FileInfoSectionProps {
  fileInfo?: FileInfo | null;
  rowBase: string;
  rowGrid: string;
  keyStyle: string;
  valueStyle: string;
  offsetStyle: string;
}

export const FileInfoSection: Component<FileInfoSectionProps> = (props) => {
  return (
    <Show when={props.fileInfo}>
      {info => (
        <div class="border-b border-border">
          <div class={`${props.rowBase} ${props.rowGrid}`}>
            <span class={props.keyStyle}>File</span>
            <span class={props.valueStyle} title={info().path}>{info().filename}</span>
            <span class={props.offsetStyle}></span>
          </div>
          <div class={`${props.rowBase} ${props.rowGrid}`}>
            <span class={props.keyStyle}>Size</span>
            <span class={props.valueStyle}>{formatBytes(info().size)}</span>
            <span class={props.offsetStyle}></span>
          </div>
          <Show when={info().segment_count && info().segment_count! > 1}>
            <div class={`${props.rowBase} ${props.rowGrid}`}>
              <span class={props.keyStyle}>Segments</span>
              <span class={props.valueStyle}>{info().segment_count}</span>
              <span class={props.offsetStyle}></span>
            </div>
          </Show>
        </div>
      )}
    </Show>
  );
};

export type { FileInfo };
