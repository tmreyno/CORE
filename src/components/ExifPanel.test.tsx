// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, beforeEach } from "vitest";
import { render } from "solid-js/web";
import { ExifPanel } from "./ExifPanel";
import { mockInvoke } from "../__tests__/setup";

// Helper to render and return the container
function renderComponent(component: () => any) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const dispose = render(component, container);
  return { container, dispose };
}

// Wait for async updates
const tick = (ms = 50) => new Promise(resolve => setTimeout(resolve, ms));

// Mock EXIF data (matches Rust ExifMetadata struct)
const mockExifData = {
  path: "/tmp/photo.jpg",
  make: "Canon",
  model: "EOS R5",
  software: "Adobe Lightroom 6.0",
  lens_model: "RF 24-70mm F2.8L IS USM",
  exposure_time: "1/250",
  f_number: "f/2.8",
  iso: 400,
  focal_length: "50mm",
  flash: "No flash",
  date_time_original: "2024-01-15 10:30:00",
  date_time_digitized: "2024-01-15 10:30:00",
  date_time: "2024-01-16 14:20:00",
  gps_timestamp: "10:30:00",
  gps: {
    latitude: 37.7749,
    longitude: -122.4194,
    altitude: 10.5,
    lat_ref: "N",
    lon_ref: "W",
  },
  width: 8192,
  height: 5464,
  orientation: 1,
  color_space: "sRGB",
  image_unique_id: "abc123def456",
  serial_number: "012345678",
  owner_name: "John Photographer",
  raw_tags: [
    ["Make", "Canon"],
    ["Model", "EOS R5"],
    ["ExposureTime", "1/250"],
    ["FNumber", "f/2.8"],
    ["ISO", "400"],
  ] as [string, string][],
};

const mockExifNoGps = {
  ...mockExifData,
  gps: null,
};

describe("ExifPanel", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
    mockInvoke.mockReset();
  });

  describe("Rendering", () => {
    it("renders camera information", async () => {
      mockInvoke.mockResolvedValueOnce(mockExifData);

      const { container } = renderComponent(() => (
        <ExifPanel path="/tmp/photo.jpg" />
      ));
      await tick();

      expect(container.textContent).toContain("Canon");
      expect(container.textContent).toContain("EOS R5");
    });

    it("calls exif_extract with the file path", async () => {
      mockInvoke.mockResolvedValueOnce(mockExifData);

      renderComponent(() => <ExifPanel path="/tmp/photo.jpg" />);
      await tick();

      expect(mockInvoke).toHaveBeenCalledWith("exif_extract", { path: "/tmp/photo.jpg" });
    });

    it("renders capture settings", async () => {
      mockInvoke.mockResolvedValueOnce(mockExifData);

      const { container } = renderComponent(() => (
        <ExifPanel path="/tmp/photo.jpg" />
      ));
      await tick();

      expect(container.textContent).toContain("1/250");
      expect(container.textContent).toContain("f/2.8");
      expect(container.textContent).toContain("400");
    });

    it("renders timestamps", async () => {
      mockInvoke.mockResolvedValueOnce(mockExifData);

      const { container } = renderComponent(() => (
        <ExifPanel path="/tmp/photo.jpg" />
      ));
      await tick();

      expect(container.textContent).toContain("2024-01-15");
    });

    it("renders GPS coordinates", async () => {
      mockInvoke.mockResolvedValueOnce(mockExifData);

      const { container } = renderComponent(() => (
        <ExifPanel path="/tmp/photo.jpg" />
      ));
      await tick();

      expect(container.textContent).toContain("37");
      expect(container.textContent).toContain("122");
    });

    it("renders image dimensions", async () => {
      mockInvoke.mockResolvedValueOnce(mockExifData);

      const { container } = renderComponent(() => (
        <ExifPanel path="/tmp/photo.jpg" />
      ));
      await tick();

      expect(container.textContent).toContain("8192");
      expect(container.textContent).toContain("5464");
    });

    it("renders forensic indicators when present", async () => {
      mockInvoke.mockResolvedValueOnce(mockExifData);

      const { container } = renderComponent(() => (
        <ExifPanel path="/tmp/photo.jpg" />
      ));
      await tick();

      expect(container.textContent).toContain("abc123def456");
      expect(container.textContent).toContain("012345678");
    });
  });

  describe("Loading and error states", () => {
    it("shows loading state initially", () => {
      mockInvoke.mockReturnValue(new Promise(() => {}));

      const { container } = renderComponent(() => (
        <ExifPanel path="/tmp/photo.jpg" />
      ));

      const spinner = container.querySelector(".animate-spin");
      expect(spinner).toBeTruthy();
    });

    it("shows error state when extraction fails", async () => {
      mockInvoke.mockRejectedValueOnce(new Error("Not an image file"));

      const { container } = renderComponent(() => (
        <ExifPanel path="/tmp/bad.txt" />
      ));
      await tick();

      expect(container.textContent).toContain("No EXIF data available");
    });
  });

  describe("Edge cases", () => {
    it("handles image with no GPS data", async () => {
      mockInvoke.mockResolvedValueOnce(mockExifNoGps);

      const { container } = renderComponent(() => (
        <ExifPanel path="/tmp/photo.jpg" />
      ));
      await tick();

      // Should render without GPS section
      expect(container.textContent).toContain("Canon");
      expect(container.textContent).not.toContain("Google Maps");
    });

    it("handles image with no forensic identifiers", async () => {
      mockInvoke.mockResolvedValueOnce({
        ...mockExifData,
        image_unique_id: null,
        serial_number: null,
        owner_name: null,
      });

      const { container } = renderComponent(() => (
        <ExifPanel path="/tmp/photo.jpg" />
      ));
      await tick();

      // Should render camera info without forensic section
      expect(container.textContent).toContain("Canon");
    });

    it("handles minimal EXIF data", async () => {
      mockInvoke.mockResolvedValueOnce({
        path: "/tmp/photo.jpg",
        make: null,
        model: null,
        software: null,
        lens_model: null,
        exposure_time: null,
        f_number: null,
        iso: null,
        focal_length: null,
        flash: null,
        date_time_original: null,
        date_time_digitized: null,
        date_time: null,
        gps_timestamp: null,
        gps: null,
        width: null,
        height: null,
        orientation: null,
        color_space: null,
        image_unique_id: null,
        serial_number: null,
        owner_name: null,
        raw_tags: [],
      });

      const { container } = renderComponent(() => (
        <ExifPanel path="/tmp/photo.jpg" />
      ));
      await tick();

      // Should render without crashing
      expect(container.innerHTML).toBeTruthy();
    });
  });
});
