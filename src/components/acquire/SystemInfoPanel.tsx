// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * SystemInfoPanel — Right-panel component showing system identification data
 * collected during the Identify System phase on the Acquire dashboard.
 *
 * Uses the shared right-panel primitives (CollapsibleGroup, MetadataRow, etc.)
 * for consistent styling with other CORE-FFX right-panel components.
 */

import { Component, Show, For } from "solid-js";
import {
  HiOutlineComputerDesktop,
  HiOutlineGlobeAlt,
  HiOutlineCircleStack,
} from "../icons";
import { CollapsibleGroup, MetadataRow, OptionalMetadataRow } from "../viewerMetadata/shared";
import type { SystemStats } from "../../hooks";
import type { DriveInfo } from "../../api/drives";

// ─── Formatters (shared with AcquireDashboard) ──────────────────────────────

const formatBytes = (bytes: number): string => {
  if (bytes <= 0) return "0 B";
  const gb = bytes / (1024 * 1024 * 1024);
  if (gb >= 1) return `${gb.toFixed(gb >= 10 ? 0 : 1)} GB`;
  const mb = bytes / (1024 * 1024);
  return `${mb.toFixed(0)} MB`;
};

const formatUptime = (secs: number): string => {
  const days = Math.floor(secs / 86400);
  const hours = Math.floor((secs % 86400) / 3600);
  const mins = Math.floor((secs % 3600) / 60);
  const parts: string[] = [];
  if (days > 0) parts.push(`${days}d`);
  if (hours > 0) parts.push(`${hours}h`);
  parts.push(`${mins}m`);
  return parts.join(" ");
};

const formatDriveSize = (bytes: number): string => {
  const gb = bytes / (1024 * 1024 * 1024);
  if (gb >= 1024) return `${(gb / 1024).toFixed(1)} TB`;
  if (gb >= 1) return `${gb.toFixed(gb >= 100 ? 0 : 1)} GB`;
  return `${(bytes / (1024 * 1024)).toFixed(0)} MB`;
};

// ─── Component ──────────────────────────────────────────────────────────────

interface SystemInfoPanelProps {
  systemStats: SystemStats | null;
  drives?: DriveInfo[];
}

