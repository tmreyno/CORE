// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, beforeEach } from "vitest";
import { render } from "solid-js/web";
import { ToastProvider, useToast } from "./Toast";
import { onMount } from "solid-js";

// Helper to render and return the container
function renderComponent(component: () => any) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const dispose = render(component, container);
  return { container, dispose };
}

// Wait for async updates
const tick = () => new Promise(resolve => setTimeout(resolve, 50));

describe("Toast System", () => {
  beforeEach(() => {
    // Clean up any previous renders
    document.body.innerHTML = "";
  });

  describe("ToastProvider", () => {
    it("renders children correctly", () => {
      const { container } = renderComponent(() => (
        <ToastProvider>
          <div data-testid="child">Child Content</div>
        </ToastProvider>
      ));

      expect(container.textContent).toContain("Child Content");
    });

    it("provides toast context to children", async () => {
      let toastContext: ReturnType<typeof useToast> | null = null;
      
      renderComponent(() => (
        <ToastProvider>
          {(() => {
            onMount(() => {
              toastContext = useToast();
            });
            return <div>Test</div>;
          })()}
        </ToastProvider>
      ));

      await tick();
      
      expect(toastContext).not.toBeNull();
      expect(toastContext!.success).toBeDefined();
      expect(toastContext!.error).toBeDefined();
    });
  });

  describe("useToast hook", () => {
    it("throws error when used outside provider", () => {
      expect(() => {
        useToast();
      }).toThrow("useToast must be used within a ToastProvider");
    });

    it("adds success toast", async () => {
      renderComponent(() => (
        <ToastProvider>
          {(() => {
            onMount(() => {
              const toast = useToast();
              toast.success("Success!", "Operation completed");
            });
            return <div>Test</div>;
          })()}
        </ToastProvider>
      ));

      await tick();
      
      expect(document.body.textContent).toContain("Success!");
    });

    it("adds error toast", async () => {
      renderComponent(() => (
        <ToastProvider>
          {(() => {
            onMount(() => {
              const toast = useToast();
              toast.error("Error", "Something went wrong");
            });
            return <div>Test</div>;
          })()}
        </ToastProvider>
      ));

      await tick();
      
      expect(document.body.textContent).toContain("Error");
    });
  });
});
