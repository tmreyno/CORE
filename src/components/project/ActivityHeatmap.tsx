import { Component, createSignal, For, Show, onMount, createMemo } from "solid-js";
import { useActivityTimeline, type FFXProject } from "../../hooks/useActivityTimeline";
import {
  HiOutlineCalendar,
  HiOutlineArrowDownTray,
  HiOutlineClock,
  HiOutlineChartBar,
  HiOutlineX,
} from "../icons";

interface ActivityHeatmapProps {
  isOpen: boolean;
  onClose: () => void;
  project: FFXProject;
}

export const ActivityHeatmap: Component<ActivityHeatmapProps> = (props) => {
  const timeline = useActivityTimeline();
  const [selectedCell, setSelectedCell] = createSignal<{
    day: number;
    hour: number;
    count: number;
  } | null>(null);
  const [hoveredCell, setHoveredCell] = createSignal<{
    day: number;
    hour: number;
    count: number;
  } | null>(null);

  onMount(() => {
    timeline.computeVisualization(props.project);
  });

  const dayLabels = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
  const hourLabels = Array.from({ length: 24 }, (_, i) => i);

  // Get color intensity based on activity count
  const getIntensityColor = (count: number, maxCount: number) => {
    if (count === 0) return "bg-bg";
    const intensity = maxCount > 0 ? count / maxCount : 0;
    
    if (intensity > 0.75) return "bg-accent";
    if (intensity > 0.5) return "bg-accent/75";
    if (intensity > 0.25) return "bg-accent/50";
    return "bg-accent/25";
  };

  // Calculate max activity count for color scaling
  const maxActivityCount = createMemo(() => {
    const heatmap = timeline.getHeatmapData();
    if (!heatmap || !heatmap.data || heatmap.data.length === 0) return 0;

    let max = 0;
    for (const dayData of heatmap.data) {
      if (dayData && Array.isArray(dayData)) {
        for (const count of dayData) {
          if (count > max) {
            max = count;
          }
        }
      }
    }
    return max;
  });

  const handleExport = async () => {
    await timeline.exportTimeline(props.project, "/export/path/timeline.json");
  };

  const handleCellClick = (day: number, hour: number, count: number) => {
    setSelectedCell({ day, hour, count });
  };

  const getMostActivePeriods = () => {
    return timeline.getMostActivePeriods();
  };

  const getTrends = () => {
    return timeline.getTrends();
  };

  if (!props.isOpen) return null;

  return (
    <div class="modal-overlay" onClick={props.onClose}>
      {/* Modal Content */}
      <div class="modal-content max-w-7xl w-full max-h-[90vh] flex flex-col" onClick={(e) => e.stopPropagation()}>
        {/* Header */}
        <div class="modal-header">
          <div class="flex items-center gap-3">
            <HiOutlineCalendar class="w-icon-lg h-icon-lg text-accent" />
            <div>
              <h2 class="text-lg font-semibold text-txt">
                Activity Heatmap
              </h2>
              <p class="text-sm text-txt-secondary">
                7-day × 24-hour activity pattern visualization
              </p>
            </div>
          </div>
          <div class="flex items-center gap-2">
            <button
              onClick={handleExport}
              class="btn-sm-primary"
            >
              <HiOutlineArrowDownTray class="w-icon-sm h-icon-sm" />
              Export
            </button>
            <button
              onClick={props.onClose}
              class="icon-btn"
            >
              <HiOutlineX class="w-icon-base h-icon-base" />
            </button>
          </div>
        </div>

        {/* Content */}
        <div class="modal-body">
            <Show
              when={!timeline.loading()}
              fallback={
                <div class="flex items-center justify-center h-full text-txt-muted">
                  Loading activity data...
                </div>
              }
            >
              <div class="space-y-6">
                {/* Heatmap Grid */}
                <div class="bg-bg rounded-lg p-4 border border-border">
                  <h3 class="text-sm font-medium text-txt mb-4">
                    Activity Heatmap
                  </h3>
                  <div class="overflow-x-auto">
                    <div class="inline-block min-w-full">
                      {/* Hour labels */}
                      <div class="flex items-center mb-2">
                        <div class="w-12" /> {/* Spacer for day labels */}
                        <div class="flex-1 flex">
                          <For each={hourLabels}>
                            {(hour) => (
                              <div
                                class="flex-1 text-xs text-txt-muted text-center min-w-[24px]"
                                classList={{
                                  "font-medium": hour % 6 === 0,
                                }}
                              >
                                {hour % 6 === 0 ? hour : ""}
                              </div>
                            )}
                          </For>
                        </div>
                      </div>

                      {/* Heatmap rows */}
                      <For each={dayLabels}>
                        {(dayLabel, dayIndex) => (
                          <div class="flex items-center mb-1">
                            {/* Day label */}
                            <div class="w-12 text-xs text-txt-muted text-right pr-2">
                              {dayLabel}
                            </div>

                            {/* Hour cells */}
                            <div class="flex-1 flex gap-1">
                              <For each={hourLabels}>
                                {(hour) => {
                                  const heatmap = timeline.getHeatmapData();
                                  const dayData = heatmap?.data[dayIndex()];
                                  const count = (dayData && dayData[hour]) ? dayData[hour] : 0;
                                  const colorClass = getIntensityColor(
                                    count,
                                    maxActivityCount()
                                  );

                                  return (
                                    <div
                                      class={`flex-1 aspect-square rounded ${colorClass} border border-border hover:border-accent cursor-pointer transition-all min-w-[24px] min-h-[24px]`}
                                      onClick={() =>
                                        handleCellClick(dayIndex(), hour, count)
                                      }
                                      onMouseEnter={() =>
                                        setHoveredCell({
                                          day: dayIndex(),
                                          hour,
                                          count,
                                        })
                                      }
                                      onMouseLeave={() => setHoveredCell(null)}
                                      title={`${dayLabel} ${hour}:00 - ${count} activities`}
                                    />
                                  );
                                }}
                              </For>
                            </div>
                          </div>
                        )}
                      </For>

                      {/* Legend */}
                      <div class="flex items-center gap-4 mt-4 justify-end">
                        <span class="text-xs text-txt-muted">Less</span>
                        <div class="flex gap-1">
                          <div class="w-4 h-4 rounded bg-accent/25 border border-border" />
                          <div class="w-4 h-4 rounded bg-accent/50 border border-border" />
                          <div class="w-4 h-4 rounded bg-accent/75 border border-border" />
                          <div class="w-4 h-4 rounded bg-accent border border-border" />
                        </div>
                        <span class="text-xs text-txt-muted">More</span>
                      </div>
                    </div>
                  </div>
                </div>

                {/* Activity Details */}
                <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
                  {/* Most Active Days */}
                  <div class="bg-bg rounded-lg p-4 border border-border">
                    <h3 class="text-sm font-medium text-txt mb-3 flex items-center gap-2">
                      <HiOutlineClock class="w-icon-sm h-icon-sm text-accent" />
                      Most Active Days
                    </h3>
                    <div class="space-y-2">
                      <For each={getMostActivePeriods()}>
                        {(day) => (
                          <div class="flex items-center justify-between p-2 bg-bg-secondary rounded">
                            <div class="flex items-center gap-2">
                              <span class="text-sm text-txt">
                                {new Date(day.date).toLocaleDateString()}
                              </span>
                              <span class="text-xs text-txt-muted">
                                {day.unique_users} user{day.unique_users !== 1 ? 's' : ''}
                              </span>
                            </div>
                            <div class="flex items-center gap-2">
                              <div class="w-24 h-2 bg-bg-hover rounded-full overflow-hidden">
                                <div
                                  class="h-full bg-accent"
                                  style={{
                                    width: `${(day.count / maxActivityCount()) * 100}%`,
                                  }}
                                />
                              </div>
                              <span class="text-sm font-medium text-accent">
                                {day.count}
                              </span>
                            </div>
                          </div>
                        )}
                      </For>
                    </div>
                  </div>

                  {/* Trends */}
                  <div class="bg-bg rounded-lg p-4 border border-border">
                    <h3 class="text-sm font-medium text-txt mb-3 flex items-center gap-2">
                      <HiOutlineChartBar class="w-icon-sm h-icon-sm text-accent" />
                      Activity Trends
                    </h3>
                    <div class="space-y-3">
                      <Show when={getTrends()}>
                        {(trends) => (
                          <>
                            <div class="p-3 bg-bg-secondary rounded">
                              <div class="flex items-center justify-between mb-1">
                                <span class="text-xs text-txt-muted">
                                  Peak Hour
                                </span>
                                <span class="text-sm font-medium text-txt">
                                  {trends().peak_hour}:00
                                </span>
                              </div>
                              <p class="text-xs text-txt-secondary">
                                Most active time of day
                              </p>
                            </div>

                            <div class="p-3 bg-bg-secondary rounded">
                              <div class="flex items-center justify-between mb-1">
                                <span class="text-xs text-txt-muted">
                                  Peak Day
                                </span>
                                <span class="text-sm font-medium text-txt">
                                  {trends().peak_day}
                                </span>
                              </div>
                              <p class="text-xs text-txt-secondary">
                                Most active day of week
                              </p>
                            </div>

                            <div class="p-3 bg-bg-secondary rounded">
                              <div class="flex items-center justify-between mb-1">
                                <span class="text-xs text-txt-muted">
                                  Avg Daily Activities
                                </span>
                                <span class="text-sm font-medium text-txt">
                                  {trends().daily_average.toFixed(1)}
                                </span>
                              </div>
                              <p class="text-xs text-txt-secondary">
                                Average across all days
                              </p>
                            </div>

                            <div class="p-3 bg-bg-secondary rounded">
                              <div class="flex items-center justify-between mb-1">
                                <span class="text-xs text-txt-muted">
                                  Trend Direction
                                </span>
                                <span
                                  class={`text-sm font-medium ${
                                    trends().trend_direction === "increasing"
                                      ? "text-success"
                                      : trends().trend_direction === "decreasing"
                                      ? "text-error"
                                      : "text-txt"
                                  }`}
                                >
                                  {trends().trend_direction}
                                </span>
                              </div>
                              <p class="text-xs text-txt-secondary">
                                Activity trend over time
                              </p>
                            </div>
                          </>
                        )}
                      </Show>
                    </div>
                  </div>
                </div>

                {/* Selected Cell Details */}
                <Show when={selectedCell()}>
                  {(cell) => (
                    <div class="bg-accent/10 border border-accent rounded-lg p-4">
                      <h3 class="text-sm font-medium text-accent mb-2">
                        Selected Period
                      </h3>
                      <div class="grid grid-cols-3 gap-4 text-sm">
                        <div>
                          <span class="text-txt-muted">Day:</span>{" "}
                          <span class="text-txt font-medium">
                            {dayLabels[cell().day]}
                          </span>
                        </div>
                        <div>
                          <span class="text-txt-muted">Time:</span>{" "}
                          <span class="text-txt font-medium">
                            {cell().hour}:00 - {cell().hour + 1}:00
                          </span>
                        </div>
                        <div>
                          <span class="text-txt-muted">Activities:</span>{" "}
                          <span class="text-accent font-medium">
                            {cell().count}
                          </span>
                        </div>
                      </div>
                    </div>
                  )}
                </Show>

                {/* Hover Tooltip */}
                <Show when={hoveredCell()}>
                  {(_cell) => (
                    <div class="fixed z-tooltip pointer-events-none">
                      {/* This would need dynamic positioning based on mouse coordinates */}
                      {/* For simplicity, showing selected cell info instead */}
                    </div>
                  )}
                </Show>
              </div>
            </Show>
          </div>
        </div>
      </div>
  );
};
