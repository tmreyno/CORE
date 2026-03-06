// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show } from "solid-js";
import {
  HiOutlineMagnifyingGlass,
  HiOutlineArrowPath,
} from "../../icons";
import { formatBytes } from "../../../utils";
import type { ContainerDetailsSectionProps } from "./types";

type EnhancedEwfPanelProps = Pick<
  ContainerDetailsSectionProps,
  | "enhancedEwfInfo"
  | "enhancedEwfLoading"
  | "enhancedEwfError"
  | "fetchEnhancedEwfInfo"
  | "rowBase"
  | "rowGrid"
  | "keyStyle"
  | "valueStyle"
  | "offsetStyle"
>;

export function EnhancedEwfPanel(props: EnhancedEwfPanelProps) {
  return (
    <>
      {/* Load button / loading / error */}
      <div class={`${props.rowBase} grid-cols-1`}>
        <Show when={!props.enhancedEwfInfo() && !props.enhancedEwfLoading()}>
          <button
            class="flex items-center gap-1 text-[10px] text-accent hover:text-accent-hover transition-colors"
            onClick={props.fetchEnhancedEwfInfo}
            title="Load additional metadata via libewf (format details, encryption, media type)"
          >
            <HiOutlineMagnifyingGlass class="w-3 h-3" />
            Load Enhanced Info
          </button>
        </Show>
        <Show when={props.enhancedEwfLoading()}>
          <span class="flex items-center gap-1 text-[10px] text-txt-muted">
            <HiOutlineArrowPath class="w-3 h-3 animate-spin" /> Loading libewf info...
          </span>
        </Show>
        <Show when={props.enhancedEwfError()}>
          <span class="text-[10px] text-error">{props.enhancedEwfError()}</span>
        </Show>
      </div>

      {/* Display enhanced info */}
      <Show when={props.enhancedEwfInfo()}>
        {(info) => {
          const Row = (p: { label: string; value: string | number | undefined | null; special?: string }) => (
            <Show when={p.value != null}>
              <div class={`${props.rowBase} ${props.rowGrid}`}>
                <span class={props.keyStyle}>{p.label}</span>
                <span class={p.special || props.valueStyle}>{typeof p.value === "number" ? String(p.value) : p.value}</span>
                <span class={props.offsetStyle}></span>
              </div>
            </Show>
          );

          return (
            <>
              <div class={`${props.rowBase} ${props.rowGrid}`}>
                <span class={props.keyStyle}>FORMAT (libewf)</span>
                <span class={props.valueStyle}>
                  {info().format}
                  {info().isV2 ? " (v2)" : ""}
                </span>
                <span class={props.offsetStyle}>.{info().formatExtension}</span>
              </div>
              <Show when={info().isLogical !== undefined}>
                <div class={`${props.rowBase} ${props.rowGrid}`}>
                  <span class={props.keyStyle}>IMAGE TYPE</span>
                  <span class={props.valueStyle}>
                    {info().isLogical ? "Logical" : "Physical"}
                  </span>
                  <span class={props.offsetStyle}></span>
                </div>
              </Show>
              <div class={`${props.rowBase} ${props.rowGrid}`}>
                <span class={props.keyStyle}>MEDIA SIZE</span>
                <span class={props.valueStyle}>{formatBytes(info().mediaSize)}</span>
                <span class={props.offsetStyle}></span>
              </div>
              <Row label="MEDIA TYPE" value={info().mediaType} />
              <Row label="COMPRESS METHOD" value={info().compressionMethod} />
              <Row label="COMPRESS LEVEL" value={info().compressionLevel} />
              <Show when={info().isEncrypted}>
                <div class={`${props.rowBase} ${props.rowGrid}`}>
                  <span class={props.keyStyle}>ENCRYPTED</span>
                  <span class="font-mono text-[10px] leading-tight text-warning">Yes</span>
                  <span class={props.offsetStyle}></span>
                </div>
              </Show>
              <Show when={info().isCorrupted}>
                <div class={`${props.rowBase} ${props.rowGrid}`}>
                  <span class={props.keyStyle}>CORRUPTED</span>
                  <span class="font-mono text-[10px] leading-tight text-error">Yes</span>
                  <span class={props.offsetStyle}></span>
                </div>
              </Show>
              <Show when={info().md5Hash}>
                <div class={`${props.rowBase} ${props.rowGrid}`}>
                  <span class={props.keyStyle}>MD5 (libewf)</span>
                  <span class={`${props.valueStyle} text-[10px] select-all`}>{info().md5Hash}</span>
                  <span class={props.offsetStyle}></span>
                </div>
              </Show>
              <Show when={info().sha1Hash}>
                <div class={`${props.rowBase} ${props.rowGrid}`}>
                  <span class={props.keyStyle}>SHA1 (libewf)</span>
                  <span class={`${props.valueStyle} text-[10px] select-all`}>{info().sha1Hash}</span>
                  <span class={props.offsetStyle}></span>
                </div>
              </Show>
              <Show when={info().caseInfo}>
                {(caseInfo) => (
                  <>
                    <Row label="CASE # (libewf)" value={caseInfo().caseNumber} />
                    <Row label="EXAMINER (libewf)" value={caseInfo().examinerName} />
                    <Row label="ACQUIRED (libewf)" value={caseInfo().acquiryDate} />
                  </>
                )}
              </Show>
            </>
          );
        }}
      </Show>
    </>
  );
}
