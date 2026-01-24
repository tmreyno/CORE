import { Component, createSignal, For, Show, onMount, createMemo } from "solid-js";
import { useActivityTimeline } from "../../hooks/useActivityTimeline";
import type { ActivityHeatmap as HeatmapData } from "../../hooks/useActivityTimeline";
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
  projectPath: string;
}

const ActivityHeatmap: Component<ActivityHeatmapProps> = (props) => {
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
    timeline.getVisualization(props.projectPath);
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
    await timeline.exportTimeline(props.projectPath, "/export/path/timeline.json");
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
              <HiOutlineCalendar class="w-icon-lg h-icon-lg text-accent" />
              <div>
                <h2 className="text-lg font-semibold text-txt">
                  Activity Heatmap
                </h2>
                <p className="text-sm text-txt-secondary">
                  7-day × 24-hour activity pattern visualization
                </p>
              </div>
            </div>
            <div className="flex items-center gap-2">
              <button
                onClick={handleExport}
                className="px-3 py-2 bg-accent hover:bg-accent-hover text-white rounded-md flex items-center gap-2"
              >
                <HiOutlineArrowDownTray class="w-icon-sm h-icon-sm" />
                Export
              </button>
              <button
                onClick={props.onClose}
                className="p-2 hover:bg-bg-hover rounded-md text-txt-secondary hover:text-txt"
              >
                <HiOutlineX class="w-icon-base h-icon-base" />
              </button>
            </div>
          </div>

          {/* Content */}
          <div className="flex-1 overflow-auto p-6">
            <Show
              when={!timeline.loading()}
              fallback={
                <div className="flex items-center justify-center h-full text-txt-muted">
                  Loading activity data...
                </div>
              }
            >
              <div className="space-y-6">
                {/* Heatmap Grid */}
                <div className="bg-bg rounded-lg p-4 border border-border">
                  <h3 className="text-sm font-medium text-txt mb-4">
                    Activity Heatmap
                  </h3>
                  <div className="overflow-x-auto">
                    <div className="inline-block min-w-full">
                      {/* Hour labels */}
                      <div className="flex items-center mb-2">
                        <div className="w-12" /> {/* Spacer for day labels */}
                        <div className="flex-1 flex">
                          <For each={hourLabels}>
                            {(hour) => (
                              <div
                                className="flex-1 text-xs text-txt-muted text-center min-w-[24px]"
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
                          <div className="flex items-center mb-1">
                            {/* Day label */}
                            <div className="w-12 text-xs text-txt-muted text-right pr-2">
                              {dayLabel}
                            </div>

                            {/* Hour cells */}
                            <div className="flex-1 flex gap-1">
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
                                      className={`flex-1 aspect-square rounded ${colorClass} border border-border hover:border-accent cursor-pointer transition-all min-w-[24px] min-h-[24px]`}
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
                      <div className="flex items-center gap-4 mt-4 justify-end">
                        <span className="text-xs text-txt-muted">Less</span>
                        <div className="flex gap-1">
                          <div className="w-4 h-4 rounded bg-accent/25 border border-border" />
                          <div className="w-4 h-4 rounded bg-accent/50 border border-border" />
                          <div className="w-4 h-4 rounded bg-accent/75 border border-border" />
                          <div className="w-4 h-4 rounded bg-accent border border-border" />
                        </div>
                        <span className="text-xs text-txt-muted">More</span>
                      </div>
                    </div>
                  </div>
                </div>

                {/* Activity Details */}
                <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
                  {/* Most Active Days */}
                  <div className="bg-bg rounded-lg p-4 border border-border">
                    <h3 className="text-sm font-medium text-txt mb-3 flex items-center gap-2">
                      <HiOutlineClock class="w-icon-sm h-icon-sm text-accent" />
                      Most Active Days
                    </h3>
                    <div className="space-y-2">
                      <For each={getMostActivePeriods()}>
                        {(day) => (
                          <div className="flex items-center justify-between p-2 bg-bg-secondary rounded">
                            <div className="flex items-center gap-2">
                              <span className="text-sm text-txt">
                                {new Date(day.date).toLocaleDateString()}
                              </span>
                              <span className="text-xs text-txt-muted">
                                {day.unique_users} user{day.unique_users !== 1 ? 's' : ''}
                              </span>
                            </div>
                            <div className="flex items-center gap-2">
                              <div className="w-24 h-2 bg-bg-hover rounded-full overflow-hidden">
                                <div
                                  className="h-full bg-accent"
                                  style={{
                                    width: `${(day.count / maxActivityCount()) * 100}%`,
                                  }}
                                />
                              </div>
                              <span className="text-sm font-medium text-accent">
                                {day.count}
                              </span>
                            </div>
                          </div>
                        )}
                      </For>
                    </div>
                  </div>

                  {/* Trends */}
                  <div className="bg-bg rounded-lg p-4 border border-border">
                    <h3 className="text-sm font-medium text-txt mb-3 flex items-center gap-2">
                      <HiOutlineChartBar class="w-icon-sm h-icon-sm text-accent" />
                      Activity Trends
                    </h3>
                    <div className="space-y-3">
                      <Show when={getTrends()}>
                        {(trends) => (
                          <>
                            <div className="p-3 bg-bg-secondary rounded">
                              <div className="flex items-center justify-between mb-1">
                                <span className="text-xs text-txt-muted">
                                  Peak Hour
                                </span>
                                <span className="text-sm font-medium text-txt">
                                  {trends().peak_hour}:00
                                </span>
                              </div>
                              <p className="text-xs text-txt-secondary">
                                Most active time of day
                              </p>
                            </div>

                            <div className="p-3 bg-bg-secondary rounded">
                              <div className="flex items-center justify-between mb-1">
                                <span className="text-xs text-txt-muted">
                                  Peak Day
                                </span>
                                <span className="text-sm font-medium text-txt">
                                  {dayLabels[trends().peak_day]}
                                </span>
                              </div>
                              <p className="text-xs text-txt-secondary">
                                Most active day of week
                              </p>
                            </div>

                            <div className="p-3 bg-bg-secondary rounded">
                              <div className="flex items-center justify-between mb-1">
                                <span className="text-xs text-txt-muted">
                                  Avg Daily Activities
                                </span>
                                <span className="text-sm font-medium text-txt">
                                  {trends().daily_average.toFixed(1)}
                                </span>
                              </div>
                              <p className="text-xs text-txt-secondary">
                                Average across all days
                              </p>
                            </div>

                            <div className="p-3 bg-bg-secondary rounded">
                              <div className="flex items-center justify-between mb-1">
                                <span className="text-xs text-txt-muted">
                                  Trend Direction
                                </span>
                                <span
                                  className={`text-sm font-medium ${
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
                              <p className="text-xs text-txt-secondary">
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
                    <div className="bg-accent/10 border border-accent rounded-lg p-4">
                      <h3 className="text-sm font-medium text-accent mb-2">
                        Selected Period
                      </h3>
                      <div className="grid grid-cols-3 gap-4 text-sm">
                        <div>
                          <span className="text-txt-muted">Day:</span>{" "}
                          <span className="text-txt font-medium">
                            {dayLabels[cell().day]}
                          </span>
                        </div>
                        <div>
                          <span className="text-txt-muted">Time:</span>{" "}
                          <span className="text-txt font-medium">
                            {cell().hour}:00 - {cell().hour + 1}:00
                          </span>
                        </div>
                        <div>
                          <span className="text-txt-muted">Activities:</span>{" "}
                          <span className="text-accent font-medium">
                            {cell().count}
                          </span>
                        </div>
                      </div>
                    </div>
                  )}
                </Show>

                {/* Hover Tooltip */}
                <Show when={hoveredCell()}>
                  {(cell) => (
                    <div className="fixed z-tooltip pointer-events-none">
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
    </>
  );
};

export default ActivityHeatmap;
