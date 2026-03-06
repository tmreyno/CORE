// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================
// Shim - re-exports from decomposed onboarding/ subdirectory

export * from "./onboarding/types";
export { useTour } from "./onboarding/useTour";
export { TourOverlay } from "./onboarding/TourOverlay";
export type { TourOverlayProps } from "./onboarding/TourOverlay";
export { Tooltip, HelpButton } from "./onboarding/Tooltip";
export { WelcomeModal } from "./onboarding/WelcomeModal";
export { DEFAULT_TOUR_STEPS } from "./onboarding/defaultTourSteps";
export { formatRelativeTime } from "./onboarding/welcomeHelpers";
export type { RecentProjectInfo, WelcomeModalProps } from "./onboarding/welcomeTypes";

// Animation keyframes
const styleSheet = document.createElement("style");
styleSheet.textContent = `
  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(-4px); }
    to { opacity: 1; transform: translateY(0); }
  }
`;
document.head.appendChild(styleSheet);
