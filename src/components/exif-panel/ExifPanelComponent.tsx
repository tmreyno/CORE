// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show } from "solid-js";
import { HiOutlineExclamationTriangle } from "../icons";
import { useExifData } from "./useExifData";
import {
  CameraSection,
  CaptureSection,
  TimestampsSection,
  GpsSection,
  ImageSection,
  ForensicSection,
  RawTagsSection,
} from "./ExifSections";
import type { ExifPanelProps } from "./types";

export function ExifPanelComponent(props: ExifPanelProps) {
  const state = useExifData(props);

  return (
    <div class={`exif-panel flex flex-col h-full overflow-y-auto ${props.class || ""}`}>
      <Show
        when={!state.loading()}
        fallback={
          <div class="flex flex-col items-center justify-center h-full gap-2">
            <div class="animate-spin w-6 h-6 border-2 border-accent border-t-transparent rounded-full" />
            <span class="text-txt-muted text-xs">Reading EXIF...</span>
          </div>
        }
      >
        <Show
          when={!state.error()}
          fallback={
            <div class="flex flex-col items-center gap-2 p-4 text-center">
              <HiOutlineExclamationTriangle class="w-8 h-8 text-txt-muted" />
              <span class="text-xs text-txt-muted">No EXIF data available</span>
            </div>
          }
        >
          <Show when={state.exif()}>
            {(data) => (
              <div class="p-3 space-y-4">
                <CameraSection data={data()} />
                <CaptureSection data={data()} />
                <TimestampsSection data={data()} />
                <GpsSection data={data()} />
                <ImageSection data={data()} />
                <ForensicSection data={data()} show={!!state.hasForensicIndicators()} />
                <RawTagsSection
                  data={data()}
                  showRawTags={state.showRawTags()}
                  onToggle={() => state.setShowRawTags(!state.showRawTags())}
                  rawFilter={state.rawFilter()}
                  onFilterChange={(v) => state.setRawFilter(v)}
                  filteredTags={state.filteredRawTags()}
                />
              </div>
            )}
          </Show>
        </Show>
      </Show>
    </div>
  );
}
