// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ExifPanel - EXIF metadata panel for forensic image analysis
 *
 * Displays EXIF metadata extracted from photos:
 * - Camera info (make, model, lens)
 * - Capture settings (exposure, aperture, ISO)
 * - Timestamps (forensically critical!)
 * - GPS coordinates with map link
 * - Forensic indicators (serial number, unique ID)
 * - All raw tags
 *
 * Designed to be displayed alongside the ImageViewer.
 */

import { createSignal, createEffect, Show, For, createMemo } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import {
  HiOutlineExclamationTriangle,
} from "./icons";
import { TimeIcon, LocationIcon, ChevronDownIcon, ChevronRightIcon, SearchIcon } from "./icons";
import { logger } from "../utils/logger";
import type { ExifMetadataSection } from "../types/viewerMetadata";
const log = logger.scope("ExifPanel");

// ============================================================================
// Types (matching Rust structs)
// ============================================================================

interface GpsCoordinates {
  latitude: number;
  longitude: number;
  altitude: number | null;
  latitude_ref: string;
  longitude_ref: string;
}

interface ExifMetadata {
  path: string;
  // Camera
  make: string | null;
  model: string | null;
  software: string | null;
  lens_model: string | null;
  // Capture
  exposure_time: string | null;
  f_number: string | null;
  iso: number | null;
  focal_length: string | null;
  flash: string | null;
  // Timestamps
  date_time_original: string | null;
  date_time_digitized: string | null;
  date_time: string | null;
  gps_timestamp: string | null;
  // GPS
  gps: GpsCoordinates | null;
  // Image
  width: number | null;
  height: number | null;
  orientation: number | null;
  color_space: string | null;
  // Forensic
  image_unique_id: string | null;
  owner_name: string | null;
  serial_number: string | null;
  // Raw
  raw_tags: [string, string][];
}

// ============================================================================
// Props
// ============================================================================

interface ExifPanelProps {
  /** Path to the image file */
  path: string;
  /** Optional class name */
  class?: string;
  /** Callback to emit metadata section for right panel */
  onMetadata?: (section: ExifMetadataSection) => void;
}

// ============================================================================
// Helpers
// ============================================================================

function formatGpsCoord(gps: GpsCoordinates): string {
  return `${Math.abs(gps.latitude).toFixed(6)}°${gps.latitude_ref}, ${Math.abs(gps.longitude).toFixed(6)}°${gps.longitude_ref}`;
}

function googleMapsUrl(gps: GpsCoordinates): string {
  return `https://www.google.com/maps?q=${gps.latitude},${gps.longitude}`;
}

function orientationName(o: number | null): string | null {
  if (o === null) return null;
  const names: Record<number, string> = {
    1: "Normal",
    2: "Flipped H",
    3: "Rotated 180°",
    4: "Flipped V",
    5: "Rotated 90° CW + Flipped",
    6: "Rotated 90° CW",
    7: "Rotated 90° CCW + Flipped",
    8: "Rotated 90° CCW",
  };
  return names[o] || `Unknown (${o})`;
}

// ============================================================================
// Component
// ============================================================================

