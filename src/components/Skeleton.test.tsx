// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import { render } from "solid-js/web";
import { Skeleton, SkeletonFileRow, SkeletonLoader } from "./Skeleton";

// Helper to render and return the container
function renderComponent(component: () => any) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  render(component, container);
  return container;
}

describe("Skeleton Components", () => {
  describe("Skeleton", () => {
    it("renders with default props", () => {
      const container = renderComponent(() => <Skeleton />);
      
      const skeleton = container.querySelector(".animate-pulse");
      expect(skeleton).toBeTruthy();
    });

    it("applies custom className", () => {
      const container = renderComponent(() => <Skeleton class="custom-class" />);
      
      const skeleton = container.querySelector(".custom-class");
      expect(skeleton).toBeTruthy();
    });
  });

  describe("SkeletonFileRow", () => {
    it("renders file row skeleton structure", () => {
      const container = renderComponent(() => <SkeletonFileRow />);
      
      // Should have multiple skeleton elements for file row
      const skeletons = container.querySelectorAll(".animate-pulse");
      expect(skeletons.length).toBeGreaterThan(0);
    });
  });

  describe("SkeletonLoader", () => {
    it("renders with message", () => {
      const container = renderComponent(() => <SkeletonLoader message="Loading files..." />);
      
      expect(container.textContent).toContain("Loading files...");
    });

    it("renders without message", () => {
      const container = renderComponent(() => <SkeletonLoader />);
      
      // Should render the spinner element
      const spinner = container.querySelector(".animate-spin");
      expect(spinner).toBeTruthy();
    });
  });
});
