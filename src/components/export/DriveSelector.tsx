// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * DriveSelector — modal picker that lists system drives / volumes.
 *
 * Shows physical + logical drives with size, filesystem, kind, and mount point.
 * The user selects one (or more) and confirms; the caller receives the device
 * paths (or mount points) to add to the export source list.
 */

import { createSignal, createResource, Show, For, type Component } from "solid-js";
import {
  HiOutlineXMark,
  HiOutlineServer,
  HiOutlineArrowPath,
  HiOutlineLockClosed,
  HiOutlineLockOpen,
} from "../icons";
import { listDrives, formatDriveSize } from "../../api/drives";

export interface DriveSelectorProps {
  /** Whether the modal is visible */
  isOpen: boolean;
  /** Called when the user cancels / closes the modal */
  onClose: () => void;
  /** Called when the user confirms selection; receives chosen mount points and whether to remount read-only */
  onSelect: (paths: string[], mountReadOnly: boolean) => void;
  /** Allow multiple drive selection (default false — single-select) */
  multiple?: boolean;
}

const DriveSelector: Component<DriveSelectorProps> = (props) => {
  const [selected, setSelected] = createSignal<Set<string>>(new Set());
  const [mountReadOnly, setMountReadOnly] = createSignal(true);

  // Fetch drive list when modal opens
  const [drives, { refetch }] = createResource(
    () => props.isOpen,
    async (open) => {
      if (!open) return [];
      return listDrives();
    },
  );

  const toggleDrive = (path: string) => {
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(path)) {
        next.delete(path);
      } else {
        if (!props.multiple) next.clear();
        next.add(path);
      }
      return next;
    });
  };

  const handleConfirm = () => {
    const paths = Array.from(selected());
    if (paths.length > 0) {
      props.onSelect(paths, mountReadOnly());
      setSelected(new Set<string>());
    }
  };

  const handleClose = () => {
    setSelected(new Set<string>());
    props.onClose();
  };

  const kindIcon = (kind: string) => {
    switch (kind) {
      case "SSD":
        return "⚡";
      case "HDD":
        return "💿";
      default:
        return "💾";
    }
  };

  return (
    <Show when={props.isOpen}>
      <div class="modal-overlay" onClick={handleClose}>
        <div
          class="modal-content w-[560px]"
          onClick={(e) => e.stopPropagation()}
        >
          {/* Header */}
          <div class="modal-header">
            <div class="flex items-center gap-2">
              <HiOutlineServer class="w-5 h-5 text-accent" />
              <h2 class="text-base font-semibold text-txt">Select Drive</h2>
            </div>
            <button class="icon-btn-sm" onClick={handleClose} title="Close">
              <HiOutlineXMark class="w-4 h-4" />
            </button>
          </div>

          {/* Body */}
          <div class="modal-body max-h-[400px] overflow-y-auto space-y-1">
            <Show when={drives.loading}>
              <div class="flex items-center justify-center py-8 text-txt-muted text-sm">
                Scanning drives…
              </div>
            </Show>

            <Show when={drives.error}>
              <div class="text-error text-sm py-4 text-center">
                Failed to list drives: {String(drives.error)}
              </div>
            </Show>

            <Show when={!drives.loading && !drives.error && drives()?.length === 0}>
              <div class="text-txt-muted text-sm py-4 text-center italic">
                No drives detected
              </div>
            </Show>

            <For each={drives()}>
              {(drive) => {
                const isSelected = () => selected().has(drive.mountPoint);
                const usedPercent = () =>
                  drive.totalBytes > 0
                    ? Math.round((drive.usedBytes / drive.totalBytes) * 100)
                    : 0;

                return (
                  <button
                    class="w-full text-left p-3 rounded-lg border transition-colors"
                    classList={{
                      "border-accent bg-accent/10": isSelected(),
                      "border-border bg-bg-secondary hover:bg-bg-hover": !isSelected(),
                      "opacity-60": drive.isReadOnly,
                    }}
                    onClick={() => toggleDrive(drive.mountPoint)}
                    title={`Device: ${drive.devicePath}\nMount: ${drive.mountPoint}\nFS: ${drive.fileSystem}`}
                  >
                    <div class="flex items-center gap-3">
                      {/* Icon + name */}
                      <span class="text-lg" aria-hidden>
                        {kindIcon(drive.kind)}
                      </span>
                      <div class="flex-1 min-w-0">
                        <div class="flex items-center gap-2">
                          <span class="font-medium text-txt truncate">
                            {drive.mountPoint || drive.name}
                          </span>
                          <Show when={drive.isSystemDisk}>
                            <span class="badge badge-error text-[10px]">
                              System
                            </span>
                          </Show>
                          <Show when={drive.isRemovable}>
                            <span class="badge badge-warning text-[10px]">
                              Removable
                            </span>
                          </Show>
                          <Show when={drive.isReadOnly}>
                            <span class="badge badge-success text-[10px] flex items-center gap-0.5">
                              <HiOutlineLockClosed class="w-2.5 h-2.5" />
                              Read-only
                            </span>
                          </Show>
                        </div>
                        <div class="flex items-center gap-3 text-xs text-txt-muted mt-0.5">
                          <span>{drive.fileSystem.toUpperCase()}</span>
                          <span>{drive.kind}</span>
                          <span>{drive.devicePath}</span>
                        </div>
                      </div>

                      {/* Size info */}
                      <div class="text-right shrink-0">
                        <div class="text-sm font-medium text-txt">
                          {formatDriveSize(drive.totalBytes)}
                        </div>
                        <div class="text-xs text-txt-muted">
                          {formatDriveSize(drive.availableBytes)} free
                        </div>
                      </div>
                    </div>

                    {/* System disk warning */}
                    <Show when={drive.isSystemDisk}>
                      <div class="mt-2 flex items-center gap-1.5 text-[11px] text-warning">
                        <span>⚠</span>
                        <span>
                          This is the active system volume. Imaging a running OS disk may produce inconsistent data.
                        </span>
                      </div>
                    </Show>

                    {/* Usage bar */}
                    <div class="mt-2 h-1.5 bg-bg rounded-full overflow-hidden">
                      <div
                        class="h-full rounded-full transition-all"
                        classList={{
                          "bg-success": usedPercent() < 70,
                          "bg-warning": usedPercent() >= 70 && usedPercent() < 90,
                          "bg-error": usedPercent() >= 90,
                        }}
                        style={{ width: `${usedPercent()}%` }}
                      />
                    </div>
                  </button>
                );
              }}
            </For>
          </div>

          {/* Read-only mount option */}
          <div class="px-5 py-2 border-t border-border bg-bg-secondary/50">
            <label class="flex items-center gap-2 cursor-pointer select-none">
              <input
                type="checkbox"
                class="accent-accent"
                checked={mountReadOnly()}
                onChange={(e) => setMountReadOnly(e.currentTarget.checked)}
              />
              <div class="flex items-center gap-1.5 text-sm text-txt">
                <Show when={mountReadOnly()} fallback={<HiOutlineLockOpen class="w-4 h-4 text-txt-muted" />}>
                  <HiOutlineLockClosed class="w-4 h-4 text-accent" />
                </Show>
                Mount read-only before imaging
              </div>
            </label>
            <p class="text-[11px] text-txt-muted mt-1 ml-6">
              Remounts the selected volume as read-only to prevent accidental writes during acquisition.
              {" "}The volume will be restored to its original state after imaging completes.
            </p>
          </div>

          {/* Footer */}
          <div class="modal-footer justify-between">
            <button
              class="btn btn-ghost text-sm"
              onClick={() => refetch()}
              title="Refresh drive list"
            >
              <HiOutlineArrowPath class="w-4 h-4" />
              Refresh
            </button>
            <div class="flex gap-2">
              <button class="btn btn-secondary" onClick={handleClose}>
                Cancel
              </button>
              <button
                class="btn btn-primary"
                disabled={selected().size === 0}
                onClick={handleConfirm}
              >
                Select{selected().size > 0 ? ` (${selected().size})` : ""}
              </button>
            </div>
          </div>
        </div>
      </div>
    </Show>
  );
};

export default DriveSelector;
