import { Component, createSignal } from "solid-js";
import { RecoveryModal } from "./RecoveryModal";
import { ProfileSelector } from "./ProfileSelector";
import { TemplateGallery } from "./TemplateGallery";
import { ActivityHeatmap } from "./ActivityHeatmap";
import { ComparisonView } from "./ComparisonView";
import type { FFXProject } from "../../hooks/useActivityTimeline";
import {
  HiOutlineShieldCheck,
  HiOutlineFolder,
  HiOutlineClock,
  HiOutlineDocumentDuplicate,
} from "../icons";

interface ProjectToolbarProps {
  currentProject: FFXProject;
  comparisonProject?: FFXProject;
}

/**
 * Integrated project management toolbar
 * Provides access to all project enhancement features
 */
export const ProjectToolbar: Component<ProjectToolbarProps> = (props) => {
  const [showRecovery, setShowRecovery] = createSignal(false);
  const [showTemplates, setShowTemplates] = createSignal(false);
  const [showHeatmap, setShowHeatmap] = createSignal(false);
  const [showComparison, setShowComparison] = createSignal(false);

  return (
    <>
      {/* Toolbar */}
      <div class="flex items-center gap-3 p-4 bg-bg-panel border-b border-border">
        {/* Profile Selector */}
        <ProfileSelector
          onProfileChange={(profileId) => {
            console.log("Applied profile:", profileId);
          }}
        />

        <div class="w-px h-6 bg-border" /> {/* Divider */}

        {/* Backup & Health */}
        <button
          onClick={() => setShowRecovery(true)}
          class="btn btn-primary"
          title="Backup & Health Monitoring"
        >
          <HiOutlineShieldCheck class="w-icon-sm h-icon-sm" />
          <span class="hidden sm:inline">Backup & Health</span>
        </button>

        {/* Templates */}
        <button
          onClick={() => setShowTemplates(true)}
          class="btn btn-secondary"
          title="Project Templates"
        >
          <HiOutlineFolder class="w-icon-sm h-icon-sm" />
          <span class="hidden sm:inline">Templates</span>
        </button>

        {/* Activity Timeline */}
        <button
          onClick={() => setShowHeatmap(true)}
          class="px-3 py-2 bg-bg-secondary hover:bg-bg-hover text-txt rounded-md flex items-center gap-2 border border-border transition-colors"
          title="Activity Heatmap"
        >
          <HiOutlineClock class="w-icon-sm h-icon-sm" />
          <span class="hidden sm:inline">Activity</span>
        </button>

        {/* Project Comparison */}
        <button
          onClick={() => setShowComparison(true)}
          class="px-3 py-2 bg-bg-secondary hover:bg-bg-hover text-txt rounded-md flex items-center gap-2 border border-border transition-colors"
          title="Compare Projects"
          disabled={!props.comparisonProject}
        >
          <HiOutlineDocumentDuplicate class="w-icon-sm h-icon-sm" />
          <span class="hidden sm:inline">Compare</span>
        </button>

        <div class="flex-1" /> {/* Spacer */}

        <div class="text-sm text-txt-secondary">
          {props.currentProject.name || "Current Project"}
        </div>
      </div>

      {/* Modals */}
      <RecoveryModal
        isOpen={showRecovery()}
        onClose={() => setShowRecovery(false)}
        projectPath={props.currentProject.path}
      />

      <TemplateGallery
        isOpen={showTemplates()}
        onClose={() => setShowTemplates(false)}
        projectPath={props.currentProject.path}
        onTemplateApplied={(templateId: string) => {
          console.log("Applied template:", templateId);
          setShowTemplates(false);
        }}
      />

      <ActivityHeatmap
        isOpen={showHeatmap()}
        onClose={() => setShowHeatmap(false)}
        project={props.currentProject}
      />

      <ComparisonView
        isOpen={showComparison()}
        onClose={() => setShowComparison(false)}
        projectA={props.currentProject}
        projectB={props.comparisonProject!}
      />
    </>
  );
};