const SystemInfoPanel: Component<SystemInfoPanelProps> = (props) => {
  const stats = () => props.systemStats;

  return (
    <div class="flex flex-col h-full bg-bg">
      {/* Header */}
      <div class="flex items-center justify-between px-3 py-2 border-b border-border bg-bg-secondary shrink-0">
        <span class="text-xs font-medium text-txt">System Information</span>
        <HiOutlineComputerDesktop class="w-icon-sm h-icon-sm text-txt-muted" />
      </div>

      {/* Empty state — no system info gathered yet */}
      <Show when={!stats()}>
        <div class="flex flex-col items-center justify-center flex-1 py-8 text-txt-muted text-sm gap-2 px-4">
          <HiOutlineComputerDesktop class="w-8 h-8 opacity-30" />
          <span class="text-center text-xs">
            Run <strong class="text-txt">Identify System</strong> on the Dashboard to populate system information.
          </span>
        </div>
      </Show>

      {/* Scrollable body — shown when system info is available */}
      <Show when={stats()}>
        {(s) => (
        <div class="flex-1 overflow-y-auto p-3 space-y-3">
          {/* System Identity */}
          <CollapsibleGroup title="System Identity" defaultOpen={true}>
            <OptionalMetadataRow label="Model" value={s().systemModel} mono />
            <OptionalMetadataRow label="Serial" value={s().systemSerialNumber} mono />
            <OptionalMetadataRow label="Manufacturer" value={s().systemManufacturer} />
            <OptionalMetadataRow label="Hostname" value={s().hostname} mono />
            <OptionalMetadataRow
              label="OS"
              value={s().longOsVersion || `${s().osName} ${s().osVersion}`}
            />
            <OptionalMetadataRow label="Kernel" value={s().kernelVersion} mono />
            <OptionalMetadataRow label="Timezone" value={s().timezone} mono />
          </CollapsibleGroup>

          {/* Processor */}
          <CollapsibleGroup title="Processor" defaultOpen={true}>
            <OptionalMetadataRow label="CPU" value={s().cpuBrand} />
            <OptionalMetadataRow label="Vendor" value={s().cpuVendor} />
            <OptionalMetadataRow label="Architecture" value={s().cpuArch} mono />
            <MetadataRow
              label="Cores"
              value={
                s().physicalCores > 0
                  ? `${s().cpuCores} logical / ${s().physicalCores} physical`
                  : `${s().cpuCores} logical`
              }
            />
            <Show when={s().cpuFrequencyMhz > 0}>
              <MetadataRow
                label="Frequency"
                value={
                  s().cpuFrequencyMhz >= 1000
                    ? `${(s().cpuFrequencyMhz / 1000).toFixed(2)} GHz`
                    : `${s().cpuFrequencyMhz} MHz`
                }
              />
            </Show>
          </CollapsibleGroup>

          {/* Memory */}
          <CollapsibleGroup title="Memory" defaultOpen={true}>
            <MetadataRow label="Total RAM" value={formatBytes(s().memoryTotal)} />
            <MetadataRow
              label="Used"
              value={`${formatBytes(s().memoryUsed)} (${s().memoryPercent.toFixed(1)}%)`}
            />
            <Show when={s().totalSwap > 0}>
              <MetadataRow
                label="Swap"
                value={`${formatBytes(s().usedSwap)} / ${formatBytes(s().totalSwap)}`}
              />
            </Show>
          </CollapsibleGroup>

          {/* Timing */}
          <CollapsibleGroup title="Timing" defaultOpen={true}>
            <Show when={s().uptimeSecs > 0}>
              <MetadataRow label="Uptime" value={formatUptime(s().uptimeSecs)} />
            </Show>
            <Show when={s().bootTimeEpoch > 0}>
              <MetadataRow
                label="Last Boot"
                value={new Date(s().bootTimeEpoch * 1000).toLocaleString()}
                mono
              />
            </Show>
          </CollapsibleGroup>

          {/* Volumes / Drives */}
          <Show when={props.drives && props.drives.length > 0}>
            <CollapsibleGroup
              title="Volumes"
              defaultOpen={true}
              trailing={
                <span class="text-2xs text-txt-muted bg-bg-secondary rounded px-1.5 py-0.5">
                  {props.drives!.length}
                </span>
              }
            >
              <For each={props.drives}>
                {(drive) => (
                  <div class="py-1 border-b border-border/20 last:border-b-0 space-y-0.5">
                    <div class="flex items-center gap-1.5 text-xs">
                      <HiOutlineCircleStack class="w-3 h-3 text-txt-muted shrink-0" />
                      <span class="text-txt font-medium truncate" title={drive.mountPoint}>
                        {drive.name || drive.mountPoint}
                      </span>
                      <Show when={drive.isSystemDisk}>
                        <span class="text-2xs text-warning bg-warning/10 rounded px-1 shrink-0">System</span>
                      </Show>
                    </div>
                    <div class="pl-4.5 space-y-0.5">
                      <OptionalMetadataRow label="Mount" value={drive.mountPoint} mono truncate />
                      <OptionalMetadataRow label="FS Type" value={drive.fileSystem} mono />
                      <OptionalMetadataRow label="Model" value={drive.model} />
                      <OptionalMetadataRow label="Serial" value={drive.serial} mono />
                      <OptionalMetadataRow label="Vendor" value={drive.vendor} />
                      <OptionalMetadataRow label="Connection" value={drive.connectionType} />
                      <MetadataRow
                        label="Capacity"
                        value={`${formatDriveSize(drive.availableBytes)} free / ${formatDriveSize(drive.totalBytes)}`}
                      />
                    </div>
                  </div>
                )}
              </For>
            </CollapsibleGroup>
          </Show>

          {/* Network Interfaces */}
          <Show when={s().networkInterfaces.length > 0}>
            <CollapsibleGroup
              title="Network Interfaces"
              defaultOpen={false}
              trailing={
                <span class="text-2xs text-txt-muted bg-bg-secondary rounded px-1.5 py-0.5">
                  {s().networkInterfaces.length}
                </span>
              }
            >
              <For each={s().networkInterfaces}>
                {(iface) => (
                  <div class="py-1 border-b border-border/20 last:border-b-0 space-y-0.5">
                    <div class="flex items-center gap-1.5 text-xs">
                      <HiOutlineGlobeAlt class="w-3 h-3 text-txt-muted shrink-0" />
                      <span class="text-txt font-medium">{iface.name}</span>
                    </div>
                    <div class="pl-4.5 space-y-0.5">
                      <MetadataRow label="MAC" value={iface.macAddress} mono />
                      <Show when={iface.ipAddresses.length > 0}>
                        <MetadataRow label="IP" value={iface.ipAddresses.join(", ")} mono />
                      </Show>
                    </div>
                  </div>
                )}
              </For>
            </CollapsibleGroup>
          </Show>
        </div>
        )}
      </Show>
    </div>
  );
};

export default SystemInfoPanel;
