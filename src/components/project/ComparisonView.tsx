import { Component, createSignal, For, Show, onMount } from "solid-js";
import { useProjectComparison } from "../../hooks/useProjectComparison";
import type {
  ProjectComparison,
  MergeStrategy,
} from "../../hooks/useProjectComparison";
import {
  HiOutlineDocumentDuplicate,
  HiOutlineX,
  HiOutlineArrowsRightLeft,
  HiOutlineCheckCircle,
  HiOutlineExclamationCircle,
  HiOutlineBookmark,
  HiOutlineDocumentText,
  HiOutlineFolder,
  HiOutlineClock,
} from "../icons";

type ComparisonTab = "bookmarks" | "notes" | "evidence" | "activity";

interface ComparisonViewProps {
  isOpen: boolean;
  onClose: () => void;
  projectPathA: string;
  projectPathB: string;
  projectNameA?: string;
  projectNameB?: string;
}

const ComparisonView: Component<ComparisonViewProps> = (props) => {
  const comparison = useProjectComparison();

  const [activeTab, setActiveTab] = createSignal<ComparisonTab>("bookmarks");
  const [selectedStrategy, setSelectedStrategy] =
    createSignal<MergeStrategy>("prefer_a");
  const [showMergePreview, setShowMergePreview] = createSignal(false);

  onMount(() => {
    comparison.compareProjects(props.projectPathA, props.projectPathB);
  });

  const strategies: Array<{ value: MergeStrategy; label: string; desc: string }> =
    [
      {
        value: "prefer_a",
        label: "Prefer A",
        desc: "Keep items from Project A when conflicts occur",
      },
      {
        value: "prefer_b",
        label: "Prefer B",
        desc: "Keep items from Project B when conflicts occur",
      },
      {
        value: "keep_both",
        label: "Keep Both",
        desc: "Keep all items from both projects",
      },
      {
        value: "skip",
        label: "Skip Conflicts",
        desc: "Skip conflicting items, keep only non-conflicting",
      },
      {
        value: "manual",
        label: "Manual Review",
        desc: "Mark conflicts for manual resolution",
      },
    ];

  const handleMerge = async () => {
    const strategy = selectedStrategy();
    const success = await comparison.mergeProjects(
      props.projectPathA,
      props.projectPathB,
      strategy
    );
    if (success) {
      setShowMergePreview(false);
      props.onClose();
    }
  };

  const handleSyncBookmarks = async () => {
    await comparison.syncBookmarks(props.projectPathA, props.projectPathB);
  };

  const handleSyncNotes = async () => {
    await comparison.syncNotes(props.projectPathA, props.projectPathB);
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
      {/* Modal Backdrop */}
      <div
        className="fixed inset-0 bg-black/50 z-modal-backdrop"
        onClick={props.onClose}
      />

      {/* Modal Content */}
      <div className="fixed inset-0 z-modal flex items-center justify-center p-4">
        <div className="bg-bg-panel rounded-lg border border-border w-full max-w-7xl max-h-[90vh] flex flex-col">
          {/* Header */}
          <div className="flex items-center justify-between p-4 border-b border-border">
            <div className="flex items-center gap-3">
              <HiOutlineDocumentDuplicate class="w-icon-lg h-icon-lg text-accent" />
              <div>
                <h2 className="text-lg font-semibold text-txt">
                  Project Comparison
                </h2>
                <p className="text-sm text-txt-secondary">
                  {props.projectNameA || "Project A"} ↔{" "}
                  {props.projectNameB || "Project B"}
                </p>
              </div>
            </div>
            <div className="flex items-center gap-2">
              <button
                onClick={() => setShowMergePreview(true)}
                className="px-3 py-2 bg-accent hover:bg-accent-hover text-white rounded-md flex items-center gap-2"
              >
                <HiOutlineArrowsRightLeft class="w-icon-sm h-icon-sm" />
                Merge Projects
              </button>
              <button
                onClick={props.onClose}
                className="p-2 hover:bg-bg-hover rounded-md text-txt-secondary hover:text-txt"
              >
                <HiOutlineX class="w-icon-base h-icon-base" />
              </button>
            </div>
          </div>

          {/* Summary Bar */}
          <Show when={getComparisonData()}>
            {(comp) => (
              <div className="p-4 border-b border-border bg-bg">
                <div className="grid grid-cols-5 gap-4 text-center">
                  <div>
                    <div className="text-2xl font-bold text-accent">
                      {comp().summary.unique_to_a + comp().summary.unique_to_b + comp().summary.common}
                    </div>
                    <div className="text-xs text-txt-muted">Total Items</div>
                  </div>
                  <div>
                    <div className="text-2xl font-bold text-success">
                      {comp().summary.common}
                    </div>
                    <div className="text-xs text-txt-muted">Common</div>
                  </div>
                  <div>
                    <div className="text-2xl font-bold text-warning">
                      {comp().summary.modified}
                    </div>
                    <div className="text-xs text-txt-muted">Modified</div>
                  </div>
                  <div>
                    <div className="text-2xl font-bold text-error">
                      {comp().conflicts.length}
                    </div>
                    <div className="text-xs text-txt-muted">Conflicts</div>
                  </div>
                  <div>
                    <div
                      className={`text-2xl font-bold ${getSimilarityColor(
                        comp().summary.similarity_percent
                      )}`}
                    >
                      {comp().summary.similarity_percent.toFixed(0)}%
                    </div>
                    <div className="text-xs text-txt-muted">Similarity</div>
                  </div>
                </div>
              </div>
            )}
          </Show>

          {/* Tabs */}
          <div className="flex items-center gap-2 p-4 border-b border-border">
            <button
              onClick={() => setActiveTab("bookmarks")}
              className={`px-4 py-2 rounded-md flex items-center gap-2 ${
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
              className={`px-4 py-2 rounded-md flex items-center gap-2 ${
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
              className={`px-4 py-2 rounded-md flex items-center gap-2 ${
                activeTab() === "evidence"
                  ? "bg-accent text-white"
                  : "bg-bg text-txt-secondary hover:text-txt hover:bg-bg-hover"
              }`}
            >
              <HiOutlineFolder class="w-icon-sm h-icon-sm" />
              Evidence
            </button>
            <button
              onClick={() => setActiveTab("activity")}
              className={`px-4 py-2 rounded-md flex items-center gap-2 ${
                activeTab() === "activity"
                  ? "bg-accent text-white"
                  : "bg-bg text-txt-secondary hover:text-txt hover:bg-bg-hover"
              }`}
            >
              <HiOutlineClock class="w-icon-sm h-icon-sm" />
              Activity
            </button>
            <div className="flex-1" />
            <Show when={activeTab() === "bookmarks"}>
              <button
                onClick={handleSyncBookmarks}
                className="px-3 py-2 bg-bg-secondary hover:bg-bg-hover text-txt rounded-md text-sm border border-border"
              >
                Sync Bookmarks →
              </button>
            </Show>
            <Show when={activeTab() === "notes"}>
              <button
                onClick={handleSyncNotes}
                className="px-3 py-2 bg-bg-secondary hover:bg-bg-hover text-txt rounded-md text-sm border border-border"
              >
                Sync Notes →
              </button>
            </Show>
          </div>

          {/* Content */}
          <div className="flex-1 overflow-auto p-4">
            <Show
              when={!comparison.loading()}
              fallback={
                <div className="flex items-center justify-center h-full text-txt-muted">
                  Comparing projects...
                </div>
              }
            >
              <Show when={getComparisonData()}>
                {(comp) => (
                  <div className="grid grid-cols-3 gap-4 h-full">
                    {/* Only in A */}
                    <div className="bg-bg rounded-lg border border-border p-4">
                      <h3 className="text-sm font-medium text-txt mb-3 flex items-center gap-2">
                        <HiOutlineCheckCircle class="w-icon-sm h-icon-sm text-type-ad1" />
                        Only in {props.projectNameA || "Project A"}
                      </h3>
                      <div className="space-y-2 max-h-96 overflow-auto">
                        <Show when={activeTab() === "bookmarks"}>
                          <For each={comparison.getUniqueToA().bookmarks}>
                            {(item: any) => (
                              <div className="p-2 bg-bg-secondary rounded text-sm">
                                <div className="font-medium text-txt truncate">
                                  {item.name || item.title || "Unnamed"}
                                </div>
                                <div className="text-xs text-txt-muted truncate">
                                  {item.target_path || item.path}
                                </div>
                              </div>
                            )}
                          </For>
                        </Show>
                        <Show when={activeTab() === "notes"}>
                          <For each={comparison.getUniqueToA().notes}>
                            {(item: any) => (
                              <div className="p-2 bg-bg-secondary rounded text-sm">
                                <div className="font-medium text-txt truncate">
                                  {item.title || "Untitled"}
                                </div>
                                <div className="text-xs text-txt-muted line-clamp-2">
                                  {item.content}
                                </div>
                              </div>
                            )}
                          </For>
                        </Show>
                        <Show when={activeTab() === "evidence"}>
                          <For each={comparison.getUniqueToA().evidence}>
                            {(item: any) => (
                              <div className="p-2 bg-bg-secondary rounded text-sm">
                                <div className="font-medium text-txt truncate">
                                  {item.name || item.path}
                                </div>
                                <div className="text-xs text-txt-muted">
                                  {item.type || "Unknown type"}
                                </div>
                              </div>
                            )}
                          </For>
                        </Show>
                        <Show when={activeTab() === "activity"}>
                          <For each={comparison.getUniqueToA().bookmarks}>
                            {(item: any) => (
                              <div className="p-2 bg-bg-secondary rounded text-sm">
                                <div className="font-medium text-txt truncate">
                                  {item.action || item.type}
                                </div>
                                <div className="text-xs text-txt-muted">
                                  {new Date(item.timestamp).toLocaleString()}
                                </div>
                              </div>
                            )}
                          </For>
                        </Show>
                      </div>
                      <div className="mt-3 text-xs text-txt-muted text-center">
                        {activeTab() === "bookmarks" && `${comparison.getUniqueToA().bookmarks.length} unique items`}
                        {activeTab() === "notes" && `${comparison.getUniqueToA().notes.length} unique items`}
                        {activeTab() === "evidence" && `${comparison.getUniqueToA().evidence.length} unique items`}
                        {activeTab() === "activity" && `${comparison.getUniqueToA().bookmarks.length} unique items`}
                      </div>
                    </div>

                    {/* Common / Modified */}
                    <div className="bg-bg rounded-lg border border-border p-4">
                      <h3 className="text-sm font-medium text-txt mb-3 flex items-center gap-2">
                        <HiOutlineCheckCircle class="w-icon-sm h-icon-sm text-success" />
                        Common Items
                      </h3>
                      <div className="space-y-2 max-h-96 overflow-auto">
                        <Show when={activeTab() === "bookmarks"}>
                          <For each={comparison.getCommonItems().bookmarks}>
                            {(item: any) => {
                              const isModified = comparison
                                .getModifiedItems()
                                .bookmarks.includes(item);
                              return (
                                <div
                                  className={`p-2 rounded text-sm ${
                                    isModified
                                      ? "bg-warning/20 border border-warning"
                                      : "bg-bg-secondary"
                                  }`}
                                >
                                  <div className="font-medium text-txt truncate">
                                    {item.name || item.title || "Unnamed"}
                                  </div>
                                  <div className="text-xs text-txt-muted truncate">
                                    {item.target_path || item.path}
                                  </div>
                                  <Show when={isModified}>
                                    <div className="text-xs text-warning mt-1">
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
                            {(item: any) => {
                              const isModified = comparison
                                .getModifiedItems()
                                .notes.includes(item);
                              return (
                                <div
                                  className={`p-2 rounded text-sm ${
                                    isModified
                                      ? "bg-warning/20 border border-warning"
                                      : "bg-bg-secondary"
                                  }`}
                                >
                                  <div className="font-medium text-txt truncate">
                                    {item.title || "Untitled"}
                                  </div>
                                  <div className="text-xs text-txt-muted line-clamp-2">
                                    {item.content}
                                  </div>
                                  <Show when={isModified}>
                                    <div className="text-xs text-warning mt-1">
                                      Modified in one project
                                    </div>
                                  </Show>
                                </div>
                              );
                            }}
                          </For>
                        </Show>
                      </div>
                      <div className="mt-3 text-xs text-txt-muted text-center">
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
                    <div className="bg-bg rounded-lg border border-border p-4">
                      <h3 className="text-sm font-medium text-txt mb-3 flex items-center gap-2">
                        <HiOutlineCheckCircle class="w-icon-sm h-icon-sm text-type-e01" />
                        Only in {props.projectNameB || "Project B"}
                      </h3>
                      <div className="space-y-2 max-h-96 overflow-auto">
                        <Show when={activeTab() === "bookmarks"}>
                          <For each={comparison.getUniqueToB().bookmarks}>
                            {(item: any) => (
                              <div className="p-2 bg-bg-secondary rounded text-sm">
                                <div className="font-medium text-txt truncate">
                                  {item.name || item.title || "Unnamed"}
                                </div>
                                <div className="text-xs text-txt-muted truncate">
                                  {item.target_path || item.path}
                                </div>
                              </div>
                            )}
                          </For>
                        </Show>
                        <Show when={activeTab() === "notes"}>
                          <For each={comparison.getUniqueToB().notes}>
                            {(item: any) => (
                              <div className="p-2 bg-bg-secondary rounded text-sm">
                                <div className="font-medium text-txt truncate">
                                  {item.title || "Untitled"}
                                </div>
                                <div className="text-xs text-txt-muted line-clamp-2">
                                  {item.content}
                                </div>
                              </div>
                            )}
                          </For>
                        </Show>
                        <Show when={activeTab() === "evidence"}>
                          <For each={comparison.getUniqueToB().evidence}>
                            {(item: any) => (
                              <div className="p-2 bg-bg-secondary rounded text-sm">
                                <div className="font-medium text-txt truncate">
                                  {item.name || item.path}
                                </div>
                                <div className="text-xs text-txt-muted">
                                  {item.type || "Unknown type"}
                                </div>
                              </div>
                            )}
                          </For>
                        </Show>
                        <Show when={activeTab() === "activity"}>
                          <For each={comparison.getUniqueToB().bookmarks}>
                            {(item: any) => (
                              <div className="p-2 bg-bg-secondary rounded text-sm">
                                <div className="font-medium text-txt truncate">
                                  {item.action || item.type}
                                </div>
                                <div className="text-xs text-txt-muted">
                                  {new Date(item.timestamp).toLocaleString()}
                                </div>
                              </div>
                            )}
                          </For>
                        </Show>
                      </div>
                      <div className="mt-3 text-xs text-txt-muted text-center">
                        {activeTab() === "bookmarks" && `${comparison.getUniqueToB().bookmarks.length} unique items`}
                        {activeTab() === "notes" && `${comparison.getUniqueToB().notes.length} unique items`}
                        {activeTab() === "evidence" && `${comparison.getUniqueToB().evidence.length} unique items`}
                        {activeTab() === "activity" && `${comparison.getUniqueToB().bookmarks.length} unique items`}
                      </div>
                    </div>
                  </div>
                )}
              </Show>
            </Show>
          </div>

          {/* Conflicts Section */}
          <Show when={(getComparisonData()?.conflicts.length || 0) > 0}>
            <div className="p-4 border-t border-border bg-error/10">
              <div className="flex items-center gap-2 text-error">
                <HiOutlineExclamationCircle class="w-icon-base h-icon-base" />
                <span className="font-medium">
                  {getComparisonData()?.conflicts.length || 0} conflicts detected
                </span>
              </div>
              <p className="text-sm text-txt-secondary mt-1">
                Review conflicts before merging. Use merge strategy to resolve.
              </p>
            </div>
          </Show>
        </div>
      </div>

      {/* Merge Preview Modal */}
      <Show when={showMergePreview()}>
        <div
          className="fixed inset-0 bg-black/50 z-modal-backdrop"
          onClick={() => setShowMergePreview(false)}
        />
        <div className="fixed inset-0 z-modal flex items-center justify-center p-4">
          <div className="bg-bg-panel rounded-lg border border-border w-full max-w-2xl">
            <div className="p-4 border-b border-border">
              <h3 className="text-lg font-semibold text-txt">
                Merge Projects - Strategy Selection
              </h3>
            </div>
            <div className="p-4 space-y-4">
              <p className="text-sm text-txt-secondary">
                Choose how to handle conflicts when merging:
              </p>
              <div className="space-y-2">
                <For each={strategies}>
                  {(strategy) => (
                    <label className="flex items-start gap-3 p-3 bg-bg rounded-md border border-border hover:bg-bg-hover cursor-pointer">
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
                        className="mt-1"
                      />
                      <div className="flex-1">
                        <div className="font-medium text-txt">
                          {strategy.label}
                        </div>
                        <div className="text-sm text-txt-secondary">
                          {strategy.desc}
                        </div>
                      </div>
                    </label>
                  )}
                </For>
              </div>
            </div>
            <div className="p-4 border-t border-border flex justify-end gap-2">
              <button
                onClick={() => setShowMergePreview(false)}
                className="px-4 py-2 bg-bg-secondary hover:bg-bg-hover text-txt rounded-md border border-border"
              >
                Cancel
              </button>
              <button
                onClick={handleMerge}
                className="px-4 py-2 bg-accent hover:bg-accent-hover text-white rounded-md"
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

export default ComparisonView;
