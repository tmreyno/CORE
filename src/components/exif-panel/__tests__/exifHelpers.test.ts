// =============================================================================
// exifHelpers — GPS formatting and orientation name tests
// =============================================================================

import { describe, it, expect } from "vitest";
import { formatGpsCoord, googleMapsUrl, orientationName } from "../exifHelpers";
import type { GpsCoordinates } from "../types";

const GPS_NYC: GpsCoordinates = {
  latitude: 40.7128,
  longitude: -74.006,
  altitude: 10,
  latitude_ref: "N",
  longitude_ref: "W",
};

const GPS_SYDNEY: GpsCoordinates = {
  latitude: -33.8688,
  longitude: 151.2093,
  altitude: null,
  latitude_ref: "S",
  longitude_ref: "E",
};

describe("formatGpsCoord", () => {
  it("formats positive lat/long with refs", () => {
    expect(formatGpsCoord(GPS_NYC)).toBe("40.712800°N, 74.006000°W");
  });

  it("uses absolute value for negative coordinates", () => {
    expect(formatGpsCoord(GPS_SYDNEY)).toBe("33.868800°S, 151.209300°E");
  });

  it("formats zero coordinates", () => {
    const gps: GpsCoordinates = { latitude: 0, longitude: 0, altitude: null, latitude_ref: "N", longitude_ref: "E" };
    expect(formatGpsCoord(gps)).toBe("0.000000°N, 0.000000°E");
  });

  it("preserves 6 decimal places", () => {
    const gps: GpsCoordinates = { latitude: 1.1, longitude: 2.2, altitude: null, latitude_ref: "N", longitude_ref: "E" };
    expect(formatGpsCoord(gps)).toBe("1.100000°N, 2.200000°E");
  });
});

describe("googleMapsUrl", () => {
  it("builds a valid Google Maps URL", () => {
    expect(googleMapsUrl(GPS_NYC)).toBe("https://www.google.com/maps?q=40.7128,-74.006");
  });

  it("includes negative coordinates directly", () => {
    expect(googleMapsUrl(GPS_SYDNEY)).toBe("https://www.google.com/maps?q=-33.8688,151.2093");
  });

  it("handles zero coordinates", () => {
    const gps: GpsCoordinates = { latitude: 0, longitude: 0, altitude: null, latitude_ref: "N", longitude_ref: "E" };
    expect(googleMapsUrl(gps)).toBe("https://www.google.com/maps?q=0,0");
  });
});

describe("orientationName", () => {
  it("returns null for null input", () => {
    expect(orientationName(null)).toBeNull();
  });

  it("returns 'Normal' for orientation 1", () => {
    expect(orientationName(1)).toBe("Normal");
  });

  it("returns 'Flipped H' for orientation 2", () => {
    expect(orientationName(2)).toBe("Flipped H");
  });

  it("returns 'Rotated 180°' for orientation 3", () => {
    expect(orientationName(3)).toBe("Rotated 180°");
  });

  it("returns 'Flipped V' for orientation 4", () => {
    expect(orientationName(4)).toBe("Flipped V");
  });

  it("returns 'Rotated 90° CW' for orientation 6", () => {
    expect(orientationName(6)).toBe("Rotated 90° CW");
  });

  it("returns 'Rotated 90° CCW' for orientation 8", () => {
    expect(orientationName(8)).toBe("Rotated 90° CCW");
  });

  it("returns Unknown for unrecognized values", () => {
    expect(orientationName(0)).toBe("Unknown (0)");
    expect(orientationName(99)).toBe("Unknown (99)");
  });

  it("covers all 8 standard orientations", () => {
    for (let i = 1; i <= 8; i++) {
      const name = orientationName(i);
      expect(name).not.toBeNull();
      expect(name).not.toContain("Unknown");
    }
  });
});
