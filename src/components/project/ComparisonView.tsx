import { Component, createSignal, Show, onMount } from "solid-js";
import { useProjectComparison } from "../../hooks/useProjectComparison";
import type { MergeStrategy } from "../../hooks/useProjectComparison";
import type { FFXProject } from "../../hooks/useActivityTimeline";
import {
  HiOutlineDocumentDuplicate,
  HiOutlineX,
  HiOutlineArrowsRightLeft,
  HiOutlineCheckCircle,
  HiOutlineExclamationCircle,
} from "../icons";
import { ComparisonSummaryBar } from "./ComparisonSummaryBar";
import { ComparisonTabs } from "./ComparisonTabs";
import { ComparisonColumn } from "./ComparisonColumn";
import { MergeStrategyModal } from "./MergeStrategyModal";

type ComparisonTab = "bookmarks" | "notes" | "evidence";

interface ComparisonViewProps {
  isOpen: boolean;
  onClose: () => void;
  projectA: FFXProject;
  projectB: FFXProject;
}

export const ComparisonView: Component<ComparisonViewProps> = (props) => {
  const comparison = useProjectComparison();

  const [activeTab, setActiveTab] = createSignal<ComparisonTab>("bookmarks");
  const [selectedStrategy, setSelectedStrategy] =
    createSignal<MergeStrategy>("PreferA");
  const [showMergePreview, setShowMergePreview] = createSignal(false);

  onMount(() => {
    comparison.compareProjects(props.projectA, props.projectB);
  });

  const handleMerge = async () => {
    const strategy = selectedStrategy();
    const success = await comparison.mergeProjects(
      props.projectA,
      props.projectB,
      strategy
    );
    if (success) {
      setShowMergePreview(false);
      props.onClose();
    }
  };

  const getComparisonData = () => comparison.comparison();

  if (!props.isOpen) return null;

  return (
    <>
    <div class="modal-overlay" onClick={props.onClose}>
      {/* Modal Content */}
      <div class="modal-content max-w-7xl w-full max-h-[90vh] flex flex-col" onClick={(e) => e.stopPropagation()}>
        {/* Header */}
        <div class="modal-header">
          <div class="flex items-center gap-3">
            <HiOutlineDocumentDuplicate class="w-icon-lg h-icon-lg text-accent" />
            <div>
              <h2 class="text-lg font-semibold text-txt">
                Project Comparison
              </h2>
              <p class="text-sm text-txt-secondary">
                {props.projectA.name || "Project A"} ↔{" "}
                {props.projectB.name || "Project B"}
              </p>
            </div>
          </div>
          <div class="flex items-center gap-2">
            <button
              onClick={() => setShowMergePreview(true)}
              class="btn-sm-primary"
            >
              <HiOutlineArrowsRightLeft class="w-icon-sm h-icon-sm" />
              Merge Projects
            </button>
            <button
              onClick={props.onClose}
              class="icon-btn"
            >
              <HiOutlineX class="w-icon-base h-icon-base" />
            </button>
          </div>
        </div>

          {/* Summary Bar */}
          <ComparisonSummaryBar comparison={getComparisonData() ?? undefined} />

          {/* Tabs */}
          <ComparisonTabs
            activeTab={activeTab()}
            onTabChange={setActiveTab}
            onSyncBookmarks={() => comparison.syncBookmarks(props.projectA, props.projectB)}
            onSyncNotes={() => comparison.syncNotes(props.projectA, props.projectB)}
          />

          {/* Content */}
          <div class="flex-1 overflow-auto p-4">
            <Show
              when={!comparison.loading()}
              fallback={
                <div class="flex items-center justify-center h-full text-txt-muted">
                  Comparing projects...
                </div>
              }
            >
              <Show when={getComparisonData()}>
                {(_comp) => (
                  <div class="grid grid-cols-3 gap-4 h-full">
                    {/* Only in A */}
                    <ComparisonColumn
                      title={`Only in ${props.projectA.name || "Project A"}`}
                      icon={HiOutlineCheckCircle}
                      iconColor="text-type-ad1"
                      activeTab={activeTab()}
                      bookmarks={comparison.getUniqueToA().bookmarks}
                      notes={comparison.getUniqueToA().notes}
                      evidence={comparison.getUniqueToA().evidence}
                    />

                    {/* Common / Modified */}
                    <ComparisonColumn
                      title="Common Items"
                      icon={HiOutlineCheckCircle}
                      iconColor="text-success"
                      activeTab={activeTab()}
                      bookmarks={comparison.getCommonItems().bookmarks}
                      notes={comparison.getCommonItems().notes}
                      evidence={comparison.getCommonItems().evidence}
                      modifiedBookmarks={comparison.getModifiedItems().bookmarks}
                      modifiedNotes={comparison.getModifiedItems().notes}
                      isCommonColumn={true}
                    />

                    {/* Only in B */}
                    <ComparisonColumn
                      title={`Only in ${props.projectB.name || "Project B"}`}
                      icon={HiOutlineCheckCircle}
                      iconColor="text-type-e01"
                      activeTab={activeTab()}
                      bookmarks={comparison.getUniqueToB().bookmarks}
                      notes={comparison.getUniqueToB().notes}
                      evidence={comparison.getUniqueToB().evidence}
                    />
                  </div>
                )}
              </Show>
            </Show>
          </div>

          {/* Conflicts Section */}
          <Show when={(getComparisonData()?.conflicts.length || 0) > 0}>
            <div class="p-4 border-t border-border bg-error/10">
              <div class="flex items-center gap-2 text-error">
                <HiOutlineExclamationCircle class="w-icon-base h-icon-base" />
                <span class="font-medium">
                  {getComparisonData()?.conflicts.length || 0} conflicts detected
                </span>
              </div>
              <p class="text-sm text-txt-secondary mt-1">
                Review conflicts before merging. Use merge strategy to resolve.
              </p>
            </div>
          </Show>
        </div>
      </div>

      {/* Merge Preview Modal */}
      <MergeStrategyModal
        isOpen={showMergePreview()}
        selectedStrategy={selectedStrategy()}
        onStrategyChange={setSelectedStrategy}
        onCancel={() => setShowMergePreview(false)}
        onConfirm={handleMerge}
      />
    </>
  );
};
