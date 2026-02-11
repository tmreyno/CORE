// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, onMount } from "solid-js";
import type { UseTourOptions } from "./types";

const TOUR_STORAGE_KEY = "ffx-tour-completed";

export function useTour(options: UseTourOptions) {
  const storageKey = options.storageKey ?? TOUR_STORAGE_KEY;
  
  const [isActive, setIsActive] = createSignal(false);
  const [currentStepIndex, setCurrentStepIndex] = createSignal(0);
  const [hasCompleted, setHasCompleted] = createSignal(false);

  // Check if tour was already completed
  onMount(() => {
    try {
      const completed = localStorage.getItem(storageKey);
      if (completed === "true") {
        setHasCompleted(true);
      } else if (options.autoStart) {
        start();
      }
    } catch (e) {}
  });

  const currentStep = () => options.steps[currentStepIndex()];
  const isFirstStep = () => currentStepIndex() === 0;
  const isLastStep = () => currentStepIndex() === options.steps.length - 1;
  const progress = () => ((currentStepIndex() + 1) / options.steps.length) * 100;

  const start = () => {
    setCurrentStepIndex(0);
    setIsActive(true);
    currentStep()?.onShow?.();
  };

  const next = () => {
    if (isLastStep()) {
      complete();
    } else {
      setCurrentStepIndex((i) => i + 1);
      currentStep()?.onShow?.();
    }
  };

  const previous = () => {
    if (!isFirstStep()) {
      setCurrentStepIndex((i) => i - 1);
      currentStep()?.onShow?.();
    }
  };

  const goTo = (index: number) => {
    if (index >= 0 && index < options.steps.length) {
      setCurrentStepIndex(index);
      currentStep()?.onShow?.();
    }
  };

  const skip = () => {
    setIsActive(false);
    options.onSkip?.();
  };

  const complete = () => {
    setIsActive(false);
    setHasCompleted(true);
    try {
      localStorage.setItem(storageKey, "true");
    } catch (e) {}
    options.onComplete?.();
  };

  const reset = () => {
    setHasCompleted(false);
    try {
      localStorage.removeItem(storageKey);
    } catch (e) {}
  };

  return {
    isActive,
    currentStep,
    currentStepIndex,
    isFirstStep,
    isLastStep,
    progress,
    hasCompleted,
    start,
    next,
    previous,
    goTo,
    skip,
    complete,
    reset,
  };
}
