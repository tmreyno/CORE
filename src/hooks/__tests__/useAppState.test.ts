// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, vi, beforeEach } from "vitest";
import { createRoot } from "solid-js";

// ---------------------------------------------------------------------------
// Mocks
// ---------------------------------------------------------------------------

vi.mock("../../components/preferences", () => ({
  getPreference: vi.fn().mockReturnValue("hex"),
}));

import { getPreference } from "../../components/preferences";
const mockGetPreference = vi.mocked(getPreference);

import { useAppState } from "../useAppState";

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

beforeEach(() => {
  vi.clearAllMocks();
  mockGetPreference.mockReturnValue("hex");
});

// ---------------------------------------------------------------------------
// Modal State
// ---------------------------------------------------------------------------

describe("useAppState", () => {
  describe("modals", () => {
    it("initializes all modal states to false", () => {
      createRoot((dispose) => {
        const { modals } = useAppState();

        expect(modals.showCommandPalette()).toBe(false);
        expect(modals.showShortcutsModal()).toBe(false);
        expect(modals.showPerformancePanel()).toBe(false);
        expect(modals.showSettingsPanel()).toBe(false);
        expect(modals.showSearchPanel()).toBe(false);
        expect(modals.showWelcomeModal()).toBe(false);
        expect(modals.showReportWizard()).toBe(false);
        expect(modals.showProjectWizard()).toBe(false);
        dispose();
      });
    });

    it("toggles showCommandPalette", () => {
      createRoot((dispose) => {
        const { modals } = useAppState();

        modals.setShowCommandPalette(true);
        expect(modals.showCommandPalette()).toBe(true);

        modals.setShowCommandPalette(false);
        expect(modals.showCommandPalette()).toBe(false);
        dispose();
      });
    });

    it("toggles showSettingsPanel", () => {
      createRoot((dispose) => {
        const { modals } = useAppState();

        modals.setShowSettingsPanel(true);
        expect(modals.showSettingsPanel()).toBe(true);
        dispose();
      });
    });

    it("toggles showSearchPanel", () => {
      createRoot((dispose) => {
        const { modals } = useAppState();

        modals.setShowSearchPanel(true);
        expect(modals.showSearchPanel()).toBe(true);
        dispose();
      });
    });

    it("toggles showReportWizard", () => {
      createRoot((dispose) => {
        const { modals } = useAppState();

        modals.setShowReportWizard(true);
        expect(modals.showReportWizard()).toBe(true);
        dispose();
      });
    });

    it("toggles showProjectWizard", () => {
      createRoot((dispose) => {
        const { modals } = useAppState();

        modals.setShowProjectWizard(true);
        expect(modals.showProjectWizard()).toBe(true);
        dispose();
      });
    });

    it("toggles showWelcomeModal", () => {
      createRoot((dispose) => {
        const { modals } = useAppState();

        modals.setShowWelcomeModal(true);
        expect(modals.showWelcomeModal()).toBe(true);
        dispose();
      });
    });

    it("toggles showPerformancePanel", () => {
      createRoot((dispose) => {
        const { modals } = useAppState();

        modals.setShowPerformancePanel(true);
        expect(modals.showPerformancePanel()).toBe(true);
        dispose();
      });
    });

    it("toggles showShortcutsModal", () => {
      createRoot((dispose) => {
        const { modals } = useAppState();

        modals.setShowShortcutsModal(true);
        expect(modals.showShortcutsModal()).toBe(true);
        dispose();
      });
    });
  });

  // -------------------------------------------------------------------------
  // View State
  // -------------------------------------------------------------------------

  describe("views", () => {
    it("initializes openTabs as empty array", () => {
      createRoot((dispose) => {
        const { views } = useAppState();
        expect(views.openTabs()).toEqual([]);
        dispose();
      });
    });

    it("initializes currentViewMode to 'info'", () => {
      createRoot((dispose) => {
        const { views } = useAppState();
        expect(views.currentViewMode()).toBe("info");
        dispose();
      });
    });

    it("initializes hexMetadata to null", () => {
      createRoot((dispose) => {
        const { views } = useAppState();
        expect(views.hexMetadata()).toBeNull();
        dispose();
      });
    });

    it("initializes selectedContainerEntry to null", () => {
      createRoot((dispose) => {
        const { views } = useAppState();
        expect(views.selectedContainerEntry()).toBeNull();
        dispose();
      });
    });

    it("initializes entryContentViewMode from preference 'hex'", () => {
      mockGetPreference.mockReturnValue("hex");
      createRoot((dispose) => {
        const { views } = useAppState();
        expect(views.entryContentViewMode()).toBe("hex");
        dispose();
      });
    });

    it("maps 'preview' preference to 'document' view mode", () => {
      mockGetPreference.mockReturnValue("preview");
      createRoot((dispose) => {
        const { views } = useAppState();
        expect(views.entryContentViewMode()).toBe("document");
        dispose();
      });
    });

    it("maps 'auto' preference to 'auto' view mode", () => {
      mockGetPreference.mockReturnValue("auto");
      createRoot((dispose) => {
        const { views } = useAppState();
        expect(views.entryContentViewMode()).toBe("auto");
        dispose();
      });
    });

    it("maps 'text' preference to 'text' view mode", () => {
      mockGetPreference.mockReturnValue("text");
      createRoot((dispose) => {
        const { views } = useAppState();
        expect(views.entryContentViewMode()).toBe("text");
        dispose();
      });
    });

    it("falls back to 'hex' for unknown preference values", () => {
      mockGetPreference.mockReturnValue("unknown-value");
      createRoot((dispose) => {
        const { views } = useAppState();
        expect(views.entryContentViewMode()).toBe("hex");
        dispose();
      });
    });

    it("initializes requestViewMode to null", () => {
      createRoot((dispose) => {
        const { views } = useAppState();
        expect(views.requestViewMode()).toBeNull();
        dispose();
      });
    });

    it("initializes hexNavigator to null", () => {
      createRoot((dispose) => {
        const { views } = useAppState();
        expect(views.hexNavigator()).toBeNull();
        dispose();
      });
    });

    it("initializes treeExpansionState to null", () => {
      createRoot((dispose) => {
        const { views } = useAppState();
        expect(views.treeExpansionState()).toBeNull();
        dispose();
      });
    });

    it("sets and reads currentViewMode", () => {
      createRoot((dispose) => {
        const { views } = useAppState();
        views.setCurrentViewMode("hex");
        expect(views.currentViewMode()).toBe("hex");
        dispose();
      });
    });

    it("sets and reads entryContentViewMode", () => {
      createRoot((dispose) => {
        const { views } = useAppState();
        views.setEntryContentViewMode("document");
        expect(views.entryContentViewMode()).toBe("document");
        dispose();
      });
    });

    it("sets and reads requestViewMode", () => {
      createRoot((dispose) => {
        const { views } = useAppState();
        views.setRequestViewMode("export");
        expect(views.requestViewMode()).toBe("export");
        dispose();
      });
    });
  });

  // -------------------------------------------------------------------------
  // Project State
  // -------------------------------------------------------------------------

  describe("project", () => {
    it("initializes pendingProjectRoot to null", () => {
      createRoot((dispose) => {
        const { project } = useAppState();
        expect(project.pendingProjectRoot()).toBeNull();
        dispose();
      });
    });

    it("initializes caseDocumentsPath to null", () => {
      createRoot((dispose) => {
        const { project } = useAppState();
        expect(project.caseDocumentsPath()).toBeNull();
        dispose();
      });
    });

    it("initializes caseDocuments to null", () => {
      createRoot((dispose) => {
        const { project } = useAppState();
        expect(project.caseDocuments()).toBeNull();
        dispose();
      });
    });

    it("sets and reads pendingProjectRoot", () => {
      createRoot((dispose) => {
        const { project } = useAppState();
        project.setPendingProjectRoot("/evidence/case-001");
        expect(project.pendingProjectRoot()).toBe("/evidence/case-001");
        dispose();
      });
    });

    it("sets and reads caseDocumentsPath", () => {
      createRoot((dispose) => {
        const { project } = useAppState();
        project.setCaseDocumentsPath("/case/docs");
        expect(project.caseDocumentsPath()).toBe("/case/docs");
        dispose();
      });
    });
  });

  // -------------------------------------------------------------------------
  // Left Panel State
  // -------------------------------------------------------------------------

  describe("leftPanel", () => {
    it("initializes leftPanelTab to 'evidence'", () => {
      createRoot((dispose) => {
        const { leftPanel } = useAppState();
        expect(leftPanel.leftPanelTab()).toBe("evidence");
        dispose();
      });
    });

    it("initializes leftPanelMode to 'tabs'", () => {
      createRoot((dispose) => {
        const { leftPanel } = useAppState();
        expect(leftPanel.leftPanelMode()).toBe("tabs");
        dispose();
      });
    });

    it("sets and reads leftPanelTab", () => {
      createRoot((dispose) => {
        const { leftPanel } = useAppState();
        leftPanel.setLeftPanelTab("processed");
        expect(leftPanel.leftPanelTab()).toBe("processed");

        leftPanel.setLeftPanelTab("casedocs");
        expect(leftPanel.leftPanelTab()).toBe("casedocs");

        leftPanel.setLeftPanelTab("activity");
        expect(leftPanel.leftPanelTab()).toBe("activity");

        leftPanel.setLeftPanelTab("bookmarks");
        expect(leftPanel.leftPanelTab()).toBe("bookmarks");
        dispose();
      });
    });

    it("sets and reads leftPanelMode", () => {
      createRoot((dispose) => {
        const { leftPanel } = useAppState();
        leftPanel.setLeftPanelMode("unified");
        expect(leftPanel.leftPanelMode()).toBe("unified");

        leftPanel.setLeftPanelMode("tabs");
        expect(leftPanel.leftPanelMode()).toBe("tabs");
        dispose();
      });
    });
  });

  // -------------------------------------------------------------------------
  // Isolation
  // -------------------------------------------------------------------------

  describe("isolation", () => {
    it("returns independent state per call", () => {
      createRoot((dispose) => {
        const state1 = useAppState();
        const state2 = useAppState();

        state1.modals.setShowCommandPalette(true);
        expect(state1.modals.showCommandPalette()).toBe(true);
        expect(state2.modals.showCommandPalette()).toBe(false);
        dispose();
      });
    });
  });
});
