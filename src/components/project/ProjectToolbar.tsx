import { Component, createSignal } from "solid-js";
import { RecoveryModal } from "./RecoveryModal";
import { ProfileSelector } from "./ProfileSelector";
import TemplateGallery from "./TemplateGallery";
import ActivityHeatmap from "./ActivityHeatmap";
import ComparisonView from "./ComparisonView";
import {
  HiOutlineShieldCheck,
  HiOutlineFolder,
  HiOutlineClock,
  HiOutlineDocumentDuplicate,
} from "../icons";

interface ProjectToolbarProps {
  currentProjectPath: string;
  comparisonProjectPath?: string;
  currentProjectName?: string;
  comparisonProjectName?: string;
}

/**
 * Integrated project management toolbar
 * Provides access to all project enhancement features
 */
const ProjectToolbar: Component<ProjectToolbarProps> = (props) => {
  const [showRecovery, setShowRecovery] = createSignal(false);
  const [showTemplates, setShowTemplates] = createSignal(false);
  const [showHeatmap, setShowHeatmap] = createSignal(false);
  const [showComparison, setShowComparison] = createSignal(false);

  return (
    <>
      {/* Toolbar */}
      <div className="flex items-center gap-3 p-4 bg-bg-panel border-b border-border">
        {/* Profile Selector */}
        <ProfileSelector
          onProfileChange={(profileId) => {
            console.log("Applied profile:", profileId);
          }}
        />

        <div className="w-px h-6 bg-border" /> {/* Divider */}

        {/* Backup & Health */}
        <button
          onClick={() => setShowRecovery(true)}
          className="px-3 py-2 bg-accent hover:bg-accent-hover text-white rounded-md flex items-center gap-2 transition-colors"
          title="Backup & Health Monitoring"
        >
          <HiOutlineShieldCheck class="w-icon-sm h-icon-sm" />
          <span className="hidden sm:inline">Backup & Health</span>
        </button>

        {/* Templates */}
        <button
          onClick={() => setShowTemplates(true)}
          className="px-3 py-2 bg-bg-secondary hover:bg-bg-hover text-txt rounded-md flex items-center gap-2 border border-border transition-colors"
          title="Project Templates"
        >
          <HiOutlineFolder class="w-icon-sm h-icon-sm" />
          <span className="hidden sm:inline">Templates</span>
        </button>

        {/* Activity Timeline */}
        <button
          onClick={() => setShowHeatmap(true)}
          className="px-3 py-2 bg-bg-secondary hover:bg-bg-hover text-txt rounded-md flex items-center gap-2 border border-border transition-colors"
          title="Activity Heatmap"
        >
          <HiOutlineClock class="w-icon-sm h-icon-sm" />
          <span className="hidden sm:inline">Activity</span>
        </button>

        {/* Project Comparison */}
        <button
          onClick={() => setShowComparison(true)}
          className="px-3 py-2 bg-bg-secondary hover:bg-bg-hover text-txt rounded-md flex items-center gap-2 border border-border transition-colors"
          title="Compare Projects"
          disabled={!props.comparisonProjectPath}
        >
          <HiOutlineDocumentDuplicate class="w-icon-sm h-icon-sm" />
          <span className="hidden sm:inline">Compare</span>
        </button>

        <div className="flex-1" /> {/* Spacer */}

        <div className="text-sm text-txt-secondary">
          {props.currentProjectName || "Current Project"}
        </div>
      </div>

      {/* Modals */}
      <RecoveryModal
        isOpen={showRecovery()}
        onClose={() => setShowRecovery(false)}
        projectPath={props.currentProjectPath}
      />

      <TemplateGallery
        isOpen={showTemplates()}
        onClose={() => setShowTemplates(false)}
        projectPath={props.currentProjectPath}
        onTemplateApplied={(templateId) => {
          console.log("Applied template:", templateId);
          setShowTemplates(false);
        }}
      />

      <ActivityHeatmap
        isOpen={showHeatmap()}
        onClose={() => setShowHeatmap(false)}
        projectPath={props.currentProjectPath}
      />

      <ComparisonView
        isOpen={showComparison()}
        onClose={() => setShowComparison(false)}
        projectPathA={props.currentProjectPath}
        projectPathB={props.comparisonProjectPath || ""}
        projectNameA={props.currentProjectName}
        projectNameB={props.comparisonProjectName}
      />
    </>
  );
};

export default ProjectToolbar;
