// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show, For } from "solid-js";
import { TimeIcon, LocationIcon, ChevronDownIcon, ChevronRightIcon, SearchIcon } from "../icons";
import type { ExifMetadata } from "./types";
import { formatGpsCoord, googleMapsUrl, orientationName } from "./exifHelpers";

// Reusable metadata row
export function MetaRow(p: { label: string; value: string | number | null | undefined; highlight?: boolean }) {
  return (
    <Show when={p.value !== null && p.value !== undefined}>
      <div class="flex gap-2 py-0.5">
        <span class="text-txt-muted w-28 shrink-0 text-xs">{p.label}</span>
        <span class={`text-xs font-mono ${p.highlight ? "text-accent font-medium" : "text-txt"}`}>
          {String(p.value)}
        </span>
      </div>
    </Show>
  );
}

export function CameraSection(props: { data: ExifMetadata }) {
  return (
    <Show when={props.data.make || props.data.model}>
      <div>
        <h3 class="text-xs font-semibold text-txt-secondary uppercase tracking-wider mb-1">Camera</h3>
        <MetaRow label="Make" value={props.data.make} />
        <MetaRow label="Model" value={props.data.model} />
        <MetaRow label="Lens" value={props.data.lens_model} />
        <MetaRow label="Software" value={props.data.software} />
      </div>
    </Show>
  );
}

export function CaptureSection(props: { data: ExifMetadata }) {
  return (
    <Show when={props.data.exposure_time || props.data.f_number || props.data.iso}>
      <div>
        <h3 class="text-xs font-semibold text-txt-secondary uppercase tracking-wider mb-1">Capture</h3>
        <MetaRow label="Exposure" value={props.data.exposure_time} />
        <MetaRow label="Aperture" value={props.data.f_number} />
        <MetaRow label="ISO" value={props.data.iso} />
        <MetaRow label="Focal Length" value={props.data.focal_length} />
        <MetaRow label="Flash" value={props.data.flash} />
      </div>
    </Show>
  );
}

export function TimestampsSection(props: { data: ExifMetadata }) {
  return (
    <Show when={props.data.date_time_original || props.data.date_time_digitized || props.data.date_time}>
      <div>
        <h3 class="text-xs font-semibold text-txt-secondary uppercase tracking-wider mb-1 flex items-center gap-1">
          <TimeIcon class="w-3 h-3" /> Timestamps
        </h3>
        <MetaRow label="Original" value={props.data.date_time_original} highlight={true} />
        <MetaRow label="Digitized" value={props.data.date_time_digitized} />
        <MetaRow label="Modified" value={props.data.date_time} />
        <MetaRow label="GPS Time" value={props.data.gps_timestamp} />
      </div>
    </Show>
  );
}

export function GpsSection(props: { data: ExifMetadata }) {
  return (
    <Show when={props.data.gps}>
      <div>
        <h3 class="text-xs font-semibold text-txt-secondary uppercase tracking-wider mb-1 flex items-center gap-1">
          <LocationIcon class="w-3 h-3" /> GPS Location
        </h3>
        <div class="bg-bg-secondary rounded p-2 space-y-1">
          <div class="text-xs font-mono text-accent">{formatGpsCoord(props.data.gps!)}</div>
          <Show when={props.data.gps!.altitude !== null}>
            <div class="text-xs text-txt-muted">Altitude: {props.data.gps!.altitude!.toFixed(1)}m</div>
          </Show>
          <a
            href={googleMapsUrl(props.data.gps!)}
            target="_blank"
            rel="noopener noreferrer"
            class="text-xs text-accent hover:text-accent-hover underline"
          >
            View on Google Maps →
          </a>
        </div>
      </div>
    </Show>
  );
}

export function ImageSection(props: { data: ExifMetadata }) {
  return (
    <Show when={props.data.width || props.data.height}>
      <div>
        <h3 class="text-xs font-semibold text-txt-secondary uppercase tracking-wider mb-1">Image</h3>
        <Show when={props.data.width && props.data.height}>
          <MetaRow label="Dimensions" value={`${props.data.width} × ${props.data.height}`} />
        </Show>
        <MetaRow label="Orientation" value={orientationName(props.data.orientation)} />
        <MetaRow label="Color Space" value={props.data.color_space} />
      </div>
    </Show>
  );
}

export function ForensicSection(props: { data: ExifMetadata; show: boolean }) {
  return (
    <Show when={props.show}>
      <div>
        <h3 class="text-xs font-semibold text-warning uppercase tracking-wider mb-1">⚠ Forensic Indicators</h3>
        <div class="bg-warning/10 border border-warning/30 rounded p-2">
          <MetaRow label="Unique ID" value={props.data.image_unique_id} highlight={true} />
          <MetaRow label="Serial No." value={props.data.serial_number} highlight={true} />
          <MetaRow label="Owner" value={props.data.owner_name} highlight={true} />
        </div>
      </div>
    </Show>
  );
}

interface RawTagsSectionProps {
  data: ExifMetadata;
  showRawTags: boolean;
  onToggle: () => void;
  rawFilter: string;
  onFilterChange: (v: string) => void;
  filteredTags: [string, string][];
}

export function RawTagsSection(props: RawTagsSectionProps) {
  return (
    <Show when={props.data.raw_tags.length > 0}>
      <div>
        <button
          class="flex items-center gap-1 text-xs font-semibold text-txt-secondary uppercase tracking-wider hover:text-txt w-full text-left"
          onClick={props.onToggle}
        >
          <Show when={props.showRawTags} fallback={<ChevronRightIcon class="w-3 h-3" />}>
            <ChevronDownIcon class="w-3 h-3" />
          </Show>
          All Tags ({props.data.raw_tags.length})
        </button>
        <Show when={props.showRawTags}>
          <div class="mt-1 space-y-0.5">
            <div class="relative mb-2">
              <SearchIcon class="w-3 h-3 absolute left-2 top-1/2 -translate-y-1/2 text-txt-muted" />
              <input
                type="text"
                class="input-xs pl-6 w-full"
                placeholder="Filter tags..."
                value={props.rawFilter}
                onInput={(e) => props.onFilterChange(e.currentTarget.value)}
              />
            </div>
            <div class="max-h-48 overflow-y-auto space-y-0.5">
              <For each={props.filteredTags}>
                {([name, value]) => (
                  <div class="flex gap-1 text-[11px]">
                    <span class="text-accent shrink-0 w-32 truncate" title={name}>{name}</span>
                    <span class="text-txt-muted truncate" title={value}>{value}</span>
                  </div>
                )}
              </For>
            </div>
          </div>
        </Show>
      </div>
    </Show>
  );
}
