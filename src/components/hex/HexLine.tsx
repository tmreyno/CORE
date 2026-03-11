// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show, For } from "solid-js";
import type { HeaderRegion } from "../../types";
import { byteToHex, byteToAscii, formatOffset } from "../../utils";

const BYTES_PER_LINE = 16;

interface HexByteData {
  value: number;
  color: string | null;
  region: HeaderRegion | null;
}

interface HexLineData {
  offset: number;
  bytes: HexByteData[];
}

interface HexLineProps {
  line: HexLineData;
  showAddress: boolean;
  showAscii: boolean;
  hoveredOffset: number | null;
  selectedRegion: HeaderRegion | null;
  navigatedRange: { offset: number; size: number } | null;
  navigatedColor: string;
  onHoverByte: (offset: number | null) => void;
  onClearNavigation: () => void;
}

export const HexLine: Component<HexLineProps> = (props) => {
  const isInSelectedRegion = (byteOffset: number) => {
    const sel = props.selectedRegion;
    return !!(sel && byteOffset >= sel.start && byteOffset <= sel.end);
  };

  const isHovered = (byteOffset: number) => props.hoveredOffset === byteOffset;

  const isNavigated = (byteOffset: number) => {
    const nav = props.navigatedRange;
    return nav !== null && byteOffset >= nav.offset && byteOffset < nav.offset + nav.size;
  };

  const getBgColor = (byteData: HexByteData, byteOffset: number) => {
    if (isNavigated(byteOffset)) return props.navigatedColor;
    return byteData.color || undefined;
  };

  return (
    <div class="flex items-center gap-0 leading-tight hover:bg-bg-panel/30">
      <Show when={props.showAddress}>
        <span class="w-20 shrink-0 text-2xs text-accent/80">
          {formatOffset(props.line.offset)}
        </span>
      </Show>
      
      <span class="flex gap-0">
        <For each={props.line.bytes}>
          {(byteData, byteIdx) => {
            const byteOffset = props.line.offset + byteIdx();
            const bgColor = getBgColor(byteData, byteOffset);
            
            return (
              <span
                class="w-[22px] text-center text-2xs cursor-default"
                classList={{
                  'ring-1 ring-accent/50': isInSelectedRegion(byteOffset),
                  'ring-1 ring-white/30': isHovered(byteOffset) && !isInSelectedRegion(byteOffset),
                  'font-bold': isNavigated(byteOffset)
                }}
                style={bgColor ? { "background-color": bgColor } : {}}
                title={byteData.region ? `${byteData.region.name}: ${byteData.region.description}` : undefined}
                onMouseEnter={() => props.onHoverByte(byteOffset)}
                onMouseLeave={() => props.onHoverByte(null)}
                onClick={() => props.onClearNavigation()}
              >
                {byteToHex(byteData.value)}
              </span>
            );
          }}
        </For>
        {/* Padding for incomplete lines */}
        <For each={[...Array(Math.max(0, BYTES_PER_LINE - props.line.bytes.length)).keys()]}>
          {() => <span class="w-[22px] text-center text-2xs">  </span>}
        </For>
      </span>
      
      <Show when={props.showAscii}>
        <span class="ml-2 flex text-2xs text-txt-secondary">
          <For each={props.line.bytes}>
            {(byteData, byteIdx) => {
              const byteOffset = props.line.offset + byteIdx();
              const bgColor = getBgColor(byteData, byteOffset);
              
              return (
                <span
                  class="w-2 text-center cursor-default"
                  classList={{
                    'ring-1 ring-accent/50': isInSelectedRegion(byteOffset),
                    'ring-1 ring-white/30': isHovered(byteOffset) && !isInSelectedRegion(byteOffset),
                    'font-bold': isNavigated(byteOffset)
                  }}
                  style={bgColor ? { "background-color": bgColor } : {}}
                  onMouseEnter={() => props.onHoverByte(byteOffset)}
                  onMouseLeave={() => props.onHoverByte(null)}
                >
                  {byteToAscii(byteData.value)}
                </span>
              );
            }}
          </For>
        </span>
      </Show>
    </div>
  );
};
