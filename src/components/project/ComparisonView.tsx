import { Component, createSignal, For, Show, onMount } from "solid-js";
import { useProjectComparison } from "../../hooks/useProjectComparison";
import type {
  MergeStrategy,
} from "../../hooks/useProjectComparison";
import type { FFXProject } from "../../hooks/useActivityTimeline";
import {
  HiOutlineDocumentDuplicate,
  HiOutlineX,
  HiOutlineArrowsRightLeft,
  HiOutlineCheckCircle,
  HiOutlineExclamationCircle,
  HiOutlineBookmark,
  HiOutlineDocumentText,
  HiOutlineFolder,
} from "../icons";

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

  const strategies: Array<{ value: MergeStrategy; label: string; desc: string }> =
    [
      {
        value: "PreferA",
        label: "Prefer A",
        desc: "Keep items from Project A when conflicts occur",
      },
      {
        value: "PreferB",
        label: "Prefer B",
        desc: "Keep items from Project B when conflicts occur",
      },
      {
        value: "KeepBoth",
        label: "Keep Both",
        desc: "Keep all items from both projects",
      },
      {
        value: "Skip",
        label: "Skip Conflicts",
        desc: "Skip conflicting items, keep only non-conflicting",
      },
      {
        value: "Manual",
        label: "Manual Review",
        desc: "Mark conflicts for manual resolution",
      },
    ];

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

  const handleSyncBookmarks = async () => {
    await comparison.syncBookmarks(props.projectA, props.projectB);
  };

  const handleSyncNotes = async () => {
    await comparison.syncNotes(props.projectA, props.projectB);
  };

  const getComparisonData = () => comparison.comparison();

  const getSimilarityColor = (similarity: number) => {
    if (similarity >= 80) return "text-success";
    if (similarity >= 50) return "text-warning";
    return "text-error";
  };

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
              class="btn btn-primary"
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
          <Show when={getComparisonData()}>
            {(comp) => (
              <div class="p-4 border-b border-border bg-bg">
                <div class="grid grid-cols-5 gap-4 text-center">
                  <div>
                    <div class="text-2xl font-bold text-accent">
                      {comp().summary.unique_to_a + comp().summary.unique_to_b + comp().summary.common}
                    </div>
                    <div class="text-xs text-txt-muted">Total Items</div>
                  </div>
                  <div>
                    <div class="text-2xl font-bold text-success">
                      {comp().summary.common}
                    </div>
                    <div class="text-xs text-txt-muted">Common</div>
                  </div>
                  <div>
                    <div class="text-2xl font-bold text-warning">
                      {comp().summary.modified}
                    </div>
                    <div class="text-xs text-txt-muted">Modified</div>
                  </div>
                  <div>
                    <div class="text-2xl font-bold text-error">
                      {comp().conflicts.length}
                    </div>
                    <div class="text-xs text-txt-muted">Conflicts</div>
                  </div>
                  <div>
                    <div
                      class={`text-2xl font-bold ${getSimilarityColor(
                        comp().summary.similarity_percent
                      )}`}
                    >
                      {comp().summary.similarity_percent.toFixed(0)}%
                    </div>
                    <div class="text-xs text-txt-muted">Similarity</div>
                  </div>
                </div>
              </div>
            )}
          </Show>

          {/* Tabs */}
          <div class="flex items-center gap-2 p-4 border-b border-border">
            <button
              onClick={() => setActiveTab("bookmarks")}
              class={`px-4 py-2 rounded-md flex items-center gap-2 ${
                activeTab() === "bookmarks"
                  ? "bg-accent text-white"
                  : "bg-bg text-txt-secondary hover:text-txt hover:bg-bg-hover"
              }`}
            >
              <HiOutlineBookmark class="w-icon-sm h-icon-sm" />
              Bookmarks
            </button>
            <button
              onClick={() => setActiveTab("notes")}
              class={`px-4 py-2 rounded-md flex items-center gap-2 ${
                activeTab() === "notes"
                  ? "bg-accent text-white"
                  : "bg-bg text-txt-secondary hover:text-txt hover:bg-bg-hover"
              }`}
            >
              <HiOutlineDocumentText class="w-icon-sm h-icon-sm" />
              Notes
            </button>
            <button
              onClick={() => setActiveTab("evidence")}
              class={`px-4 py-2 rounded-md flex items-center gap-2 ${
                activeTab() === "evidence"
                  ? "bg-accent text-white"
                  : "bg-bg text-txt-secondary hover:text-txt hover:bg-bg-hover"
              }`}
            >
              <HiOutlineFolder class="w-icon-sm h-icon-sm" />
              Evidence
            </button>
            <div class="flex-1" />
            <Show when={activeTab() === "bookmarks"}>
              <button
                onClick={handleSyncBookmarks}
                class="px-3 py-2 bg-bg-secondary hover:bg-bg-hover text-txt rounded-md text-sm border border-border"
              >
                Sync Bookmarks →
              </button>
            </Show>
            <Show when={activeTab() === "notes"}>
              <button
                onClick={handleSyncNotes}
                class="px-3 py-2 bg-bg-secondary hover:bg-bg-hover text-txt rounded-md text-sm border border-border"
              >
                Sync Notes →
              </button>
            </Show>
          </div>

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
                    <div class="bg-bg rounded-lg border border-border p-4">
                      <h3 class="text-sm font-medium text-txt mb-3 flex items-center gap-2">
                        <HiOutlineCheckCircle class="w-icon-sm h-icon-sm text-type-ad1" />
                        Only in {props.projectA.name || "Project A"}
                      </h3>
                      <div class="space-y-2 max-h-96 overflow-auto">
                        <Show when={activeTab() === "bookmarks"}>
                          <For each={comparison.getUniqueToA().bookmarks}>
                            {(name) => (
                              <div class="p-2 bg-bg-secondary rounded text-sm">
                                <div class="font-medium text-txt truncate">
                                  {name}
                                </div>
                              </div>
                            )}
                          </For>
                        </Show>
                        <Show when={activeTab() === "notes"}>
                          <For each={comparison.getUniqueToA().notes}>
                            {(title) => (
                              <div class="p-2 bg-bg-secondary rounded text-sm">
                                <div class="font-medium text-txt truncate">
                                  {title}
                                </div>
                              </div>
                            )}
                          </For>
                        </Show>
                        <Show when={activeTab() === "evidence"}>
                          <For each={comparison.getUniqueToA().evidence}>
                            {(path) => (
                              <div class="p-2 bg-bg-secondary rounded text-sm">
                                <div class="font-medium text-txt truncate">
                                  {path}
                                </div>
                              </div>
                            )}
                          </For>
                        </Show>
                      </div>
                      <div class="mt-3 text-xs text-txt-muted text-center">
                        {activeTab() === "bookmarks" && `${comparison.getUniqueToA().bookmarks.length} unique items`}
                        {activeTab() === "notes" && `${comparison.getUniqueToA().notes.length} unique items`}
                        {activeTab() === "evidence" && `${comparison.getUniqueToA().evidence.length} unique items`}
                      </div>
                    </div>

                    {/* Common / Modified */}
                    <div class="bg-bg rounded-lg border border-border p-4">
                      <h3 class="text-sm font-medium text-txt mb-3 flex items-center gap-2">
                        <HiOutlineCheckCircle class="w-icon-sm h-icon-sm text-success" />
                        Common Items
                      </h3>
                      <div class="space-y-2 max-h-96 overflow-auto">
                        <Show when={activeTab() === "bookmarks"}>
                          <For each={comparison.getCommonItems().bookmarks}>
                            {(name) => {
                              const isModified = comparison
                                .getModifiedItems()
                                .bookmarks.includes(name);
                              return (
                                <div
                                  class={`p-2 rounded text-sm ${
                                    isModified
                                      ? "bg-warning/20 border border-warning"
                                      : "bg-bg-secondary"
                                  }`}
                                >
                                  <div class="font-medium text-txt truncate">
                                    {name}
                                  </div>
                                  <Show when={isModified}>
                                    <div class="text-xs text-warning mt-1">
                                      Modified in one project
                                    </div>
                                  </Show>
                                </div>
                              );
                            }}
                          </For>
                        </Show>
                        <Show when={activeTab() === "notes"}>
                          <For each={comparison.getCommonItems().notes}>
                            {(title) => {
                              const isModified = comparison
                                .getModifiedItems()
                                .notes.includes(title);
                              return (
                                <div
                                  class={`p-2 rounded text-sm ${
                                    isModified
                                      ? "bg-warning/20 border border-warning"
                                      : "bg-bg-secondary"
                                  }`}
                                >
                                  <div class="font-medium text-txt truncate">
                                    {title}
                                  </div>
                                  <Show when={isModified}>
                                    <div class="text-xs text-warning mt-1">
                                      Modified in one project
                                    </div>
                                  </Show>
                                </div>
                              );
                            }}
                          </For>
                        </Show>
                        <Show when={activeTab() === "evidence"}>
                          <For each={comparison.getCommonItems().evidence}>
                            {(path) => (
                              <div class="p-2 bg-bg-secondary rounded text-sm">
                                <div class="font-medium text-txt truncate">
                                  {path}
                                </div>
                              </div>
                            )}
                          </For>
                        </Show>
                      </div>
                      <div class="mt-3 text-xs text-txt-muted text-center">
                        {activeTab() === "bookmarks" && `${comparison.getCommonItems().bookmarks.length} common items`}
                        {activeTab() === "notes" && `${comparison.getCommonItems().notes.length} common items`}
                        {activeTab() === "evidence" && `${comparison.getCommonItems().evidence.length} common items`}
                        {comparison.getModifiedItems().bookmarks.length > 0 &&
                          activeTab() === "bookmarks" &&
                          ` (${comparison.getModifiedItems().bookmarks.length} modified)`}
                        {comparison.getModifiedItems().notes.length > 0 &&
                          activeTab() === "notes" &&
                          ` (${comparison.getModifiedItems().notes.length} modified)`}
                      </div>
                    </div>

                    {/* Only in B */}
                    <div class="bg-bg rounded-lg border border-border p-4">
                      <h3 class="text-sm font-medium text-txt mb-3 flex items-center gap-2">
                        <HiOutlineCheckCircle class="w-icon-sm h-icon-sm text-type-e01" />
                        Only in {props.projectB.name || "Project B"}
                      </h3>
                      <div class="space-y-2 max-h-96 overflow-auto">
                        <Show when={activeTab() === "bookmarks"}>
                          <For each={comparison.getUniqueToB().bookmarks}>
                            {(name) => (
                              <div class="p-2 bg-bg-secondary rounded text-sm">
                                <div class="font-medium text-txt truncate">
                                  {name}
                                </div>
                              </div>
                            )}
                          </For>
                        </Show>
                        <Show when={activeTab() === "notes"}>
                          <For each={comparison.getUniqueToB().notes}>
                            {(title) => (
                              <div class="p-2 bg-bg-secondary rounded text-sm">
                                <div class="font-medium text-txt truncate">
                                  {title}
                                </div>
                              </div>
                            )}
                          </For>
                        </Show>
                        <Show when={activeTab() === "evidence"}>
                          <For each={comparison.getUniqueToB().evidence}>
                            {(path) => (
                              <div class="p-2 bg-bg-secondary rounded text-sm">
                                <div class="font-medium text-txt truncate">
                                  {path}
                                </div>
                              </div>
                            )}
                          </For>
                        </Show>
                      </div>
                      <div class="mt-3 text-xs text-txt-muted text-center">
                        {activeTab() === "bookmarks" && `${comparison.getUniqueToB().bookmarks.length} unique items`}
                        {activeTab() === "notes" && `${comparison.getUniqueToB().notes.length} unique items`}
                        {activeTab() === "evidence" && `${comparison.getUniqueToB().evidence.length} unique items`}
                      </div>
                    </div>
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
      <Show when={showMergePreview()}>
        <div class="modal-overlay" onClick={() => setShowMergePreview(false)}>
          <div class="modal-content max-w-2xl w-full" onClick={(e) => e.stopPropagation()}>
            <div class="modal-header">
              <h3 class="text-lg font-semibold text-txt">
                Merge Projects - Strategy Selection
              </h3>
            </div>
            <div class="modal-body space-y-4">
              <p class="text-sm text-txt-secondary">
                Choose how to handle conflicts when merging:
              </p>
              <div class="space-y-2">
                <For each={strategies}>
                  {(strategy) => (
                    <label class="flex items-start gap-3 p-3 bg-bg rounded-md border border-border hover:bg-bg-hover cursor-pointer">
                      <input
                        type="radio"
                        name="merge-strategy"
                        value={strategy.value}
                        checked={selectedStrategy() === strategy.value}
                        onChange={(e) =>
                          setSelectedStrategy(
                            e.currentTarget.value as MergeStrategy
                          )
                        }
                        class="mt-1"
                      />
                      <div class="flex-1">
                        <div class="font-medium text-txt">
                          {strategy.label}
                        </div>
                        <div class="text-sm text-txt-secondary">
                          {strategy.desc}
                        </div>
                      </div>
                    </label>
                  )}
                </For>
              </div>
            </div>
            <div class="modal-footer justify-end">
              <button
                onClick={() => setShowMergePreview(false)}
                class="btn btn-secondary"
              >
                Cancel
              </button>
              <button
                onClick={handleMerge}
                class="btn btn-primary"
              >
                Merge Projects
              </button>
            </div>
          </div>
        </div>
      </Show>
    </>
  );
};