export function ExifPanel(props: ExifPanelProps) {
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);
  const [exif, setExif] = createSignal<ExifMetadata | null>(null);
  const [showRawTags, setShowRawTags] = createSignal(false);
  const [rawFilter, setRawFilter] = createSignal("");

  const loadExif = async () => {
    setLoading(true);
    setError(null);

    try {
      const data = await invoke<ExifMetadata>("exif_extract", { path: props.path });
      setExif(data);
    } catch (e) {
      log.error("Failed to extract EXIF:", e);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  };

  createEffect(() => {
    if (props.path) {
      loadExif();
    }
  });

  const filteredRawTags = createMemo(() => {
    const data = exif();
    if (!data) return [];
    const query = rawFilter().toLowerCase();
    if (!query) return data.raw_tags;
    return data.raw_tags.filter(
      ([name, value]) =>
        name.toLowerCase().includes(query) || value.toLowerCase().includes(query)
    );
  });

  const hasForensicIndicators = createMemo(() => {
    const data = exif();
    return data && (data.image_unique_id || data.serial_number || data.owner_name);
  });

  // Emit metadata section when EXIF data loads
  createEffect(() => {
    const data = exif();
    if (!data || !props.onMetadata) return;
    const section: ExifMetadataSection = {
      kind: "exif",
      make: data.make || undefined,
      model: data.model || undefined,
      software: data.software || undefined,
      lensModel: data.lens_model || undefined,
      exposureTime: data.exposure_time || undefined,
      fNumber: data.f_number || undefined,
      iso: data.iso || undefined,
      focalLength: data.focal_length || undefined,
      flash: data.flash || undefined,
      dateTimeOriginal: data.date_time_original || undefined,
      dateTimeDigitized: data.date_time_digitized || undefined,
      dateTime: data.date_time || undefined,
      gpsTimestamp: data.gps_timestamp || undefined,
      gps: data.gps ? {
        latitude: data.gps.latitude,
        longitude: data.gps.longitude,
        altitude: data.gps.altitude || undefined,
        latitudeRef: data.gps.latitude_ref,
        longitudeRef: data.gps.longitude_ref,
      } : undefined,
      width: data.width || undefined,
      height: data.height || undefined,
      orientation: data.orientation || undefined,
      colorSpace: data.color_space || undefined,
      imageUniqueId: data.image_unique_id || undefined,
      ownerName: data.owner_name || undefined,
      serialNumber: data.serial_number || undefined,
      rawTagCount: data.raw_tags?.length,
    };
    props.onMetadata(section);
  });

  // Helper to render a metadata row
  const MetaRow = (p: { label: string; value: string | number | null | undefined; highlight?: boolean }) => (
    <Show when={p.value !== null && p.value !== undefined}>
      <div class="flex gap-2 py-0.5">
        <span class="text-txt-muted w-28 shrink-0 text-xs">{p.label}</span>
        <span class={`text-xs font-mono ${p.highlight ? "text-accent font-medium" : "text-txt"}`}>
          {String(p.value)}
        </span>
      </div>
    </Show>
  );

  return (
    <div class={`exif-panel flex flex-col h-full overflow-y-auto ${props.class || ""}`}>
      <Show
        when={!loading()}
        fallback={
          <div class="flex flex-col items-center justify-center h-full gap-2">
            <div class="animate-spin w-6 h-6 border-2 border-accent border-t-transparent rounded-full" />
            <span class="text-txt-muted text-xs">Reading EXIF...</span>
          </div>
        }
      >
        <Show
          when={!error()}
          fallback={
            <div class="flex flex-col items-center gap-2 p-4 text-center">
              <HiOutlineExclamationTriangle class="w-8 h-8 text-txt-muted" />
              <span class="text-xs text-txt-muted">No EXIF data available</span>
            </div>
          }
        >
          <Show when={exif()}>
            {(data) => (
              <div class="p-3 space-y-4">
                {/* Camera Section */}
                <Show when={data().make || data().model}>
                  <div>
                    <h3 class="text-xs font-semibold text-txt-secondary uppercase tracking-wider mb-1">Camera</h3>
                    <MetaRow label="Make" value={data().make} />
                    <MetaRow label="Model" value={data().model} />
                    <MetaRow label="Lens" value={data().lens_model} />
                    <MetaRow label="Software" value={data().software} />
                  </div>
                </Show>

                {/* Capture Settings */}
                <Show when={data().exposure_time || data().f_number || data().iso}>
                  <div>
                    <h3 class="text-xs font-semibold text-txt-secondary uppercase tracking-wider mb-1">Capture</h3>
                    <MetaRow label="Exposure" value={data().exposure_time} />
                    <MetaRow label="Aperture" value={data().f_number} />
                    <MetaRow label="ISO" value={data().iso} />
                    <MetaRow label="Focal Length" value={data().focal_length} />
                    <MetaRow label="Flash" value={data().flash} />
                  </div>
                </Show>

                {/* Timestamps (forensically critical) */}
                <Show when={data().date_time_original || data().date_time_digitized || data().date_time}>
                  <div>
                    <h3 class="text-xs font-semibold text-txt-secondary uppercase tracking-wider mb-1 flex items-center gap-1">
                      <TimeIcon class="w-3 h-3" /> Timestamps
                    </h3>
                    <MetaRow label="Original" value={data().date_time_original} highlight={true} />
                    <MetaRow label="Digitized" value={data().date_time_digitized} />
                    <MetaRow label="Modified" value={data().date_time} />
                    <MetaRow label="GPS Time" value={data().gps_timestamp} />
                  </div>
                </Show>

                {/* GPS */}
                <Show when={data().gps}>
                  <div>
                    <h3 class="text-xs font-semibold text-txt-secondary uppercase tracking-wider mb-1 flex items-center gap-1">
                      <LocationIcon class="w-3 h-3" /> GPS Location
                    </h3>
                    <div class="bg-bg-secondary rounded p-2 space-y-1">
                      <div class="text-xs font-mono text-accent">
                        {formatGpsCoord(data().gps!)}
                      </div>
                      <Show when={data().gps!.altitude !== null}>
                        <div class="text-xs text-txt-muted">
                          Altitude: {data().gps!.altitude!.toFixed(1)}m
                        </div>
                      </Show>
                      <a
                        href={googleMapsUrl(data().gps!)}
                        target="_blank"
                        rel="noopener noreferrer"
                        class="text-xs text-accent hover:text-accent-hover underline"
                      >
                        View on Google Maps →
                      </a>
                    </div>
                  </div>
                </Show>

                {/* Image Info */}
                <Show when={data().width || data().height}>
                  <div>
                    <h3 class="text-xs font-semibold text-txt-secondary uppercase tracking-wider mb-1">Image</h3>
                    <Show when={data().width && data().height}>
                      <MetaRow label="Dimensions" value={`${data().width} × ${data().height}`} />
                    </Show>
                    <MetaRow label="Orientation" value={orientationName(data().orientation)} />
                    <MetaRow label="Color Space" value={data().color_space} />
                  </div>
                </Show>

                {/* Forensic Indicators */}
                <Show when={hasForensicIndicators()}>
                  <div>
                    <h3 class="text-xs font-semibold text-warning uppercase tracking-wider mb-1">⚠ Forensic Indicators</h3>
                    <div class="bg-warning/10 border border-warning/30 rounded p-2">
                      <MetaRow label="Unique ID" value={data().image_unique_id} highlight={true} />
                      <MetaRow label="Serial No." value={data().serial_number} highlight={true} />
                      <MetaRow label="Owner" value={data().owner_name} highlight={true} />
                    </div>
                  </div>
                </Show>

                {/* Raw Tags */}
                <Show when={data().raw_tags.length > 0}>
                  <div>
                    <button
                      class="flex items-center gap-1 text-xs font-semibold text-txt-secondary uppercase tracking-wider hover:text-txt w-full text-left"
                      onClick={() => setShowRawTags(!showRawTags())}
                    >
                      <Show when={showRawTags()} fallback={<ChevronRightIcon class="w-3 h-3" />}>
                        <ChevronDownIcon class="w-3 h-3" />
                      </Show>
                      All Tags ({data().raw_tags.length})
                    </button>
                    <Show when={showRawTags()}>
                      <div class="mt-1 space-y-0.5">
                        {/* Filter */}
                        <div class="relative mb-2">
                          <SearchIcon class="w-3 h-3 absolute left-2 top-1/2 -translate-y-1/2 text-txt-muted" />
                          <input
                            type="text"
                            class="input-xs pl-6 w-full"
                            placeholder="Filter tags..."
                            value={rawFilter()}
                            onInput={(e) => setRawFilter(e.currentTarget.value)}
                          />
                        </div>
                        <div class="max-h-48 overflow-y-auto space-y-0.5">
                          <For each={filteredRawTags()}>
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
              </div>
            )}
          </Show>
        </Show>
      </Show>
    </div>
  );
}
