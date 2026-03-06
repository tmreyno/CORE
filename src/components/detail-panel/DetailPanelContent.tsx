// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * DetailPanelContent — main detail panel body showing file header,
 * stats, hash progress, container details, and the file tree.
 */

import { Show, createSignal, createEffect, createMemo } from "solid-js";
import {
  HiOutlineClipboardDocument,
  HiOutlineInformationCircle,
} from "../icons";
import { debounce } from "@solid-primitives/scheduled";
import { FileHeader } from "./FileHeader";
import { StatsRow } from "./StatsRow";
import { HashDisplay } from "./HashDisplay";
import { HashHistory } from "./HashHistory";
import { FileTree } from "./FileTree";
import { ContainerDetails } from "./ContainerDetails";
import type { DetailPanelContentProps } from "./types";

export function DetailPanelContent(props: DetailPanelContentProps) {
  // Local state for immediate input feedback, with debounced propagation
  const [localTreeFilter, setLocalTreeFilter] = createSignal(props.treeFilter);

  const debouncedFilterChange = debounce((value: string) => {
    props.onTreeFilterChange(value);
  }, 150);

  // Sync local filter when prop changes (e.g., from external clear)
  createEffect(() => {
    setLocalTreeFilter(props.treeFilter);
  });

  const handleTreeFilterInput = (value: string) => {
    setLocalTreeFilter(value);
    debouncedFilterChange(value);
  };

  // Memoized computed values for status checks
  const isHashing = createMemo(() => props.fileStatus?.status === "hashing");
  const currentProgress = createMemo(() => props.fileStatus?.progress ?? 0);

  // Memoized container type accessors
  const ad1Info = createMemo(() => props.fileInfo?.ad1);
  const e01Info = createMemo(() => props.fileInfo?.e01);
  const ufedInfo = createMemo(() => props.fileInfo?.ufed);

  // Memoized date accessors
  const acquiryDate = createMemo(
    () =>
      e01Info()?.acquiry_date ||
      ad1Info()?.companion_log?.acquisition_date ||
      ufedInfo()?.extraction_info?.start_time,
  );
  const hasAcquiryDate = createMemo(() => !!acquiryDate());

  // Memoized tree info
  const treeCount = createMemo(() => props.tree.length);
  const hasTree = createMemo(() => treeCount() > 0);
  const treeExceedsLimit = createMemo(() => treeCount() > 500);

  // Memoized hash history (reversed once, not on every render)
  const reversedHashHistory = createMemo(() =>
    props.hashHistory.slice().reverse(),
  );

  return (
    <main class="flex flex-col flex-1 min-h-0 overflow-y-auto bg-bg p-4">
      <Show
        when={props.activeFile}
        keyed
        fallback={
          <div class="flex flex-col items-center justify-center flex-1 text-txt-muted gap-2">
            <HiOutlineClipboardDocument class="w-12 h-12 opacity-50" />
            <p>Select a file to view details</p>
          </div>
        }
      >
        {(file) => {
          return (
            <div class="flex flex-col gap-4">
              <FileHeader file={file} />

              <StatsRow
                file={file}
                ad1Info={ad1Info}
                e01Info={e01Info}
                ufedInfo={ufedInfo}
                hasAcquiryDate={hasAcquiryDate}
              />

              <HashDisplay
                fileHash={props.fileHash}
                isHashing={isHashing}
                currentProgress={currentProgress}
                selectedHashAlgorithm={props.selectedHashAlgorithm}
                storedHashes={props.storedHashes}
              />

              <HashHistory
                hashHistory={props.hashHistory}
                reversedHashHistory={reversedHashHistory}
              />

              <Show when={props.fileInfo}>
                <ContainerDetails
                  info={props.fileInfo!}
                  storedHashes={props.storedHashes}
                />
              </Show>

              <FileTree
                tree={props.tree}
                filteredTree={props.filteredTree}
                localTreeFilter={localTreeFilter}
                treeCount={treeCount}
                hasTree={hasTree}
                treeExceedsLimit={treeExceedsLimit}
                handleTreeFilterInput={handleTreeFilterInput}
              />

              <div class="flex flex-wrap gap-2 pt-2">
                <Show when={!props.fileInfo}>
                  <button
                    class="btn-sm"
                    onClick={props.onLoadInfo}
                    disabled={props.busy}
                  >
                    <HiOutlineInformationCircle class="w-3 h-3" /> Load Info
                  </button>
                </Show>
              </div>
            </div>
          );
        }}
      </Show>
    </main>
  );
}
