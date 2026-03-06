// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { ExifMetadataSection } from "../../types/viewerMetadata";

export interface GpsCoordinates {
  latitude: number;
  longitude: number;
  altitude: number | null;
  latitude_ref: string;
  longitude_ref: string;
}

export interface ExifMetadata {
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

export interface ExifPanelProps {
  /** Path to the image file */
  path: string;
  /** Optional class name */
  class?: string;
  /** Callback to emit metadata section for right panel */
  onMetadata?: (section: ExifMetadataSection) => void;
}
