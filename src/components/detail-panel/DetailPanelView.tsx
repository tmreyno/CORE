// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * DetailPanel — tab-based detail view with info, hex, text, PDF
 * and export modes. Tab management is delegated to useDetailPanelTabs.
 */

import { Show } from "solid-js";
import { DetailPanelContent } from "./DetailPanelContent";
import { useDetailPanelTabs } from "./useDetailPanelTabs";
import { HexViewer } from "../HexViewer";
import { TextViewer } from "../TextViewer";
import { PdfViewer } from "../PdfViewer";
import { ExportPanel } from "../ExportPanel";
import { TabBar } from "../TabBar";
import type { TabViewMode, OpenTab } from "../TabBar";
import { Breadcrumb } from "../Breadcrumb";
import { logger } from "../../utils/logger";
import type { DetailPanelProps } from "./DetailPanelTypes";

// Re-export TabViewMode and OpenTab for backward compatibility
export type { TabViewMode, OpenTab };

export function DetailPanel(props: DetailPanelProps) {
  const tabs = useDetailPanelTabs({
    activeFile: () => props.activeFile,
    onTabSelect: props.onTabSelect,
    onTabsChange: props.onTabsChange,
    onViewModeChange: props.onViewModeChange,
    requestViewMode: () => props.requestViewMode,
    onViewModeRequestHandled: props.onViewModeRequestHandled,
  });

  // Data accessors for the active tab
  const activeFileInfo = () => {
    const file = tabs.activeTabFile();
    return file ? props.fileInfoMap().get(file.path) : undefined;
  };
  const activeFileHash = () => {
    const file = tabs.activeTabFile();
    return file ? props.fileHashMap().get(file.path) : undefined;
  };
  const activeFileStatus = () => {
    const file = tabs.activeTabFile();
    return file ? props.fileStatusMap().get(file.path) : undefined;
  };
  const activeFileHashHistory = () => {
    const file = tabs.activeTabFile();
    return file ? props.hashHistory().get(file.path) ?? [] : [];
  };

  return (
    <main
      class="flex flex-col h-full overflow-hidden"
      role="main"
      aria-label="File detail view"
    >
      <TabBar
        tabs={tabs.openTabs()}
        activeTabId={tabs.activeTabId()}
        viewMode={tabs.getActiveViewMode()}
        onTabSelect={tabs.selectTab}
        onTabClose={tabs.closeTab}
        onCloseOthers={tabs.closeOtherTabs}
        onCloseAll={tabs.closeAllTabs}
        onCloseToRight={tabs.closeTabsToRight}
        onTabMove={tabs.moveTab}
        onViewModeChange={tabs.setActiveViewMode}
        onCopyPath={tabs.copyPath}
      />

      {/* Breadcrumb navigation */}
      <Show
        when={props.breadcrumbItems && props.breadcrumbItems.length > 0}
      >
        <div class="px-2 py-0.5 border-b border-border/50 bg-bg/50">
          <Breadcrumb
            items={props.breadcrumbItems!}
            onNavigate={(path) => props.onBreadcrumbNavigate?.(path)}
          />
        </div>
      </Show>

      {/* Content area */}
      <div
        class="flex-1 overflow-y-auto min-h-0"
        role="tabpanel"
        aria-label={`${tabs.getActiveViewMode()} view for ${tabs.activeTabFile()?.filename || "no file selected"}`}
      >
        <Show when={tabs.getActiveViewMode() === "info"}>
          <DetailPanelContent
            activeFile={tabs.activeTabFile()}
            fileInfo={activeFileInfo()}
            fileHash={activeFileHash()}
            fileStatus={activeFileStatus()}
            tree={props.tree}
            filteredTree={props.filteredTree}
            treeFilter={props.treeFilter}
            onTreeFilterChange={props.onTreeFilterChange}
            selectedHashAlgorithm={props.selectedHashAlgorithm}
            hashHistory={activeFileHashHistory()}
            storedHashes={props.storedHashesGetter(activeFileInfo())}
            busy={props.busy}
            onLoadInfo={() =>
              tabs.activeTabFile() &&
              props.onLoadInfo(tabs.activeTabFile()!)
            }
            formatHashDate={props.formatHashDate}
          />
        </Show>

        <Show when={tabs.getActiveViewMode() === "hex" && tabs.activeTabFile()}>
          <HexViewer
            file={tabs.activeTabFile()!}
            onMetadataLoaded={props.onMetadataLoaded}
            onNavigatorReady={props.onHexNavigatorReady}
          />
        </Show>

        <Show when={tabs.getActiveViewMode() === "text" && tabs.activeTabFile()}>
          <TextViewer file={tabs.activeTabFile()!} />
        </Show>

        <Show when={tabs.getActiveViewMode() === "pdf" && tabs.activeTabFile()}>
          <PdfViewer path={tabs.activeTabFile()!.path} />
        </Show>

        <Show when={tabs.getActiveViewMode() === "export"}>
          <ExportPanel
            initialSources={props.selectedFiles?.map((f) => f.path) || []}
            onComplete={(destination) => {
              logger.debug("Export completed:", destination);
            }}
          />
        </Show>
      </div>
    </main>
  );
}
