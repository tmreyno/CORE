// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { GpsCoordinates } from "./types";

export function formatGpsCoord(gps: GpsCoordinates): string {
  return `${Math.abs(gps.latitude).toFixed(6)}°${gps.latitude_ref}, ${Math.abs(gps.longitude).toFixed(6)}°${gps.longitude_ref}`;
}

export function googleMapsUrl(gps: GpsCoordinates): string {
  return `https://www.google.com/maps?q=${gps.latitude},${gps.longitude}`;
}

export function orientationName(o: number | null): string | null {
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
