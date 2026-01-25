// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Performance Monitor Panel - Dev-only component for tracking render performance
 * Shows FPS, memory usage, and component render metrics
 */

import { Component, Show, createSignal, For } from "solid-js";
import {
  useFPSMonitor,
  useMemoryMonitor,
  getAllRenderMetrics,
  getPerformanceSummary,
  clearPerformanceData,
  setPerformanceMonitoringEnabled,
  isPerformanceMonitoringEnabled,
  formatDuration,
  getPerformanceGrade,
} from "../utils/performance";
import {
  HiOutlineChartBar,
  HiOutlineXMark,
  HiOutlineTrash,
  HiOutlineCog6Tooth,
} from "./icons";

interface PerformancePanelProps {
  /** Whether the panel is visible */
  isOpen: boolean;
  /** Callback to close the panel */
  onClose: () => void;
}

/**
 * Performance Monitor Panel
 * Displays real-time performance metrics for debugging
 */
export const PerformancePanel: Component<PerformancePanelProps> = (props) => {
  const fpsMonitor = useFPSMonitor();
  const memory = useMemoryMonitor(2000);
  const [showDetails, setShowDetails] = createSignal(false);
  
  const summary = () => getPerformanceSummary();
  const renderMetrics = () => getAllRenderMetrics();
  const monitoringEnabled = () => isPerformanceMonitoringEnabled();
  
  const fpsColor = () => {
    const f = fpsMonitor.fps();
    if (f >= 55) return "text-success";
    if (f >= 30) return "text-warning";
    return "text-error";
  };
  
  const memoryPercent = () => {
    const m = memory();
    if (!m) return 0;
    return (m.usedJSHeapSize / m.jsHeapSizeLimit) * 100;
  };
  
  const gradeColor = (grade: string) => {
    switch (grade) {
      case "excellent": return "text-success";
      case "good": return "text-accent";
      case "fair": return "text-warning";
      case "poor": return "text-error";
      default: return "text-txt-muted";
    }
  };

  return (
    <Show when={props.isOpen}>
      <div class="fixed bottom-4 left-4 z-50 w-80 bg-bg-card border border-border rounded-lg shadow-xl">
        {/* Header */}
        <div class="flex items-center justify-between px-3 py-2 border-b border-border">
          <div class="flex items-center gap-2">
            <HiOutlineChartBar class="w-4 h-4 text-accent" />
            <span class="text-sm font-semibold text-txt">Performance Monitor</span>
          </div>
          <div class="flex items-center gap-1">
            <button
              class="p-1 text-txt-muted hover:text-txt rounded transition-colors"
              onClick={() => setShowDetails(v => !v)}
              title="Toggle details"
            >
              <HiOutlineCog6Tooth class="w-4 h-4" />
            </button>
            <button
              class="p-1 text-txt-muted hover:text-txt rounded transition-colors"
              onClick={clearPerformanceData}
              title="Clear metrics"
            >
              <HiOutlineTrash class="w-4 h-4" />
            </button>
            <button
              class="p-1 text-txt-muted hover:text-txt rounded transition-colors"
              onClick={props.onClose}
              title="Close"
            >
              <HiOutlineXMark class="w-4 h-4" />
            </button>
          </div>
        </div>
        
        {/* Main Stats */}
        <div class="p-3 space-y-3">
          {/* FPS */}
          <div class="flex items-center justify-between">
            <span class="text-xs text-txt-muted">FPS</span>
            <span class={`text-lg font-mono font-bold ${fpsColor()}`}>
              {fpsMonitor.fps().toFixed(0)}
            </span>
          </div>
          
          {/* Memory */}
          <div class="space-y-1">
            <div class="flex items-center justify-between">
              <span class="text-xs text-txt-muted">Memory</span>
              <span class="text-xs font-mono text-txt">
                {memory()?.usedHeapFormatted || "N/A"} / {memory()?.totalHeapFormatted || "N/A"}
              </span>
            </div>
            <div class="h-1.5 bg-bg-surface rounded-full overflow-hidden">
              <div 
                class="h-full bg-accent transition-all duration-300"
                style={{ width: `${memoryPercent()}%` }}
              />
            </div>
          </div>
          
          {/* Summary Stats */}
          <div class="grid grid-cols-2 gap-2 text-xs">
            <div class="bg-bg-surface rounded p-2">
              <div class="text-txt-muted">Total Renders</div>
              <div class="font-mono text-txt">{summary().totalRenders}</div>
            </div>
            <div class="bg-bg-surface rounded p-2">
              <div class="text-txt-muted">Avg Duration</div>
              <div class="font-mono text-txt">{formatDuration(summary().avgRenderDuration)}</div>
            </div>
            <div class="bg-bg-surface rounded p-2">
              <div class="text-txt-muted">Slowest</div>
              <div class="font-mono text-txt truncate" title={summary().slowestComponent || undefined}>
                {summary().slowestComponent || "N/A"}
              </div>
            </div>
            <div class="bg-bg-surface rounded p-2">
              <div class="text-txt-muted">Grade</div>
              <div class={`font-semibold ${gradeColor(getPerformanceGrade(summary().avgRenderDuration))}`}>
                {getPerformanceGrade(summary().avgRenderDuration).toUpperCase()}
              </div>
            </div>
          </div>
          
          {/* Monitoring Toggle */}
          <div class="flex items-center justify-between pt-2 border-t border-border">
            <span class="text-xs text-txt-muted">Enable Tracking</span>
            <button
              class={`px-2 py-1 text-xs rounded transition-colors ${
                monitoringEnabled() 
                  ? "bg-success/20 text-success" 
                  : "bg-bg-surface text-txt-muted"
              }`}
              onClick={() => setPerformanceMonitoringEnabled(!monitoringEnabled())}
            >
              {monitoringEnabled() ? "ON" : "OFF"}
            </button>
          </div>
        </div>
        
        {/* Details Panel */}
        <Show when={showDetails()}>
          <div class="border-t border-border max-h-48 overflow-y-auto">
            <div class="p-2 space-y-1">
              <div class="text-xs font-semibold text-txt-muted mb-2">Component Metrics</div>
              <For each={renderMetrics().slice(0, 10)}>
                {(metric) => (
                  <div class="flex items-center justify-between text-xs py-1 border-b border-border/50 last:border-0">
                    <span class="text-txt truncate max-w-[120px]" title={metric.name}>
                      {metric.name}
                    </span>
                    <div class="flex items-center gap-2 text-txt-muted">
                      <span>{metric.renderCount}x</span>
                      <span class="font-mono">{formatDuration(metric.avgRenderDuration)}</span>
                    </div>
                  </div>
                )}
              </For>
              <Show when={renderMetrics().length === 0}>
                <div class="text-xs text-txt-muted text-center py-2">
                  No render data yet
                </div>
              </Show>
            </div>
          </div>
        </Show>
        
        {/* Footer */}
        <div class="px-3 py-1.5 border-t border-border bg-bg-surface/50 rounded-b-lg">
          <div class="text-[10px] leading-tight text-txt-muted text-center">
            Dev Mode Only • Press Ctrl+Shift+P to toggle
          </div>
        </div>
      </div>
    </Show>
  );
};
