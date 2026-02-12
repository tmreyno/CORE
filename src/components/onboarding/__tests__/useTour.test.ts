// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, vi, beforeEach } from "vitest";
import { createRoot } from "solid-js";
import { useTour } from "../useTour";
import type { TourStep } from "../types";

// Mock localStorage
const localStorageMock = (() => {
  let store: Record<string, string> = {};
  return {
    getItem: vi.fn((key: string) => store[key] ?? null),
    setItem: vi.fn((key: string, val: string) => { store[key] = val; }),
    removeItem: vi.fn((key: string) => { delete store[key]; }),
    clear: vi.fn(() => { store = {}; }),
  };
})();
Object.defineProperty(globalThis, "localStorage", { value: localStorageMock });

function makeSteps(count = 3): TourStep[] {
  return Array.from({ length: count }, (_, i) => ({
    id: `step-${i + 1}`,
    title: `Step ${i + 1}`,
    content: `Content for step ${i + 1}`,
    position: "center" as const,
  }));
}

describe("useTour", () => {
  beforeEach(() => {
    localStorageMock.clear();
    vi.clearAllMocks();
  });

  it("starts inactive by default", () => {
    createRoot((dispose) => {
      const tour = useTour({ steps: makeSteps() });
      expect(tour.isActive()).toBe(false);
      expect(tour.currentStepIndex()).toBe(0);
      expect(tour.hasCompleted()).toBe(false);
      dispose();
    });
  });

  it("starts tour with start()", () => {
    createRoot((dispose) => {
      const tour = useTour({ steps: makeSteps() });
      tour.start();
      expect(tour.isActive()).toBe(true);
      expect(tour.currentStepIndex()).toBe(0);
      expect(tour.currentStep()?.id).toBe("step-1");
      dispose();
    });
  });

  it("advances through steps with next()", () => {
    createRoot((dispose) => {
      const tour = useTour({ steps: makeSteps() });
      tour.start();
      expect(tour.currentStepIndex()).toBe(0);
      expect(tour.isFirstStep()).toBe(true);
      expect(tour.isLastStep()).toBe(false);

      tour.next();
      expect(tour.currentStepIndex()).toBe(1);
      expect(tour.currentStep()?.id).toBe("step-2");
      expect(tour.isFirstStep()).toBe(false);
      expect(tour.isLastStep()).toBe(false);

      tour.next();
      expect(tour.currentStepIndex()).toBe(2);
      expect(tour.isLastStep()).toBe(true);
      dispose();
    });
  });

  it("goes back with previous()", () => {
    createRoot((dispose) => {
      const tour = useTour({ steps: makeSteps() });
      tour.start();
      tour.next();
      tour.next();
      expect(tour.currentStepIndex()).toBe(2);

      tour.previous();
      expect(tour.currentStepIndex()).toBe(1);

      tour.previous();
      expect(tour.currentStepIndex()).toBe(0);
      dispose();
    });
  });

  it("does not go before the first step", () => {
    createRoot((dispose) => {
      const tour = useTour({ steps: makeSteps() });
      tour.start();
      tour.previous();
      expect(tour.currentStepIndex()).toBe(0);
      dispose();
    });
  });

  it("completes the tour when next() is called on the last step", () => {
    const onComplete = vi.fn();
    createRoot((dispose) => {
      const tour = useTour({ steps: makeSteps(), onComplete });
      tour.start();
      tour.next(); // step 2
      tour.next(); // step 3 (last)
      tour.next(); // should complete

      expect(tour.isActive()).toBe(false);
      expect(tour.hasCompleted()).toBe(true);
      expect(onComplete).toHaveBeenCalledOnce();
      expect(localStorageMock.setItem).toHaveBeenCalledWith("ffx-tour-completed", "true");
      dispose();
    });
  });

  it("skips the tour", () => {
    const onSkip = vi.fn();
    createRoot((dispose) => {
      const tour = useTour({ steps: makeSteps(), onSkip });
      tour.start();
      expect(tour.isActive()).toBe(true);

      tour.skip();
      expect(tour.isActive()).toBe(false);
      expect(tour.hasCompleted()).toBe(false);
      expect(onSkip).toHaveBeenCalledOnce();
      dispose();
    });
  });

  it("jumps to a specific step with goTo()", () => {
    createRoot((dispose) => {
      const tour = useTour({ steps: makeSteps() });
      tour.start();
      tour.goTo(2);
      expect(tour.currentStepIndex()).toBe(2);
      expect(tour.currentStep()?.id).toBe("step-3");
      dispose();
    });
  });

  it("does not goTo an out-of-range index", () => {
    createRoot((dispose) => {
      const tour = useTour({ steps: makeSteps() });
      tour.start();
      tour.goTo(99);
      expect(tour.currentStepIndex()).toBe(0); // unchanged
      tour.goTo(-1);
      expect(tour.currentStepIndex()).toBe(0); // unchanged
      dispose();
    });
  });

  it("reports progress correctly", () => {
    createRoot((dispose) => {
      const steps = makeSteps(4);
      const tour = useTour({ steps });
      tour.start();
      expect(tour.progress()).toBeCloseTo(25); // 1/4
      tour.next();
      expect(tour.progress()).toBeCloseTo(50); // 2/4
      tour.next();
      expect(tour.progress()).toBeCloseTo(75); // 3/4
      dispose();
    });
  });

  it("uses a custom storageKey", () => {
    const customKey = "my-custom-tour-key";
    createRoot((dispose) => {
      const tour = useTour({ steps: makeSteps(), storageKey: customKey });
      tour.start();
      tour.next();
      tour.next();
      tour.next(); // complete
      expect(localStorageMock.setItem).toHaveBeenCalledWith(customKey, "true");
      dispose();
    });
  });

  it("resets the tour completion", () => {
    createRoot((dispose) => {
      const tour = useTour({ steps: makeSteps() });
      tour.start();
      tour.next();
      tour.next();
      tour.next(); // complete
      expect(tour.hasCompleted()).toBe(true);

      tour.reset();
      expect(tour.hasCompleted()).toBe(false);
      expect(localStorageMock.removeItem).toHaveBeenCalledWith("ffx-tour-completed");
      dispose();
    });
  });

  it("calls onShow callback when step changes", () => {
    const onShow1 = vi.fn();
    const onShow2 = vi.fn();
    const steps: TourStep[] = [
      { id: "s1", title: "S1", content: "C1", onShow: onShow1 },
      { id: "s2", title: "S2", content: "C2", onShow: onShow2 },
    ];

    createRoot((dispose) => {
      const tour = useTour({ steps });
      tour.start();
      expect(onShow1).toHaveBeenCalledOnce();

      tour.next();
      expect(onShow2).toHaveBeenCalledOnce();
      dispose();
    });
  });
});
