// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, createEffect, createMemo } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { logger } from "../../utils/logger";
import type { ExifMetadataSection } from "../../types/viewerMetadata";
import type { ExifMetadata, ExifPanelProps } from "./types";

const log = logger.scope("ExifPanel");

export function useExifData(props: ExifPanelProps) {
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
    if (props.path) loadExif();
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
      gps: data.gps
        ? {
            latitude: data.gps.latitude,
            longitude: data.gps.longitude,
            altitude: data.gps.altitude || undefined,
            latitudeRef: data.gps.latitude_ref,
            longitudeRef: data.gps.longitude_ref,
          }
        : undefined,
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

  return {
    loading,
    error,
    exif,
    showRawTags,
    setShowRawTags,
    rawFilter,
    setRawFilter,
    filteredRawTags,
    hasForensicIndicators,
  };
}
