// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { lazy, type Component, type Accessor, type Setter } from "solid-js";
import { CommandPalette, KeyboardShortcutsModal, DEFAULT_SHORTCUT_GROUPS, ContextMenu, WelcomeModal, TourOverlay, DEFAULT_TOUR_STEPS, ProjectSetupWizard, SearchPanel, type RecentProjectInfo } from "../index";
import type { CommandAction, ContextMenuItem, SearchFilter, SearchResult, ProjectLocations } from "../index";

// Lazy-loaded heavy components with named exports
const PerformancePanel = lazy(() => import("../PerformancePanel").then(m => ({ default: m.PerformancePanel })));
const SettingsPanel = lazy(() => import("../SettingsPanel").then(m => ({ default: m.SettingsPanel })));

export interface TourState {
  isActive: () => boolean;
  currentStep: () => typeof DEFAULT_TOUR_STEPS[number] | undefined;
  currentStepIndex: () => number;
  progress: () => number;
  isFirstStep: () => boolean;
  isLastStep: () => boolean;
  next: () => void;
  previous: () => void;
  skip: () => void;
  start: () => void;
  hasCompleted: () => boolean;
}

export interface FileContextMenuState {
  items: Accessor<ContextMenuItem[]>;
  position: Accessor<{ x: number; y: number } | null>;
  close: () => void;
}

export interface AppModalsProps {
  // Command Palette
  commandPaletteActions: CommandAction[];
  showCommandPalette: Accessor<boolean>;
  setShowCommandPalette: Setter<boolean>;
  
  // Keyboard Shortcuts Modal
  showShortcutsModal: Accessor<boolean>;
  setShowShortcutsModal: Setter<boolean>;
  
  // Performance Panel
  showPerformancePanel: Accessor<boolean>;
  setShowPerformancePanel: Setter<boolean>;
  
  // Settings Panel
  showSettingsPanel: Accessor<boolean>;
  setShowSettingsPanel: Setter<boolean>;
  preferences: import("../preferences").AppPreferences;
  onUpdatePreference: <K extends keyof import("../preferences").AppPreferences>(key: K, value: import("../preferences").AppPreferences[K]) => void;
  onUpdateShortcut: (action: string, shortcut: string) => void;
  onResetToDefaults: () => void;
  
  // Search Panel
  showSearchPanel: Accessor<boolean>;
  setShowSearchPanel: Setter<boolean>;
  onSearch: (query: string, filters: SearchFilter) => Promise<SearchResult[]>;
  onSelectSearchResult: (result: SearchResult) => void;
  
  // Context Menus
  fileContextMenu: FileContextMenuState;
  saveContextMenu: FileContextMenuState;
  
  // Welcome Modal
  showWelcomeModal: Accessor<boolean>;
  setShowWelcomeModal: Setter<boolean>;
  /** Callback to create a new project */
  onNewProject?: () => void;
  /** Callback to open an existing project */
  onOpenProject?: () => void;
  /** Recent projects to display in welcome modal */
  recentProjects?: Accessor<RecentProjectInfo[]>;
  /** Callback when a recent project is selected from welcome modal */
  onSelectRecentProject?: (path: string) => void;
  
  // Tour
  tour: TourState;
  
  // Project Setup Wizard
  showProjectWizard: Accessor<boolean>;
  setShowProjectWizard: Setter<boolean>;
  pendingProjectRoot: Accessor<string | null>;
  setPendingProjectRoot: Setter<string | null>;
  onProjectSetupComplete: (locations: ProjectLocations) => Promise<void>;
}

/**
 * AppModals - Contains all modal and overlay components.
 * This extraction reduces App.tsx size while keeping modals organized.
 */
export const AppModals: Component<AppModalsProps> = (props) => {
  return (
    <>
      {/* Command Palette */}
      <CommandPalette
        actions={props.commandPaletteActions}
        isOpen={props.showCommandPalette()}
        onClose={() => props.setShowCommandPalette(false)}
        placeholder="Search commands..."
      />
      
      {/* Keyboard Shortcuts Modal */}
      <KeyboardShortcutsModal
        isOpen={props.showShortcutsModal()}
        onClose={() => props.setShowShortcutsModal(false)}
        groups={DEFAULT_SHORTCUT_GROUPS}
      />
      
      {/* Performance Panel (dev mode) */}
      <PerformancePanel
        isOpen={props.showPerformancePanel()}
        onClose={() => props.setShowPerformancePanel(false)}
      />
      
      {/* Settings Panel */}
      <SettingsPanel
        isOpen={props.showSettingsPanel()}
        onClose={() => props.setShowSettingsPanel(false)}
        preferences={props.preferences}
        onUpdatePreference={props.onUpdatePreference}
        onUpdateShortcut={props.onUpdateShortcut}
        onResetToDefaults={props.onResetToDefaults}
      />
      
      {/* Search Panel */}
      <SearchPanel
        isOpen={props.showSearchPanel()}
        onClose={() => props.setShowSearchPanel(false)}
        onSearch={props.onSearch}
        onSelectResult={props.onSelectSearchResult}
        placeholder="Search files and container contents..."
      />
      
      {/* File Context Menu */}
      <ContextMenu
        items={props.fileContextMenu.items()}
        position={props.fileContextMenu.position()}
        onClose={props.fileContextMenu.close}
      />
      
      {/* Save Context Menu */}
      <ContextMenu
        items={props.saveContextMenu.items()}
        position={props.saveContextMenu.position()}
        onClose={props.saveContextMenu.close}
      />
      
      {/* Welcome Modal (first-time users) */}
      <WelcomeModal
        isOpen={props.showWelcomeModal()}
        onClose={() => {
          props.setShowWelcomeModal(false);
          localStorage.setItem("ffx-welcome-seen", "true");
        }}
        onStartTour={() => {
          props.setShowWelcomeModal(false);
          localStorage.setItem("ffx-welcome-seen", "true");
          props.tour.start();
        }}
        onNewProject={props.onNewProject}
        onOpenProject={props.onOpenProject}
        recentProjects={props.recentProjects}
        onSelectRecentProject={props.onSelectRecentProject}
      />
      
      {/* Tour Overlay (guided onboarding) */}
      <TourOverlay
        isActive={props.tour.isActive()}
        step={props.tour.currentStep()}
        stepIndex={props.tour.currentStepIndex()}
        totalSteps={DEFAULT_TOUR_STEPS.length}
        progress={props.tour.progress()}
        isFirst={props.tour.isFirstStep()}
        isLast={props.tour.isLastStep()}
        onNext={props.tour.next}
        onPrevious={props.tour.previous}
        onSkip={props.tour.skip}
      />
      
      {/* Project Setup Wizard */}
      <ProjectSetupWizard
        projectRoot={props.pendingProjectRoot() || ''}
        isOpen={props.showProjectWizard()}
        onClose={() => {
          props.setShowProjectWizard(false);
          props.setPendingProjectRoot(null);
        }}
        onComplete={props.onProjectSetupComplete}
      />
    </>
  );
};
